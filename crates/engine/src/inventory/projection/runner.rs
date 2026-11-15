#[path = "runner/rebuild.rs"]
mod rebuild;
use super::client::{GraphClient, GraphError};
use super::cypher::QueryError;
use crate::inventory::{
    BinaryStore, PendingProjection, ProjectionAck, ProjectionEdgeCursor, ProjectionSnapshot,
    StoreError,
};
use inventory_core::inventory::ResourceKey;
use rebuild::*;
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
}

impl From<QueryError> for ProjectionError {
    fn from(_: QueryError) -> Self {
        Self::Query
    }
}

pub async fn run_projection(
    store: &BinaryStore,
    endpoint: &str,
    graph: &str,
    batch_limit: u16,
) -> Result<ProjectionReport, ProjectionError> {
    validate(graph, endpoint, batch_limit)?;
    let state = store.projection_state().await?;
    if state.pending == 0 {
        return Ok(report(state.pending, 0, false, false));
    }
    let mut client = match GraphClient::connect(endpoint, graph).await {
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
    if let Some(entry) = rebuild {
        match rebuild_graph(store, client, entry, batch_limit).await {
            Ok(true) => (),
            Ok(false) => return pending_report(store, 0, false, true).await,
            Err(error) => return failed_delivery(store, error, 0).await,
        }
    }
    deliver_entries(store, client, pending, batch_limit).await
}

async fn deliver_entries(
    store: &BinaryStore,
    client: &mut GraphClient,
    pending: Vec<PendingProjection>,
    batch_limit: u16,
) -> Result<ProjectionReport, ProjectionError> {
    let mut acknowledged = if pending.iter().any(|entry| entry.full_rebuild) {
        1_u64
    } else {
        0_u64
    };
    let mut offline = false;
    for entry in pending {
        if entry.full_rebuild {
            continue;
        }
        match deliver_entry(store, client, &entry, batch_limit).await {
            Ok(()) => acknowledged = acknowledged.saturating_add(1),
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
) -> Result<(), EntryError> {
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
        .await?;
    Ok(())
}

async fn deliver_resource(
    store: &BinaryStore,
    client: &mut GraphClient,
    key: &ResourceKey,
    batch_limit: u16,
) -> Result<(), EntryError> {
    match store.projection_resource(key).await? {
        Some(resource) => client.upsert(&resource).await.map_err(entry_graph_error)?,
        None => return client.delete(key).await.map_err(entry_graph_error),
    }
    client.clear_edges(key).await.map_err(entry_graph_error)?;
    deliver_incident_edges(store, client, key, batch_limit).await
}

async fn deliver_incident_edges(
    store: &BinaryStore,
    client: &mut GraphClient,
    key: &ResourceKey,
    batch_limit: u16,
) -> Result<(), EntryError> {
    let mut cursor: Option<ProjectionEdgeCursor> = None;
    for _ in 0..MAX_REBUILD_PAGES {
        let page = store
            .projection_edges_for(key, cursor.take(), batch_limit)
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

async fn rebuild_graph(
    store: &BinaryStore,
    client: &mut GraphClient,
    marker: &PendingProjection,
    batch_limit: u16,
) -> Result<bool, GraphError> {
    client.clear().await?;
    let mut snapshot = store
        .begin_projection_snapshot()
        .await
        .map_err(GraphError::Store)?;
    let resources_done = rebuild_resources(&mut snapshot, client, batch_limit).await?;
    let edges_done = if resources_done {
        rebuild_edges(&mut snapshot, client, batch_limit).await?
    } else {
        false
    };
    snapshot.finish().await.map_err(GraphError::Store)?;
    if resources_done && edges_done {
        store
            .ack_projection(
                ProjectionAck::full_rebuild(marker.revision).map_err(GraphError::Store)?,
            )
            .await
            .map_err(GraphError::Store)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

async fn rebuild_resources(
    snapshot: &mut ProjectionSnapshot,
    client: &mut GraphClient,
    limit: u16,
) -> Result<bool, GraphError> {
    let mut after: Option<ResourceKey> = None;
    for _ in 0..MAX_REBUILD_PAGES {
        let page = snapshot
            .resources(after.clone(), limit)
            .await
            .map_err(GraphError::Store)?;
        let count = page.len();
        after = page.last().map(|resource| resource.key.clone());
        for resource in page {
            client.upsert(&resource).await?;
        }
        if count < usize::from(limit) {
            return Ok(true);
        }
    }
    Ok(false)
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
