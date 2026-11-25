pub mod inspect;
use crate::cypher::Direction;
use crate::{
    EdgeKey, EdgeRow, GraphBackend, GraphError, GraphLimits, GraphNeighborhood, MAX_QUERY_BYTES,
    NodeKey, NodeRow, PathResult, cypher, response, traversal,
};
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore, SemaphorePermit};
use tokio::time::{Instant, timeout_at};

impl From<redis::RedisError> for GraphError {
    fn from(error: redis::RedisError) -> Self {
        let mut message = if error.is_connection_refusal() {
            format!("connection refused: {error}")
        } else if error.is_connection_dropped() {
            format!("connection reset: {error}")
        } else {
            error.to_string()
        };
        if let Some((boundary, _)) = message.char_indices().nth(1_024) {
            message.truncate(boundary);
        }
        if error.is_io_error() || error.is_connection_dropped() || error.is_timeout() {
            Self::Transport(message)
        } else {
            Self::Command(message)
        }
    }
}

pub struct FalkorDb {
    client: redis::Client,
    graph: String,
    requests: Semaphore,
    endpoint: String,
    session: Mutex<Option<crate::fence::Session>>,
}

pub(crate) struct ReadSession<'a> {
    graph: &'a str,
    connection: redis::aio::MultiplexedConnection,
    _permit: SemaphorePermit<'a>,
    deadline: Instant,
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
            endpoint: endpoint.to_owned(),
            session: Mutex::new(None),
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

    pub(crate) async fn read_session(&self) -> Result<ReadSession<'_>, GraphError> {
        let deadline = Instant::now()
            .checked_add(Duration::from_secs(10))
            .ok_or(GraphError::Timeout)?;
        timeout_at(deadline, async {
            let permit = self
                .requests
                .acquire()
                .await
                .map_err(|_| GraphError::AdmissionClosed)?;
            let connection = self
                .client
                .get_multiplexed_async_connection_with_config(
                    &redis::AsyncConnectionConfig::new().set_response_timeout(None),
                )
                .await?;
            Ok(ReadSession {
                graph: &self.graph,
                connection,
                _permit: permit,
                deadline,
            })
        })
        .await
        .map_err(|_| GraphError::Timeout)?
    }

    pub(crate) async fn read_request(&self, query: &str) -> Result<redis::Value, GraphError> {
        if query.len() > MAX_QUERY_BYTES {
            return Err(GraphError::QueryTooLarge);
        }
        self.read_session().await?.request(query).await
    }

    async fn execute(&self, query: &str) -> Result<redis::Value, GraphError> {
        let _permit = self
            .requests
            .acquire()
            .await
            .map_err(|_| GraphError::AdmissionClosed)?;
        let mut session = self.session.lock().await;
        let session = session.as_mut().ok_or(GraphError::ProjectionNotClaimed)?;
        session.request(&self.graph, query).await
    }

    async fn neighborhood(
        &self,
        key: &NodeKey,
        limits: GraphLimits,
        direction: Direction,
    ) -> Result<GraphNeighborhood, GraphError> {
        tokio::time::timeout(
            Duration::from_secs(10),
            traversal::read::<true>(self, key, limits, direction),
        )
        .await
        .map_err(|_| GraphError::Timeout)?
    }

    async fn apply_edge_chunks(
        &self,
        owner: &NodeKey,
        edges: &[EdgeRow],
        clear: bool,
    ) -> Result<(), GraphError> {
        use futures_util::TryStreamExt;
        futures_util::stream::iter(cypher::edge_chunks(edges, clear).map(Ok::<_, GraphError>))
            .try_for_each(|chunk| async move {
                let reply = self
                    .request(&cypher::edge_chunk_query(owner, chunk)?)
                    .await?;
                response::replacement_owner(&reply, owner)
            })
            .await
    }
}

impl GraphBackend for FalkorDb {
    fn target_identity(&self) -> Option<(&str, &str)> {
        Some((&self.endpoint, &self.graph))
    }

