use super::projection::row_from_row;
use super::*;

async fn enrich_graph(
    mut snapshot: cmdb_view::DatabaseSnapshot,
    graph: &mut Graph,
) -> Result<(), String> {
    for row in graph.nodes.iter_mut().take(DEFAULT_GRAPH_LIMITS.nodes) {
        let node = cmdb_view::GraphNode {
            account: row.account.clone(),
            region: row.region.clone(),
            kind: row.kind.clone(),
            id: row.id.clone(),
        };
        if let Some(resource) = snapshot
            .detail(&node)
            .await
            .map_err(|error| error.to_string())?
        {
            *row = row_from_resource(&resource)?;
        }
    }
    snapshot.finish().await.map_err(|error| error.to_string())
}

pub(super) async fn refresh_page(database: &Database, filter: Filter) -> WorkerEvent {
    let resource_filter = ResourceFilter {
        account: filter.account,
        region: filter.region,
        kind: filter.kind,
        text: (!filter.query.is_empty()).then_some(filter.query),
        tag: filter.tag,
    };
    let result = tokio::time::timeout(Duration::from_secs(10), async {
        let mut snapshot = database
            .snapshot()
            .await
            .map_err(|error| error.to_string())?;
        let page = snapshot
            .page_rows(
                &resource_filter,
                PageRequest {
                    limit: PAGE_LIMIT,
                    after: None,
                },
            )
            .await
            .map_err(|error| error.to_string())?;
        let snapshot_counts = snapshot.counts().await.map_err(|error| error.to_string())?;
        let rows = page
            .rows
            .iter()
            .map(row_from_row)
            .collect::<Result<Vec<_>, _>>()?;
        let event = WorkerEvent::Snapshot {
            counts: snapshot_counts.into(),
            rows,
            total: page.total_matching,
            error: None,
        };
        Ok::<_, String>(event)
    })
    .await;
    match result {
        Ok(Ok(event)) => event,
        Ok(Err(error)) => database_error(error),
        Err(_) => database_error("inventory query timed out after 10 seconds".to_owned()),
    }
}

pub(super) async fn query_graph(state: &mut WorkerState, node: Option<GraphNode>) -> WorkerEvent {
    let event = match node {
        Some(node) => state.graph.expand(node).await,
        None => state.graph.refresh().await,
    };
    let WorkerEvent::Graph {
        mut graph,
        error: None,
    } = event
    else {
        return event;
    };
    let snapshot = match state.database.snapshot().await {
        Ok(s) => s,
        Err(error) => {
            return WorkerEvent::Graph {
                graph,
                error: Some(format!("Failed to open snapshot for enrichment · {error}")),
            };
        }
    };
    let result =
        tokio::time::timeout(Duration::from_secs(2), enrich_graph(snapshot, &mut graph)).await;
    let error = match result {
        Ok(Ok(())) => None,
        Ok(Err(error)) => Some(format!("SQLite snapshot detail query failed · {error}")),
        Err(_) => Some("SQLite snapshot detail query timed out after 2 seconds".to_owned()),
    };
    WorkerEvent::Graph { graph, error }
}

impl From<cmdb_view::DatabaseCounts> for Counts {
    fn from(value: cmdb_view::DatabaseCounts) -> Self {
        Self {
            accounts: value.accounts,
            regions: value.regions,
            services: value.services,
            resources: value.resources,
        }
    }
}
