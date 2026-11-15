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
