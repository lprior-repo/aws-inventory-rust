use crate::cypher::Direction;
use crate::{
    EdgeKey, EdgeRow, GraphBackend, GraphError, GraphLimits, GraphNeighborhood, MAX_QUERY_BYTES,
    NodeKey, NodeRow, PathResult, cypher, response, traversal,
};
use std::time::Duration;
use tokio::sync::Semaphore;

impl From<redis::RedisError> for GraphError {
    fn from(error: redis::RedisError) -> Self {
        let mut message = error.to_string();
        if let Some((boundary, _)) = message.char_indices().nth(1_024) {
            message.truncate(boundary);
        }
        Self::Transport(message)
    }
}

pub struct FalkorDb {
    client: redis::Client,
    graph: String,
    requests: Semaphore,
}

impl FalkorDb {
    pub fn connect(endpoint: &str, graph: &str) -> Result<Self, GraphError> {
        if graph.is_empty() || graph.len() > 128 || graph.chars().any(char::is_control) {
            return Err(GraphError::GraphName);
        }
        Ok(Self {
            client: redis::Client::open(endpoint)?,
            graph: graph.to_owned(),
            requests: Semaphore::new(16),
        })
    }

    pub(crate) async fn request(&self, query: &str) -> Result<redis::Value, GraphError> {
        if query.len() > MAX_QUERY_BYTES {
            return Err(GraphError::QueryTooLarge);
        }
        tokio::time::timeout(Duration::from_secs(10), self.execute(query))
            .await
            .map_err(|_| GraphError::Timeout)?
    }

    async fn execute(&self, query: &str) -> Result<redis::Value, GraphError> {
        let _permit = self
            .requests
            .acquire()
            .await
            .map_err(|_| GraphError::AdmissionClosed)?;
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        Ok(redis::cmd("GRAPH.QUERY")
            .arg(&self.graph)
            .arg(query)
            .arg("TIMEOUT")
            .arg(10_000_u32)
            .query_async(&mut connection)
            .await?)
    }

    async fn neighborhood(
        &self,
        key: &NodeKey,
        limits: GraphLimits,
        direction: Direction,
    ) -> Result<GraphNeighborhood, GraphError> {
        tokio::time::timeout(
            Duration::from_secs(10),
            traversal::read(self, key, limits, direction),
        )
        .await
        .map_err(|_| GraphError::Timeout)?
    }
}

impl GraphBackend for FalkorDb {
    async fn upsert_node(&self, row: &NodeRow) -> Result<(), GraphError> {
        self.request(&cypher::node(row)?).await.map(|_| ())
    }

    async fn delete_node(&self, key: &NodeKey) -> Result<(), GraphError> {
        self.request(&cypher::delete(key)?).await.map(|_| ())
    }

    async fn replace_edges(&self, key: &NodeKey, edges: &[EdgeRow]) -> Result<(), GraphError> {
        self.request(&cypher::replace_edges(key, edges)?)
            .await
            .map(|_| ())
    }

    async fn clear(&self) -> Result<(), GraphError> {
        self.request(cypher::clear()).await?;
        let value = self.request(cypher::has_nodes()).await?;
        if !response::rows(&value, 1)?.is_empty() {
            return Err(GraphError::Limits);
        }
        Ok(())
    }

    async fn neighbors(
        &self,
        key: &NodeKey,
        limits: GraphLimits,
    ) -> Result<GraphNeighborhood, GraphError> {
        self.neighborhood(key, limits, Direction::Both).await
    }

    async fn path(
        &self,
        from: &NodeKey,
        to: &NodeKey,
        limits: GraphLimits,
    ) -> Result<PathResult, GraphError> {
        if from == to {
            return Ok(PathResult::SameNode);
        }
        let neighborhood = self.neighbors(from, limits).await?;
        traversal::shortest(&neighborhood, from, to, limits)
    }

    async fn blast_radius(
        &self,
        key: &NodeKey,
        limits: GraphLimits,
    ) -> Result<GraphNeighborhood, GraphError> {
        self.neighborhood(key, limits, Direction::Incoming).await
    }

    async fn node_keys(
        &self,
        after: Option<&NodeKey>,
        limit: u16,
    ) -> Result<Vec<NodeKey>, GraphError> {
        let value = self.request(&cypher::node_keys(after, limit)?).await?;
        response::nodes(&value, usize::from(limit))
    }

    async fn edge_keys(
        &self,
        after: Option<&EdgeKey>,
        limit: u16,
    ) -> Result<Vec<EdgeKey>, GraphError> {
        let value = self.request(&cypher::edge_keys(after, limit)?).await?;
        response::edges(&value, usize::from(limit))
    }
}