    async fn claim_projection(&self, generation: i64) -> Result<(), GraphError> {
        let session = tokio::time::timeout(Duration::from_secs(5), async {
            let connection = self
                .client
                .get_multiplexed_async_connection_with_config(
                    &redis::AsyncConnectionConfig::new().set_response_timeout(None),
                )
                .await?;
            crate::fence::Session::claim(connection, &self.graph, generation).await
        })
        .await
        .map_err(|_| GraphError::Timeout)??;
        *self.session.lock().await = Some(session);
        self.request(cypher::RESOURCE_INDEX).await?;
        self.request(cypher::TAG_INDEX).await?;
        Ok(())
    }

    async fn clear_tags(&self, key: &NodeKey) -> Result<(), GraphError> {
        crate::fence::clear_pages(self, &cypher::tags_exist(key)?, &cypher::clear_tags(key)?).await
    }

    async fn upsert_tag(&self, key: &NodeKey, tag: &crate::TagRow) -> Result<(), GraphError> {
        self.request(&cypher::tag(key, tag)?).await.map(|_| ())
    }

    async fn tag_affiliations(
        &self,
        key: &NodeKey,
        limit: u16,
    ) -> Result<crate::TagAffiliations, GraphError> {
        crate::tags::read(self, key, limit).await
    }

    async fn upsert_node(&self, row: &NodeRow) -> Result<(), GraphError> {
        self.request(&cypher::node(row)?).await.map(|_| ())
    }

    async fn delete_node(&self, key: &NodeKey) -> Result<(), GraphError> {
        self.clear_tags(key).await?;
        self.request(&cypher::delete(key)?).await.map(|_| ())
    }

    async fn replace_edges(&self, key: &NodeKey, edges: &[EdgeRow]) -> Result<(), GraphError> {
        if edges.len() > crate::MAX_EDGES {
            return Err(GraphError::Limits);
        }
        if edges
            .iter()
            .any(|edge| &edge.from != key && &edge.to != key)
        {
            return Err(GraphError::UnrelatedEdge);
        }
        let one_relation = edges
            .first()
            .is_none_or(|first| edges.iter().all(|edge| edge.relation == first.relation));
        if edges.len() > cypher::EDGE_BATCH_ROWS || !one_relation {
            return self.apply_edge_chunks(key, edges, true).await;
        }
        let reply = self.request(&cypher::replace_edges(key, edges)?).await?;
        response::replacement_owner(&reply, key)
    }

    async fn replace_edges_paged<Pages, E>(&self, key: &NodeKey, pages: Pages) -> Result<(), E>
    where
        Pages: futures_util::Stream<Item = Result<Vec<EdgeRow>, E>> + Send,
        E: From<GraphError> + Send,
    {
        use futures_util::TryStreamExt;
        self.apply_edge_chunks(key, &[], true).await?;
        pages
            .try_for_each(|edges| async move {
                if edges.len() > crate::MAX_EDGES {
                    return Err(E::from(GraphError::Limits));
                }
                self.apply_edge_chunks(key, &edges, false)
                    .await
                    .map_err(E::from)
            })
            .await
    }

    async fn clear(&self) -> Result<(), GraphError> {
        crate::fence::clear_pages(self, cypher::has_nodes(), cypher::clear_page()).await
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
        let value = self.read_request(&cypher::node_keys(after, limit)?).await?;
        response::nodes(&value, usize::from(limit))
    }

    async fn edge_keys(
        &self,
        after: Option<&EdgeKey>,
        limit: u16,
    ) -> Result<Vec<EdgeKey>, GraphError> {
        let value = self.read_request(&cypher::edge_keys(after, limit)?).await?;
        response::edges(&value, usize::from(limit))
    }
}

#[cfg(test)]
mod tests;

impl ReadSession<'_> {
    pub(crate) async fn request(&mut self, query: &str) -> Result<redis::Value, GraphError> {
        if query.len() > MAX_QUERY_BYTES {
            return Err(GraphError::QueryTooLarge);
        }
        timeout_at(self.deadline, async {
            Ok(redis::cmd(cypher::READ_COMMAND)
                .arg(self.graph)
                .arg(query)
                .arg("TIMEOUT")
                .arg(10_000_u32)
                .query_async(&mut self.connection)
                .await?)
        })
        .await
        .map_err(|_| GraphError::Timeout)?
    }
}
