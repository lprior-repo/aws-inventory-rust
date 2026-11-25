#[path = "runner/rebuild.rs"]
mod rebuild;
#[path = "runner/tag_pages.rs"]
mod tag_pages;
#[path = "runner/tags.rs"]
mod tags;
#[cfg(test)]
#[path = "runner/tests.rs"]
mod tests;
use crate::inventory::{
    InventoryStore, PendingProjection, ProjectionAck, ProjectionDelivery, ProjectionIdentity,
    ProjectionSnapshotRead, StoreError,
};
use futures_util::{StreamExt, TryStreamExt, stream};
use inventory_core::inventory::ResourceKey;
use inventory_graph::{EdgeRow, GraphBackend, GraphError, NodeRow, Provenance};
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

impl From<GraphError> for ProjectionError {
    fn from(error: GraphError) -> Self {
        match error {
            GraphError::OwnershipLost => Self::OwnershipLost,
            _ => Self::Query,
        }
    }
}

pub fn run_projection<S: InventoryStore, G: GraphBackend>(
    store: &S,
    graph: &G,
    batch_limit: u16,
) -> impl Future<Output = Result<ProjectionReport, ProjectionError>> + Send {
    let configuration = match graph.target_identity() {
        Some((endpoint, name)) => validate(name, endpoint, batch_limit),
        None => validate("inventory", "", batch_limit),
    };
    async move {
        configuration?;
        let ownership = store
            .acquire_projection_delivery()
            .await
            .map_err(delivery_error)?;
        let result = run_owned_projection(store, graph, batch_limit, ownership.generation()).await;
        ownership.release().await.map_err(delivery_error)?;
        result
    }
}

async fn run_owned_projection<S: InventoryStore, G: GraphBackend>(
    store: &S,
    graph: &G,
    batch_limit: u16,
    generation: i64,
) -> Result<ProjectionReport, ProjectionError> {
    let state = store.projection_state().await?;
    if destination_needs_rebuild(store, graph, state.pending).await? {
        store.mark_projection_rebuild_required().await?;
        return Ok(report(state.pending, 0, false, true));
    }
    if state.pending == 0 {
        return Ok(report(0, 0, false, false));
    }
    if let Err(error) = graph.claim_projection(generation).await {
        return failed_delivery(store, error, 0).await;
    }
    let report = deliver_batch(store, graph, batch_limit).await?;
    if report.acknowledged > 0 {
        persist_identity(store, graph).await?;
    }
    Ok(report)
}

/// A changed destination may only be fed by a full rebuild: deltas computed for
/// the previous destination would leave the new graph incomplete. The check runs
/// before any delivery regardless of the pending count, and the new identity is
/// only recorded (by `persist_identity`) after a queued rebuild was delivered.
async fn destination_needs_rebuild<S: InventoryStore, G: GraphBackend>(
    store: &S,
    backend: &G,
    pending: u64,
) -> Result<bool, StoreError> {
    let stored = store.read_projection_identity().await?;
    let changed = stored
        .zip(backend.target_identity())
        .is_some_and(|(identity, (endpoint, graph))| identity.differs(endpoint, graph));
    if !changed || pending == 0 {
        return Ok(changed);
    }
    let head = store.drain_projection(1).await?;
    Ok(!head.first().is_some_and(|entry| entry.full_rebuild))
}

async fn deliver_batch<S: InventoryStore, G: GraphBackend>(
    store: &S,
    client: &G,
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
            Err(EntryError::Offline) => return pending_report(store, 0, true, false).await,
            Err(EntryError::Projection(error)) => return Err(error),
            Err(EntryError::Store(error)) => return Err(ProjectionError::Store(error)),
        }
    } else {
        0
    };
    deliver_entries(store, client, pending, batch_limit, acknowledged).await
}

