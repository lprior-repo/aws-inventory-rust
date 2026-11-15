use super::*;

// One UI inventory session owns one pinned SQLite read transaction. A sweep
// cannot change its counts, resource cursors, edge cursors or binary details.
// Replacement and shutdown drop the read-only transaction (SQLx rolls it back).
pub(super) struct QuerySession {
    snapshot: DatabaseSnapshot,
}

impl QuerySession {
    pub async fn open(database: &Database) -> Result<Self, String> {
        let snapshot = database.snapshot().await.map_err(|error| error.to_string())?;
        Ok(Self { snapshot })
    }

    pub async fn page(&mut self, filter: Filter) -> Result<WorkerEvent, String> {
        let filter = ResourceFilter {
            account: filter.account,
            region: filter.region,
            kind: filter.kind,
            text: (!filter.query.is_empty()).then_some(filter.query),
        };
        let page = self.snapshot.page(&filter, PageRequest { limit: PAGE_LIMIT, after: None })
            .await.map_err(|error| error.to_string())?;
        let rows = page.resources.iter().map(row_from_resource).collect::<Result<Vec<_>, _>>()?;
        let counts = self.snapshot.counts().await.map_err(|error| error.to_string())?;
        Ok(WorkerEvent::Snapshot { counts: counts.into(), rows, total: page.total_matching, error: None })
    }

    async fn enrich(&mut self, graph: &mut Graph) -> Result<(), String> {
        // GraphLimits bounds the number of detail reads, all from the retained
        // inventory session, never a newer sweep. Native-only nodes stay native.
        for row in graph.nodes.iter_mut().take(GRAPH_LIMITS.nodes) {
            let node = GraphNode { account: row.account.clone(), region: row.region.clone(),
                kind: row.kind.clone(), id: row.id.clone() };
            if let Some(resource) = self.snapshot.detail(&node).await.map_err(|error| error.to_string())? {
                *row = row_from_resource(&resource)?;
            }
        }
        Ok(())
    }
}

pub(super) async fn refresh_page(
    database: &Database,
    filter: Filter,
) -> (Option<QuerySession>, WorkerEvent) {
    let result = tokio::time::timeout(Duration::from_secs(10), async {
        let mut next = QuerySession::open(database).await?;
        let event = next.page(filter).await?;
        Ok::<_, String>((next, event))
    }).await;
    match result {
        Ok(Ok((next, event))) => (Some(next), event),
        Ok(Err(error)) => (None, database_error(error)),
        Err(_) => (None, database_error("inventory query timed out after 10 seconds".to_owned())),
    }
}

pub(super) async fn query_graph(state: &mut WorkerState, node: Option<GraphNode>) -> WorkerEvent {
    let event = match node {
        Some(node) => state.graph.expand(node).await,
        None => state.graph.refresh().await,
    };
    let WorkerEvent::Graph { mut graph, error: None } = event else { return event; };
    let result = tokio::time::timeout(Duration::from_secs(2), state.session.enrich(&mut graph)).await;
    let error = match result {
        Ok(Ok(())) => None,
        Ok(Err(error)) => Some(format!("SQLite snapshot detail query failed · {error}")),
        Err(_) => Some("SQLite snapshot detail query timed out after 2 seconds".to_owned()),
    };
    WorkerEvent::Graph { graph, error }
}

impl From<cmdb_view::DatabaseCounts> for Counts {
    fn from(value: cmdb_view::DatabaseCounts) -> Self {
        Self { accounts: value.accounts, regions: value.regions,
            services: value.services, resources: value.resources }
    }
}
