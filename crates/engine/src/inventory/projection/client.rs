use super::cypher::{self, QueryError};
use crate::inventory::{ProjectionEdgeHeader, ProjectionResource, StoreError};
use inventory_core::inventory::ResourceKey;
use redis::aio::MultiplexedConnection;
use std::time::Duration;
use thiserror::Error;

pub(super) struct GraphClient {
    connection: MultiplexedConnection,
    graph: String,
}

#[derive(Debug, Error)]
pub(super) enum GraphError {
    #[error("FalkorDB command failed")]
    Redis(#[from] redis::RedisError),
    #[error("FalkorDB command timed out")]
    Timeout,
    #[error("projection query is invalid or exceeds its bound")]
    Query(#[from] QueryError),
    #[error("projection storage failed")]
    Store(#[from] StoreError),
}
impl GraphError {
    pub(super) fn is_offline(&self) -> bool {
        match self {
            Self::Redis(error) => {
                error.is_io_error() || error.is_connection_dropped() || error.is_timeout()
            }
            Self::Timeout => true,
            Self::Query(_) | Self::Store(_) => false,
        }
    }
}

fn index_already_exists(error: &redis::RedisError) -> bool {
    matches!(error.code(), Some("Attribute") | Some("ERR"))
        && matches!(
            error.detail(),
            Some("'key' is already indexed") | Some("\"Attribute\": 'key' is already indexed")
        )
}

impl GraphClient {
    pub(super) async fn connect(endpoint: &str, graph: &str) -> Result<Self, GraphError> {
        let client = redis::Client::open(endpoint)?;
        let connection = tokio::time::timeout(
            Duration::from_secs(5),
            client.get_multiplexed_async_connection(),
        )
        .await
        .map_err(|_| GraphError::Timeout)??;
        Ok(Self {
            connection,
            graph: graph.to_owned(),
        })
    }
    pub(super) async fn ensure_index(&mut self) -> Result<(), GraphError> {
        match self
            .send("CREATE INDEX FOR (n:Resource) ON (n.key)".to_owned())
            .await
        {
            Err(GraphError::Redis(error)) if index_already_exists(&error) => Ok(()),
            result => result,
        }
    }

    pub(super) async fn upsert(&mut self, resource: &ProjectionResource) -> Result<(), GraphError> {
        let query = cypher::node(&resource.key, resource)?;
        self.send(query).await
    }

    pub(super) async fn delete(&mut self, key: &ResourceKey) -> Result<(), GraphError> {
        let query = cypher::delete(key)?;
        self.send(query).await
    }

    pub(super) async fn edge(&mut self, edge: &ProjectionEdgeHeader) -> Result<(), GraphError> {
        let query = cypher::edge(edge)?;
        self.send(query).await
    }
    pub(super) async fn clear_edges(&mut self, key: &ResourceKey) -> Result<(), GraphError> {
        self.send(cypher::clear_edges(key)?).await
    }

    pub(super) async fn clear(&mut self) -> Result<(), GraphError> {
        self.send("MATCH (n) DETACH DELETE n".to_owned()).await
    }

    async fn send(&mut self, query: String) -> Result<(), GraphError> {
        let mut command = redis::cmd("GRAPH.QUERY");
        command.arg(&self.graph).arg(query);
        let response = command.query_async(&mut self.connection);
        let _: redis::Value = tokio::time::timeout(Duration::from_secs(10), response)
            .await
            .map_err(|_| GraphError::Timeout)??;
        Ok(())
    }
}
