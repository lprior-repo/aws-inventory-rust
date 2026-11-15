use super::*;

pub(super) fn graph_config(
    url: Option<String>,
    name: Option<String>,
) -> (Option<GraphConnection>, Option<String>) {
    match (url, name) {
        (None, None) => (
            None,
            Some("not configured (--graph-url and --graph-name)".to_owned()),
        ),
        (Some(url), Some(name)) => match GraphConnection::connect(&url, &name) {
            Ok(graph) => (Some(graph), None),
            Err(error) => (None, Some(error.to_string())),
        },
        _ => (
            None,
            Some("configure both --graph-url and --graph-name".to_owned()),
        ),
    }
}

pub(super) fn process_command(
    command: Command,
    runtime: &tokio::runtime::Runtime,
    state: &mut WorkerState,
    events: &SyncSender<WorkerEvent>,
    signal: &SignalToUI,
) {
    match command {
        Command::Refresh(filter) => {
            let event = match runtime.block_on(refresh_page(&state.database, filter)) {
                Ok((rows, total, counts)) => snapshot(rows, total, counts),
                Err(error) => database_error(error),
            };
            send_event(events, signal, event);
        }
        Command::Expand(node) => expand_graph(runtime, state, node, events, signal),
        Command::Stop => (),
    }
}

fn snapshot(rows: Vec<Row>, total: u64, counts: Counts) -> WorkerEvent {
    WorkerEvent::Snapshot {
        counts,
        rows,
        total,
        error: None,
    }
}

pub(super) async fn refresh_page(
    database: &Database,
    filter: Filter,
) -> Result<(Vec<Row>, u64, Counts), String> {
    tokio::time::timeout(Duration::from_secs(10), async {
        let (rows, total) = load_page(database, filter).await?;
        let counts = database.counts().await.map_err(|error| error.to_string())?;
        Ok::<_, String>((rows, total, counts.into()))
    })
    .await
    .map_err(|_| "inventory query timed out after 10 seconds".to_owned())?
}

pub(super) async fn load_page(
    database: &Database,
    filter: Filter,
) -> Result<(Vec<Row>, u64), String> {
    let reader_filter = ResourceFilter {
        account: filter.account,
        region: filter.region,
        kind: filter.kind,
        text: (!filter.query.is_empty()).then_some(filter.query),
    };
    let request = PageRequest {
        limit: PAGE_LIMIT,
        after: None,
    };
    let page = database.page(&reader_filter, request).await;
    let page = page.map_err(|error| error.to_string())?;
    let rows = page.resources.iter().map(row_from_resource);
    let rows = rows.collect::<Result<Vec<_>, _>>()?;
    Ok((rows, page.total_matching))
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

pub(super) fn expand_graph(
    runtime: &tokio::runtime::Runtime,
    state: &mut WorkerState,
    node: GraphNode,
    events: &SyncSender<WorkerEvent>,
    signal: &SignalToUI,
) {
    let Some(connection) = &state.graph else {
        let event = WorkerEvent::Graph {
            graph: Graph::default(),
            error: state.graph_error.clone(),
        };
        send_event(events, signal, event);
        return;
    };
    let neighbors = connection.neighbors(&node, GRAPH_LIMITS);
    let result =
        runtime.block_on(async { tokio::time::timeout(Duration::from_secs(8), neighbors).await });
    let event = match result {
        Ok(Ok(neighborhood)) => neighborhood_event(neighborhood),
        Ok(Err(error)) => graph_failure(error.to_string()),
        Err(_) => graph_failure("graph query timed out after 8 seconds".to_owned()),
    };
    send_event(events, signal, event);
}

fn neighborhood_event(neighborhood: cmdb_view::GraphNeighborhood) -> WorkerEvent {
    match graph_from_edges(neighborhood.edges) {
        Ok(mut graph) => {
            graph.truncated = neighborhood.truncated;
            WorkerEvent::Graph { graph, error: None }
        }
        Err(error) => graph_failure(error),
    }
}

pub(super) fn graph_failure(error: String) -> WorkerEvent {
    WorkerEvent::Graph {
        graph: Graph::default(),
        error: Some(error),
    }
}

pub(super) fn database_error(error: String) -> WorkerEvent {
    WorkerEvent::Snapshot {
        counts: Counts::default(),
        rows: Vec::new(),
        total: 0,
        error: Some(error),
    }
}

pub(super) fn send_event(
    events: &SyncSender<WorkerEvent>,
    signal: &SignalToUI,
    event: WorkerEvent,
) {
    match events.send(event) {
        Ok(()) => signal.set(),
        Err(error) => tracing::error!(error = %error, "CMDB UI stopped receiving worker output"),
    }
}
