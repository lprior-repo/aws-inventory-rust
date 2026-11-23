#[path = "runner/rebuild.rs"]
mod rebuild;
#[path = "runner/tag_pages.rs"]
mod tag_pages;
#[path = "runner/tags.rs"]
mod tags;
#[cfg(test)]
#[path = "runner/tests.rs"]
mod tests;
use super::client::{GraphClient, GraphError};
use super::cypher::QueryError;
use crate::inventory::{
    InventoryStore, PendingProjection, ProjectionAck, ProjectionDelivery, ProjectionEdgeCursor,
    ProjectionIdentity, ProjectionSnapshotRead, StoreError,
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

pub async fn run_projection<S: InventoryStore>(
    store: &S,
    endpoint: &str,
    graph: &str,
    batch_limit: u16,
) -> Result<ProjectionReport, ProjectionError> {
    validate(graph, endpoint, batch_limit)?;
    let ownership = store
        .acquire_projection_delivery()
        .await
        .map_err(delivery_error)?;
    let result =
        run_owned_projection(store, endpoint, graph, batch_limit, ownership.generation()).await;
    ownership.release().await.map_err(delivery_error)?;
    result
}

async fn run_owned_projection<S: InventoryStore>(
    store: &S,
    endpoint: &str,
    graph: &str,
    batch_limit: u16,
    generation: i64,
) -> Result<ProjectionReport, ProjectionError> {
    let state = store.projection_state().await?;
    if state.pending == 0 {
        return target_identity_check(store, endpoint, graph, state.pending).await;
    }
    let mut client = match GraphClient::connect(endpoint, graph, generation).await {
        Ok(client) => client,
        Err(error) => return failed_delivery(store, error, 0).await,
    };
    if let Err(error) = client.ensure_index().await {
        return failed_delivery(store, error, 0).await;
    }
    let report = deliver_batch(store, &mut client, batch_limit).await?;
    if report.acknowledged > 0 {
        persist_identity(store, endpoint, graph).await?;
    }
    Ok(report)
}

async fn target_identity_check<S: InventoryStore>(
    store: &S,
    endpoint: &str,
    graph: &str,
    pending: u64,
) -> Result<ProjectionReport, ProjectionError> {
    let stored = store.read_projection_identity().await?;
    match stored {
        Some(identity) if identity.differs(endpoint, graph) => {
            store.mark_projection_rebuild_required().await?;
            Ok(report(pending, 0, false, true))
        }
        _ => Ok(report(pending, 0, false, false)),
    }
}

async fn deliver_batch<S: InventoryStore>(
    store: &S,
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

async fn deliver_entries<S: InventoryStore>(
    store: &S,
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

async fn deliver_entry<S: InventoryStore>(
    store: &S,
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

async fn deliver_resource<S: InventoryStore>(
    store: &S,
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
    snapshot: &mut impl ProjectionSnapshotRead,
    client: &mut GraphClient,
    key: &ResourceKey,
    batch_limit: u16,
) -> Result<(), EntryError> {
    let mut cursor: Option<ProjectionEdgeCursor> = None;
    for _ in 0..MAX_REBUILD_PAGES {
        let page = snapshot.edges_for(key, cursor.take(), batch_limit).await?;
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

async fn persist_identity<S: InventoryStore>(
    store: &S,
    endpoint: &str,
    graph: &str,
) -> Result<(), StoreError> {
    store
        .write_projection_identity(&ProjectionIdentity {
            endpoint: endpoint.to_owned(),
            graph: graph.to_owned(),
        })
        .await
}

fn delivery_error(error: StoreError) -> ProjectionError {
    match error {
        StoreError::DeliveryBusy => ProjectionError::DeliveryBusy,
        StoreError::OwnershipIo(error) => ProjectionError::OwnershipIo(error),
        error => ProjectionError::Store(error),
    }
}
