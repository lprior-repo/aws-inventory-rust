#[path = "runner/ownership.rs"]
mod ownership;
#[path = "runner/rebuild.rs"]
mod rebuild;
#[path = "runner/tags.rs"]
mod tags;
use super::client::{GraphClient, GraphError};
use super::cypher::QueryError;
use crate::inventory::{
    BinaryStore, PendingProjection, ProjectionAck, ProjectionEdgeCursor, ProjectionSnapshot,
    StoreError,
};
use inventory_core::inventory::ResourceKey;
use rebuild::*;
use tags::project_tags;
use thiserror::Error;

const MAX_BATCH: u16 = 1000;
const MAX_REBUILD_PAGES: usize = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionReport {
    pub acknowledged: u64,
    pub pending: u64,
    pub offline: bool,
    pub rebuild_incomplete: bool,
}

#[derive(Debug, Error)]
pub enum ProjectionError {
    #[error("projection storage failed: {0}")]
    Store(#[from] StoreError),
    #[error("projection query is invalid or exceeds its bound")]
    Query,
    #[error("projection graph name or batch limit is invalid")]
    InvalidConfiguration,
    #[error("another projector owns this inventory database")]
    DeliveryBusy,
    #[error("projection delivery ownership was superseded")]
    OwnershipLost,
    #[error("projection ownership file operation failed: {0}")]
    OwnershipIo(#[from] std::io::Error),
}

impl From<QueryError> for ProjectionError {
    fn from(_: QueryError) -> Self {
        Self::Query
    }
}

/// Delivers one bounded batch to the graph owned by this inventory database.
///
/// One database must own one target graph; the outbox is not a multi-target fanout.
/// All projectors (including rebuilds) must use this entrypoint. A persistent
/// `.projection-lock.sqlite` sidecar serializes handles and processes for the
/// entire snapshot/effects/ack interval, conservatively across all graph names.
/// Preserve/restore the sidecar with its inventory; never reset its generation
/// while the target graph exists, even with no open handles. Contention fails
/// immediately with `DeliveryBusy`; ordinary inventory writers do not acquire it.
///
/// Redis CLIENT ID/SET/WATCH/GET/MULTI/EXEC, Lua EVAL and synchronous FalkorDB
/// GRAPH.QUERY inside EXEC are required. Durable generations reject delayed older
/// claims; WATCH fences effects from cancelled/crashed predecessors. Ownership
/// keys must not be externally mutated/evicted. Quiesce old projector versions
/// before upgrading. No expiring lease or unfenced fallback exists. Client deadline is 10s per
/// request; dropped callers leave unfinished delivery pending for successor repair.
/// EXEC runs each native query on Redis's main thread, temporarily serializing
/// other graphs/Redis commands too. A timed-out query may still finish remotely,
/// but its transaction cannot escape the successor's native ownership fence.
/// This is delivery serialization, not an atomic multi-query graph snapshot.
pub async fn run_projection(
    store: &BinaryStore,
    endpoint: &str,
    graph: &str,
    batch_limit: u16,
) -> Result<ProjectionReport, ProjectionError> {
    validate(graph, endpoint, batch_limit)?;
    let ownership = ownership::acquire(store).await?;
    let result = run_owned_projection(store, endpoint, graph, batch_limit, ownership.generation()).await;
    ownership.release().await?;
    result
}

async fn run_owned_projection(
    store: &BinaryStore,
    endpoint: &str,
    graph: &str,
    batch_limit: u16,
    generation: i64,
) -> Result<ProjectionReport, ProjectionError> {
    let state = store.projection_state().await?;
    if state.pending == 0 {
        return Ok(report(state.pending, 0, false, false));
    }
    let mut client = match GraphClient::connect(endpoint, graph, generation).await {
        Ok(client) => client,
        Err(error) => return failed_delivery(store, error, 0).await,
    };
    if let Err(error) = client.ensure_index().await {
        return failed_delivery(store, error, 0).await;
    }
    deliver_batch(store, &mut client, batch_limit).await
}

async fn deliver_batch(
    store: &BinaryStore,
    client: &mut GraphClient,
    batch_limit: u16,
) -> Result<ProjectionReport, ProjectionError> {
    let pending = store.drain_projection(batch_limit).await?;
    if pending.is_empty() {
        return pending_report(store, 0, false, false).await;
    }
    let rebuild = pending.iter().find(|entry| entry.full_rebuild);
    let acknowledged = if let Some(entry) = rebuild {
        match rebuild_graph(store, client, entry, batch_limit).await {
            Ok(Some(acknowledged)) => acknowledged,
            Ok(None) => return pending_report(store, 0, false, true).await,
            Err(error) => return failed_delivery(store, error, 0).await,
        }
    } else {
        0
    };
    deliver_entries(store, client, pending, batch_limit, acknowledged).await
}

async fn deliver_entries(
    store: &BinaryStore,
    client: &mut GraphClient,
    pending: Vec<PendingProjection>,
    batch_limit: u16,
    mut acknowledged: u64,
) -> Result<ProjectionReport, ProjectionError> {
    let mut offline = false;
    for entry in pending {
        if entry.full_rebuild {
            continue;
        }
        match deliver_entry(store, client, &entry, batch_limit).await {
            Ok(applied) => acknowledged = acknowledged.saturating_add(applied),
            Err(EntryError::Offline) => {
                offline = true;
                break;
            }
            Err(EntryError::Projection(error)) => return Err(error),
            Err(EntryError::Store(error)) => return Err(ProjectionError::Store(error)),
        }
    }
    pending_report(store, acknowledged, offline, false).await
}

async fn deliver_entry(
    store: &BinaryStore,
    client: &mut GraphClient,
    entry: &PendingProjection,
    batch_limit: u16,
) -> Result<u64, EntryError> {
    let key = entry.key.as_ref().ok_or(EntryError::Projection(
        ProjectionError::InvalidConfiguration,
    ))?;
    if entry.deleted {
        client.delete(key).await.map_err(entry_graph_error)?;
    } else {
        deliver_resource(store, client, key, batch_limit).await?;
    }
    store
        .ack_projection(ProjectionAck::new(key.clone(), entry.revision)?)
        .await
        .map_err(EntryError::Store)
}

async fn deliver_resource(
    store: &BinaryStore,
    client: &mut GraphClient,
    key: &ResourceKey,
    batch_limit: u16,
) -> Result<(), EntryError> {
    let mut snapshot = store.begin_projection_snapshot().await?;
    match snapshot.resource(key).await? {
        Some(resource) => client.upsert(&resource).await.map_err(entry_graph_error)?,
        None => return client.delete(key).await.map_err(entry_graph_error),
    }
    client.clear_tags(key).await.map_err(entry_graph_error)?;
    project_tags(&mut snapshot, client, key, batch_limit)
        .await
        .map_err(entry_graph_error)?;
    client.clear_edges(key).await.map_err(entry_graph_error)?;
    deliver_incident_edges(&mut snapshot, client, key, batch_limit).await?;
    snapshot.finish().await?;
    Ok(())
}

async fn deliver_incident_edges(
    snapshot: &mut ProjectionSnapshot,
    client: &mut GraphClient,
    key: &ResourceKey,
    batch_limit: u16,
) -> Result<(), EntryError> {
    let mut cursor: Option<ProjectionEdgeCursor> = None;
    for _ in 0..MAX_REBUILD_PAGES {
        let page = snapshot
            .edges_for(key, cursor.take(), batch_limit)
            .await?;
        for edge in page.edges {
            client.edge(&edge).await.map_err(entry_graph_error)?;
        }
        cursor = page.next;
        if cursor.is_none() {
            return Ok(());
        }
    }
    Err(EntryError::Projection(
        ProjectionError::InvalidConfiguration,
    ))
}

#[cfg(test)]
mod tests {
    use super::{ProjectionError, report, validate};

    #[test]
    fn graph_names_and_batches_are_strictly_bounded() {
        assert!(validate("inventory-graph_1", "redis://localhost", 1).is_ok());
        assert!(matches!(
            validate("inventory graph", "redis://localhost", 1),
            Err(ProjectionError::InvalidConfiguration)
        ));
        assert!(matches!(
            validate("inventory", "redis://localhost", 0),
            Err(ProjectionError::InvalidConfiguration)
        ));
        assert!(matches!(
            validate("inventory", "redis://localhost", 1001),
            Err(ProjectionError::InvalidConfiguration)
        ));
    }

    #[test]
    fn report_retains_pending_when_delivery_is_offline_or_rebuild_is_partial() {
        let offline = report(3, 0, true, false);
        let partial = report(2, 1, false, true);
        assert_eq!(offline.pending, 3);
        assert!(offline.offline);
        assert_eq!(partial.pending, 2);
        assert!(partial.rebuild_incomplete);
    }
    #[test]
    fn response_errors_are_fatal_but_transport_errors_are_offline() {
        use super::super::client::GraphError;
        use super::{EntryError, ProjectionError, entry_graph_error};
        let response = redis::RedisError::from((
            redis::ErrorKind::Server(redis::ServerErrorKind::ResponseError),
            "bad query",
        ));
        assert!(matches!(
            entry_graph_error(GraphError::Redis(response)),
            EntryError::Projection(ProjectionError::Query)
        ));
        let transport = redis::RedisError::from(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "offline",
        ));
        assert!(matches!(
            entry_graph_error(GraphError::Redis(transport)),
            EntryError::Offline
        ));
    }
}
