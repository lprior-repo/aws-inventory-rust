use super::*;
use futures_util::TryStreamExt;

pub(super) fn validate(graph: &str, endpoint: &str, limit: u16) -> Result<(), ProjectionError> {
    let valid_graph = !graph.is_empty()
        && graph.len() <= 128
        && graph
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'));
    if !valid_graph || endpoint.len() > 4096 || limit == 0 || limit > MAX_BATCH {
        return Err(ProjectionError::InvalidConfiguration);
    }
    Ok(())
}

pub(super) async fn pending_report<S: InventoryStore>(
    store: &S,
    acknowledged: u64,
    offline: bool,
    rebuild_incomplete: bool,
) -> Result<ProjectionReport, ProjectionError> {
    let state = store.projection_state().await?;
    Ok(report(
        state.pending,
        acknowledged,
        offline,
        rebuild_incomplete,
    ))
}

pub(super) fn report(
    pending: u64,
    acknowledged: u64,
    offline: bool,
    rebuild_incomplete: bool,
) -> ProjectionReport {
    ProjectionReport {
        acknowledged,
        pending,
        offline,
        rebuild_incomplete,
    }
}

pub(super) fn entry_graph_error(error: GraphError) -> EntryError {
    match error {
        GraphError::Transport(_) | GraphError::Timeout => EntryError::Offline,
        error => EntryError::Projection(error.into()),
    }
}

pub(super) async fn failed_delivery<S: InventoryStore>(
    store: &S,
    error: GraphError,
    acknowledged: u64,
) -> Result<ProjectionReport, ProjectionError> {
    match entry_graph_error(error) {
        EntryError::Offline => pending_report(store, acknowledged, true, false).await,
        EntryError::Projection(error) => Err(error),
        EntryError::Store(error) => Err(ProjectionError::Store(error)),
    }
}

#[derive(Debug, Error)]
pub(super) enum EntryError {
    #[error("graph backend is unavailable")]
    Offline,
    #[error(transparent)]
    Projection(#[from] ProjectionError),
    #[error(transparent)]
    Store(#[from] StoreError),
}

impl From<GraphError> for EntryError {
    fn from(error: GraphError) -> Self {
        entry_graph_error(error)
    }
}

pub(super) async fn rebuild_graph<S: InventoryStore, G: GraphBackend>(
    store: &S,
    client: &G,
    marker: &PendingProjection,
    batch_limit: u16,
) -> Result<Option<u64>, EntryError> {
    client.clear().await?;
    let mut snapshot = store.begin_projection_snapshot().await?;
    let nodes_done =
        rebuild_resources(&mut snapshot, client, batch_limit, RebuildPass::Nodes).await?;
    let edges_done = if nodes_done {
        rebuild_resources(&mut snapshot, client, batch_limit, RebuildPass::Edges).await?
    } else {
        false
    };
    snapshot.finish().await?;
    if !nodes_done || !edges_done {
        return Ok(None);
    }
    Ok(Some(
        store
            .ack_projection(ProjectionAck::full_rebuild(marker.revision)?)
            .await?,
    ))
}

#[derive(Clone, Copy)]
enum RebuildPass {
    Nodes,
    Edges,
}

async fn rebuild_resources<G: GraphBackend>(
    snapshot: &mut impl ProjectionSnapshotRead,
    client: &G,
    limit: u16,
    pass: RebuildPass,
) -> Result<bool, EntryError> {
    let (snapshot, after, done) = stream::iter(0..MAX_REBUILD_PAGES)
        .map(Ok::<_, EntryError>)
        .try_fold(
            (snapshot, None, false),
            |(snapshot, after, done), _| async move {
                if done {
                    return Ok((snapshot, after, done));
                }
                let page = snapshot.resources(after, limit).await?;
                let done = page.len() < usize::from(limit);
                let after = page.last().map(|resource| resource.key.clone());
                project_resources(snapshot, client, page, limit, pass).await?;
                Ok((snapshot, after, done))
            },
        )
        .await?;
    if done {
        return Ok(true);
    }
    Ok(snapshot.resources(after, 1).await?.is_empty())
}

async fn project_resources<G: GraphBackend>(
    snapshot: &mut impl ProjectionSnapshotRead,
    client: &G,
    page: Vec<crate::inventory::ProjectionResource>,
    limit: u16,
    pass: RebuildPass,
) -> Result<(), EntryError> {
    stream::iter(page.into_iter().map(Ok::<_, EntryError>))
        .try_fold(snapshot, |snapshot, resource| async move {
            match pass {
                RebuildPass::Nodes => {
                    let key = resource.key.clone();
                    client.upsert_node(&node_row(resource)).await?;
                    project_tags(snapshot, client, &key, limit).await?;
                }
                RebuildPass::Edges => {
                    deliver_incident_edges(snapshot, client, &resource.key, limit).await?;
                }
            }
            Ok(snapshot)
        })
        .await?;
    Ok(())
}
