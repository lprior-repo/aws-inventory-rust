use super::*;

pub(super) async fn rebuild_edges(
    snapshot: &mut ProjectionSnapshot,
    client: &mut GraphClient,
    limit: u16,
) -> Result<bool, GraphError> {
    let mut cursor: Option<ProjectionEdgeCursor> = None;
    for _ in 0..MAX_REBUILD_PAGES {
        let page = snapshot
            .edges(cursor.take(), limit)
            .await
            .map_err(GraphError::Store)?;
        cursor = page.next;
        for edge in page.edges {
            client.edge(&edge).await?;
        }
        if cursor.is_none() {
            return Ok(true);
        }
    }
    Ok(false)
}

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

pub(super) async fn pending_report(
    store: &BinaryStore,
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
    if error.is_offline() {
        return EntryError::Offline;
    }
    match error {
        GraphError::Redis(_) | GraphError::Query(_) => {
            EntryError::Projection(ProjectionError::Query)
        }
        GraphError::Store(error) => EntryError::Projection(ProjectionError::Store(error)),
        GraphError::OwnershipLost => EntryError::Projection(ProjectionError::OwnershipLost),
        GraphError::Timeout => EntryError::Offline,
    }
}

pub(super) async fn failed_delivery(
    store: &BinaryStore,
    error: GraphError,
    acknowledged: u64,
) -> Result<ProjectionReport, ProjectionError> {
    if error.is_offline() {
        return pending_report(store, acknowledged, true, false).await;
    }
    match error {
        GraphError::Redis(_) | GraphError::Query(_) => Err(ProjectionError::Query),
        GraphError::Store(error) => Err(ProjectionError::Store(error)),
        GraphError::OwnershipLost => Err(ProjectionError::OwnershipLost),
        GraphError::Timeout => pending_report(store, acknowledged, true, false).await,
    }
}

#[derive(Debug, Error)]
pub(super) enum EntryError {
    #[error("FalkorDB is unavailable")]
    Offline,
    #[error(transparent)]
    Projection(#[from] ProjectionError),
    #[error(transparent)]
    Store(#[from] StoreError),
}

pub(super) async fn rebuild_graph(
    store: &BinaryStore,
    client: &mut GraphClient,
    marker: &PendingProjection,
    batch_limit: u16,
) -> Result<Option<u64>, GraphError> {
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
            .map(Some)
            .map_err(GraphError::Store)
    } else {
        Ok(None)
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
            .resources(after.take(), limit)
            .await
            .map_err(GraphError::Store)?;
        let count = page.len();
        after = page.last().map(|resource| resource.key.clone());
        for resource in page {
            client.upsert(&resource).await?;
            project_tags(snapshot, client, &resource.key, limit).await?;
        }
        if count < usize::from(limit) {
            return Ok(true);
        }
    }
    snapshot
        .resources(after, 1)
        .await
        .map(|page| page.is_empty())
        .map_err(GraphError::Store)
}
