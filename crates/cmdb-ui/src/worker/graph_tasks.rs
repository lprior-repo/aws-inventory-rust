use super::*;

enum GraphConfig {
    Unavailable(String),
    Configured { url: String, name: String },
}

pub(super) struct GraphAccess {
    config: GraphConfig,
    connection: Option<GraphConnection>,
    error: Option<String>,
    last_node: Option<GraphNode>,
}

impl GraphAccess {
    pub fn new(url: Option<String>, name: Option<String>) -> Self {
        let config = match (url, name) {
            (None, None) => GraphConfig::Unavailable(
                "not configured (--graph-url and --graph-name)".to_owned(),
            ),
            (Some(url), Some(name)) => GraphConfig::Configured { url, name },
            _ => GraphConfig::Unavailable("configure both --graph-url and --graph-name".to_owned()),
        };
        let mut access = Self {
            config,
            connection: None,
            last_node: None,
            error: Some("connection not yet checked".to_owned()),
        };
        access.reconnect();
        access
    }

    pub fn reconnect(&mut self) {
        self.connection = None;
        match &self.config {
            GraphConfig::Unavailable(error) => self.error = Some(error.clone()),
            GraphConfig::Configured { url, name } => match GraphConnection::connect(url, name) {
                Ok(connection) => self.connection = Some(connection),
                Err(error) => self.error = Some(classify_graph_error(error)),
            },
        }
    }

    pub fn status(&self) -> WorkerEvent {
        WorkerEvent::Graph { graph: Graph::default(), error: self.error.clone() }
    }
    pub async fn refresh(&mut self) -> WorkerEvent {
        self.reconnect();
        if let Some(node) = self.last_node.take() {
            return self.expand(node).await;
        }
        self.status()
    }


    pub async fn expand(&mut self, node: GraphNode) -> WorkerEvent {
        if self.connection.is_none() {
            self.reconnect();
        }
        self.last_node = Some(node.clone());
        let Some(connection) = &self.connection else {
            return self.status();
        };
        let result = tokio::time::timeout(
            Duration::from_secs(8), connection.neighbors(&node, GRAPH_LIMITS),
        ).await;
        let event = match result {
            Ok(Ok(neighborhood)) => neighborhood_event(neighborhood),
            Ok(Err(error)) => graph_failure(classify_graph_error(error)),
            Err(_) => graph_failure("connection/query timed out after 8 seconds".to_owned()),
        };
        self.record(&event);
        event
    }

    fn record(&mut self, event: &WorkerEvent) {
        if let WorkerEvent::Graph { error, .. } = event {
            self.error = error.clone();
            if error.is_some() {
                self.connection = None;
            }
        }
    }
}

fn classify_graph_error(error: cmdb_view::ViewError) -> String {
    match error {
        cmdb_view::ViewError::Graph(source) => format!("connection/query unavailable · {source}"),
        cmdb_view::ViewError::GraphTimeout => "connection/query timed out".to_owned(),
        cmdb_view::ViewError::GraphName => "invalid graph configuration · graph name".to_owned(),
        cmdb_view::ViewError::GraphResponse => "invalid graph response".to_owned(),
        other => format!("graph query rejected · {other}"),
    }
}

fn neighborhood_event(neighborhood: cmdb_view::GraphNeighborhood) -> WorkerEvent {
    match graph_from_edges(neighborhood.edges) {
        Ok(mut graph) => {
            graph.truncated = neighborhood.truncated;
            WorkerEvent::Graph { graph, error: None }
        }
        Err(error) => graph_failure(format!("invalid graph identity · {error}")),
    }
}

pub(super) fn graph_failure(error: String) -> WorkerEvent {
    WorkerEvent::Graph { graph: Graph::default(), error: Some(error) }
}

pub(super) fn database_error(error: String) -> WorkerEvent {
    WorkerEvent::Snapshot {
        counts: Counts::default(),
        rows: Vec::new(),
        total: 0,
        error: Some(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_query_failure_when_reconnecting_then_offline_status_survives_until_success() {
        let mut access = GraphAccess::new(
            Some("redis://127.0.0.1:6379".to_owned()), Some("inventory".to_owned()),
        );
        access.record(&graph_failure("connection refused".to_owned()));
        assert!(access.connection.is_none());
        access.reconnect();
        assert!(access.connection.is_some());
        assert!(matches!(access.status(), WorkerEvent::Graph { error: Some(error), .. } if error == "connection refused"));
        access.record(&WorkerEvent::Graph { graph: Graph::default(), error: None });
        assert!(matches!(access.status(), WorkerEvent::Graph { error: None, .. }));
    }

    #[tokio::test]
    async fn given_no_selected_node_when_refreshing_then_unchecked_connection_is_not_success() {
        let mut access = GraphAccess::new(
            Some("redis://127.0.0.1:6379".to_owned()), Some("inventory".to_owned()),
        );
        assert!(matches!(access.refresh().await, WorkerEvent::Graph { error: Some(error), .. } if error == "connection not yet checked"));
    }

    #[test]
    fn given_graph_timeout_when_classified_then_timeout_is_distinct_from_invalid_data() {
        assert_eq!(classify_graph_error(cmdb_view::ViewError::GraphTimeout), "connection/query timed out");
        assert_eq!(classify_graph_error(cmdb_view::ViewError::GraphResponse), "invalid graph response");
    }
}
