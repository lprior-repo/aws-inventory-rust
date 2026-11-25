use super::*;

enum GraphConfig {
    Unavailable(String),
    Configured { url: String, name: String },
}

pub(super) struct GraphAccess {
    config: GraphConfig,
    state: Result<GraphSession, GraphFailure>,
    last_node: Option<GraphNode>,
    limits: GraphLimits,
}

struct GraphSession {
    connection: GraphConnection,
    access_error: Option<String>,
}

struct GraphFailure {
    error: String,
    access_failure: bool,
}

fn connect_graph(
    config: &GraphConfig,
    access_error: Option<&str>,
) -> Result<GraphSession, GraphFailure> {
    let result = match config {
        GraphConfig::Unavailable(error) => Err(error.clone()),
        GraphConfig::Configured { url, name } => {
            GraphConnection::connect(url, name).map_err(classify_graph_error)
        }
    };
    result
        .map(|connection| GraphSession {
            connection,
            access_error: access_error.map(str::to_owned),
        })
        .map_err(|error| GraphFailure {
            error,
            access_failure: true,
        })
}

impl GraphAccess {
    pub fn new(url: Option<String>, name: Option<String>, limits: GraphLimits) -> Self {
        let config = match (url, name) {
            (None, None) => {
                GraphConfig::Unavailable("not configured (--graph-url and --graph-name)".to_owned())
            }
            (Some(url), Some(name)) => GraphConfig::Configured { url, name },
            _ => GraphConfig::Unavailable("configure both --graph-url and --graph-name".to_owned()),
        };
        let state = connect_graph(&config, Some("connection not yet checked"));
        Self {
            config,
            state,
            last_node: None,
            limits,
        }
    }

    pub fn reconnect(&mut self) {
        self.state = connect_graph(&self.config, self.access_error());
    }

    pub fn status(&self) -> WorkerEvent {
        WorkerEvent::GraphStatus {
            error: self.access_error().map(str::to_owned),
        }
    }

    fn access_error(&self) -> Option<&str> {
        match &self.state {
            Ok(session) => session.access_error.as_deref(),
            Err(GraphFailure {
                error,
                access_failure: true,
            }) => Some(error),
            Err(GraphFailure {
                access_failure: false,
                ..
            }) => None,
        }
    }

    pub async fn refresh(&mut self) -> WorkerEvent {
        self.reconnect();
        if let Some(node) = self.last_node.take() {
            return self.expand(node).await;
        }
        self.status()
    }

    pub async fn expand(&mut self, node: GraphNode) -> WorkerEvent {
        if self.state.is_err() {
            self.reconnect();
        }
        self.last_node = Some(node.clone());
        let connection = match &self.state {
            Ok(session) => &session.connection,
            Err(failure) => return scoped_graph_failure(&node, failure.error.clone()),
        };
        let result = tokio::time::timeout(Duration::from_secs(8), async {
            let neighborhood = connection.neighbors(&node, self.limits).await?;
            let tags = connection.tag_affiliations(&node, TAG_LIMIT).await?;
            Ok::<_, cmdb_view::ViewError>((neighborhood, tags))
        })
        .await;
        let (event, access_failure) = match result {
            Ok(Ok((neighborhood, tags))) => (neighborhood_event(&node, neighborhood, tags), false),
            Ok(Err(error)) => {
                let access_failure = graph_access_failure(&error);
                (
                    scoped_graph_failure(&node, classify_graph_error(error)),
                    access_failure,
                )
            }
            Err(_) => (
                scoped_graph_failure(
                    &node,
                    "connection/query timed out after 8 seconds".to_owned(),
                ),
                true,
            ),
        };
        self.record(&event, access_failure);
        event
    }

    fn record(&mut self, event: &WorkerEvent, access_failure: bool) {
        match event {
            WorkerEvent::Graph {
                error: Some(error), ..
            } => {
                self.state = Err(GraphFailure {
                    error: error.clone(),
                    access_failure,
                });
            }
            WorkerEvent::Graph { error: None, .. } => {
                if let Ok(session) = &mut self.state {
                    session.access_error = None;
                }
            }
            _ => {}
        }
    }
}

fn graph_access_failure(error: &cmdb_view::ViewError) -> bool {
    match error {
        cmdb_view::ViewError::Backend(
            cmdb_view::GraphError::Transport(_)
            | cmdb_view::GraphError::Timeout
            | cmdb_view::GraphError::GraphName,
        ) => true,
        cmdb_view::ViewError::Backend(cmdb_view::GraphError::Command(message)) => {
            message.contains("NOAUTH") || message.contains("WRONGPASS")
        }
        _ => false,
    }
}

fn classify_graph_error(error: cmdb_view::ViewError) -> String {
    match error {
        cmdb_view::ViewError::Backend(cmdb_view::GraphError::Transport(message)) => {
            let detail = match message.strip_prefix("connection refused: ") {
                Some(message) => message,
                None => message.as_str(),
            };
            format!("connection/query unavailable · {detail}")
        }
        cmdb_view::ViewError::Backend(source) => format!("connection/query unavailable · {source}"),
        cmdb_view::ViewError::GraphTimeout => "connection/query timed out".to_owned(),
        cmdb_view::ViewError::GraphName => "invalid graph configuration · graph name".to_owned(),
        cmdb_view::ViewError::GraphResponse => "invalid graph response".to_owned(),
        other => format!("graph query rejected · {other}"),
    }
}

fn neighborhood_event(
    node: &GraphNode,
    neighborhood: cmdb_view::GraphNeighborhood,
    tags: cmdb_view::TagAffiliations,
) -> WorkerEvent {
    let result = graph_from_edges(neighborhood.edges).and_then(|mut graph| {
        graph.tag_root = Some(projection::graph_node_key(node)?);
        graph.truncated = neighborhood.truncated;
        graph.tag_affiliations = tags.affiliations;
        graph.tags_truncated = tags.truncated;
        Ok(graph)
    });
    match result {
        Ok(graph) => WorkerEvent::Graph { graph, error: None },
        Err(error) => scoped_graph_failure(node, format!("invalid graph identity · {error}")),
    }
}

fn scoped_graph_failure(node: &GraphNode, error: String) -> WorkerEvent {
    match projection::graph_node_key(node) {
        Ok(root) => WorkerEvent::Graph {
            graph: Graph {
                tag_root: Some(root),
                ..Graph::default()
            },
            error: Some(error),
        },
        Err(error) => graph_failure(format!("invalid graph identity · {error}")),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn given_no_selected_node_when_refreshing_then_unchecked_connection_is_not_success() {
        let mut access = GraphAccess::new(
            Some("redis://127.0.0.1:6379".to_owned()),
            Some("inventory".to_owned()),
            DEFAULT_GRAPH_LIMITS,
        );
        assert!(
            matches!(access.refresh().await, WorkerEvent::GraphStatus { error: Some(error) } if error == "connection not yet checked")
        );
    }

    #[test]
    fn given_graph_timeout_when_classified_then_timeout_is_distinct_from_invalid_data() {
        assert_eq!(
            classify_graph_error(cmdb_view::ViewError::GraphTimeout),
            "connection/query timed out"
        );
        assert_eq!(
            classify_graph_error(cmdb_view::ViewError::GraphResponse),
            "invalid graph response"
        );
    }
}
