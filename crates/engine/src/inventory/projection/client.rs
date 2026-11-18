mod cleanup;
mod fence;

use super::cypher::{self, QueryError};
use crate::inventory::projection_read::ProjectionTag;
use crate::inventory::{ProjectionEdgeHeader, ProjectionResource, StoreError};
use inventory_core::inventory::ResourceKey;
use redis::aio::MultiplexedConnection;
use std::time::Duration;
use thiserror::Error;

pub(super) struct GraphClient {
    connection: MultiplexedConnection,
    graph: String,
    fence_key: String,
    fence: String,
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
    #[error("projection ownership was superseded")]
    OwnershipLost,
}
impl GraphError {
    pub(super) fn is_offline(&self) -> bool {
        match self {
            Self::Redis(error) => {
                error.is_io_error() || error.is_connection_dropped() || error.is_timeout()
            }
            Self::Timeout => true,
            Self::Query(_) | Self::Store(_) | Self::OwnershipLost => false,
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
    pub(super) async fn connect(
        endpoint: &str,
        graph: &str,
        generation: i64,
    ) -> Result<Self, GraphError> {
        let client = redis::Client::open(endpoint)?;
        let mut connection = tokio::time::timeout(
            Duration::from_secs(5),
            client.get_multiplexed_async_connection(),
        )
        .await
        .map_err(|_| GraphError::Timeout)??;
        // Committed sidecar generations order claims even across server restarts;
        // CLIENT ID distinguishes the concrete native connection within a claim.
        let fence_key = format!("aws-inventory:projection-owner:{graph}");
        let fence = tokio::time::timeout(
            Duration::from_secs(5),
            fence::claim(&mut connection, &fence_key, generation),
        )
        .await
        .map_err(|_| GraphError::Timeout)??;
        Ok(Self {
            connection,
            graph: graph.to_owned(),
            fence_key,
            fence,
        })
    }
    pub(super) async fn ensure_index(&mut self) -> Result<(), GraphError> {
        self.ensure_label_index("CREATE INDEX FOR (n:Resource) ON (n.key)")
            .await?;
        self.ensure_label_index("CREATE INDEX FOR (n:Tag) ON (n.key)")
            .await
    }

    async fn ensure_label_index(&mut self, query: &str) -> Result<(), GraphError> {
        match self.send(query.to_owned()).await {
            Err(GraphError::Redis(error)) if index_already_exists(&error) => Ok(()),
            result => result,
        }
    }

    pub(super) async fn upsert(&mut self, resource: &ProjectionResource) -> Result<(), GraphError> {
        let query = cypher::node(&resource.key, resource)?;
        self.send(query).await
    }

    pub(super) async fn delete(&mut self, key: &ResourceKey) -> Result<(), GraphError> {
        self.clear_tags(key).await?;
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

    pub(super) async fn tag(
        &mut self,
        key: &ResourceKey,
        tag: &ProjectionTag,
    ) -> Result<(), GraphError> {
        self.send(cypher::tag(key, tag)?).await
    }

    pub(super) async fn clear_tags(&mut self, key: &ResourceKey) -> Result<(), GraphError> {
        self.clear_pages(&cypher::tags_exist(key)?, &cypher::clear_tags(key)?)
            .await
    }

    pub(super) async fn clear(&mut self) -> Result<(), GraphError> {
        self.clear_pages(
            "MATCH (n) RETURN 1 LIMIT 1",
            "MATCH (n) WITH n LIMIT 1000 DETACH DELETE n",
        )
        .await
    }

    async fn send(&mut self, query: String) -> Result<(), GraphError> {
        self.request(query).await.map(|_| ())
    }

    async fn request(&mut self, query: String) -> Result<redis::Value, GraphError> {
        tokio::time::timeout(Duration::from_secs(10), self.fenced_request(query))
            .await
            .map_err(|_| GraphError::Timeout)?
    }
}