async fn deliver_entries<S: InventoryStore, G: GraphBackend>(
    store: &S,
    client: &G,
    pending: Vec<PendingProjection>,
    batch_limit: u16,
    acknowledged: u64,
) -> Result<ProjectionReport, ProjectionError> {
    let (acknowledged, offline) = stream::iter(pending)
        .filter(|entry| std::future::ready(!entry.full_rebuild))
        .map(Ok::<_, ProjectionError>)
        .try_fold(
            (acknowledged, false),
            |(acknowledged, offline), entry| async move {
                if offline {
                    return Ok((acknowledged, offline));
                }
                match deliver_entry(store, client, &entry, batch_limit).await {
                    Ok(applied) => Ok((acknowledged.saturating_add(applied), false)),
                    Err(EntryError::Offline) => Ok((acknowledged, true)),
                    Err(EntryError::Projection(error)) => Err(error),
                    Err(EntryError::Store(error)) => Err(ProjectionError::Store(error)),
                }
            },
        )
        .await?;
    pending_report(store, acknowledged, offline, false).await
}

async fn deliver_entry<S: InventoryStore, G: GraphBackend>(
    store: &S,
    client: &G,
    entry: &PendingProjection,
    batch_limit: u16,
) -> Result<u64, EntryError> {
    let key = entry.key.as_ref().ok_or(EntryError::Projection(
        ProjectionError::InvalidConfiguration,
    ))?;
    if entry.deleted {
        client.delete_node(key).await.map_err(entry_graph_error)?;
    } else {
        deliver_resource(store, client, key, batch_limit).await?;
    }
    store
        .ack_projection(ProjectionAck::new(key.clone(), entry.revision)?)
        .await
        .map_err(EntryError::Store)
}

async fn deliver_resource<S: InventoryStore, G: GraphBackend>(
    store: &S,
    client: &G,
    key: &ResourceKey,
    batch_limit: u16,
) -> Result<(), EntryError> {
    let mut snapshot = store.begin_projection_snapshot().await?;
    let resource = snapshot.resource(key).await?;
    let Some(resource) = resource else {
        snapshot.finish().await?;
        return client.delete_node(key).await.map_err(entry_graph_error);
    };
    client
        .upsert_node(&node_row(resource))
        .await
        .map_err(entry_graph_error)?;
    client.clear_tags(key).await.map_err(entry_graph_error)?;
    project_tags(&mut snapshot, client, key, batch_limit).await?;
    deliver_incident_edges(&mut snapshot, client, key, batch_limit).await?;
    snapshot.finish().await?;
    Ok(())
}

async fn deliver_incident_edges<G: GraphBackend>(
    snapshot: &mut impl ProjectionSnapshotRead,
    client: &G,
    key: &ResourceKey,
    batch_limit: u16,
) -> Result<(), EntryError> {
    let pages = stream::try_unfold(
        (snapshot, None, MAX_REBUILD_PAGES, false),
        |(snapshot, cursor, remaining, done)| async move {
            if done {
                return Ok(None);
            }
            let remaining = remaining.checked_sub(1).ok_or(ProjectionError::Query)?;
            let page = snapshot.edges_for(key, cursor, batch_limit).await?;
            let done = page.next.is_none();
            Ok::<_, EntryError>(Some((page.edges, (snapshot, page.next, remaining, done))))
        },
    )
    .map_ok(|page| {
        stream::iter(page.into_iter().map(move |edge| {
            if &edge.from != key && &edge.to != key {
                return Err(GraphError::UnrelatedEdge.into());
            }
            Ok::<_, EntryError>(EdgeRow {
                from: edge.from,
                to: edge.to,
                relation: edge.relation,
                provenance: Provenance::parse(&edge.source)?,
            })
        }))
    })
    .try_flatten()
    .try_chunks(inventory_graph::MAX_EDGES)
    .map_err(|error| error.1);
    client.replace_edges_paged(key, pages).await
}

async fn persist_identity<S: InventoryStore, G: GraphBackend>(
    store: &S,
    backend: &G,
) -> Result<(), StoreError> {
    if let Some((endpoint, graph)) = backend.target_identity() {
        store
            .write_projection_identity(&ProjectionIdentity {
                endpoint: endpoint.to_owned(),
                graph: graph.to_owned(),
            })
            .await?;
    }
    Ok(())
}

fn node_row(resource: crate::inventory::ProjectionResource) -> NodeRow {
    NodeRow {
        key: resource.key,
        name: resource.name,
        content_hash: resource.content_hash,
    }
}

fn delivery_error(error: StoreError) -> ProjectionError {
    match error {
        StoreError::DeliveryBusy => ProjectionError::DeliveryBusy,
        StoreError::OwnershipIo(error) => ProjectionError::OwnershipIo(error),
        error => ProjectionError::Store(error),
    }
}
