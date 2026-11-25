use futures_util::TryStreamExt;
use inventory_graph::{
    EdgeKey, EdgeRow, GraphBackend, GraphError, GraphLimits, GraphNeighborhood, NodeKey, NodeRow,
    PathResult, TagAffiliations, TagRow,
};
use tokio::sync::Mutex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Call {
    Claim,
    Upsert(NodeRow),
    Delete(NodeKey),
    ClearTags(NodeKey),
    Tag(NodeKey, TagRow),
    Replace(NodeKey, Vec<EdgeRow>),
    Chunk(NodeKey, Vec<EdgeRow>),
    Clear,
}

pub(super) struct RecordingBackend {
    calls: Mutex<Vec<Call>>,
    offline: bool,
    edges: Mutex<Vec<EdgeRow>>,
    fail_second_chunk: Mutex<Option<GraphError>>,
}

impl RecordingBackend {
    pub(super) fn new(offline: bool) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            offline,
            edges: Mutex::new(Vec::new()),
            fail_second_chunk: Mutex::new(None),
        }
    }

    pub(super) async fn calls(&self) -> Vec<Call> {
        self.calls.lock().await.clone()
    }

    pub(super) async fn fail_second_chunk_once(&self, error: GraphError) {
        *self.fail_second_chunk.lock().await = Some(error);
    }

    pub(super) async fn seed_edges(&self, edges: Vec<EdgeRow>) {
        *self.edges.lock().await = edges;
    }

    pub(super) async fn edges(&self) -> Vec<EdgeRow> {
        let mut edges = self.edges.lock().await.clone();
        edges.sort_by(|left, right| {
            (&left.from, &left.to, &left.relation, &left.provenance).cmp(&(
                &right.from,
                &right.to,
                &right.relation,
                &right.provenance,
            ))
        });
        edges
    }

    async fn apply_chunk(
        &self,
        key: &NodeKey,
        edges: Vec<EdgeRow>,
        index: usize,
    ) -> Result<usize, GraphError> {
        if edges.len() > inventory_graph::MAX_EDGES {
            return Err(GraphError::Limits);
        }
        self.record(Call::Chunk(key.clone(), edges.clone())).await?;
        let mut calls = self.calls.lock().await;
        if let Some(Call::Replace(_, requested)) = calls
            .iter_mut()
            .rev()
            .find(|call| matches!(call, Call::Replace(owner, _) if owner == key))
        {
            requested.extend(edges.iter().cloned());
        }
        drop(calls);
        if index == 1
            && let Some(error) = self.fail_second_chunk.lock().await.take()
        {
            return Err(error);
        }
        self.edges.lock().await.extend(edges);
        index.checked_add(1).ok_or(GraphError::Limits)
    }

    async fn record(&self, call: Call) -> Result<(), GraphError> {
        self.calls.lock().await.push(call);
        if self.offline {
            Err(GraphError::Transport("offline test backend".into()))
        } else {
            Ok(())
        }
    }
}

fn unexpected_read<T>() -> std::future::Ready<Result<T, GraphError>> {
    std::future::ready(Err(GraphError::Command(
        "projection unexpectedly requested a graph read".into(),
    )))
}

impl GraphBackend for RecordingBackend {
    fn target_identity(&self) -> Option<(&str, &str)> {
        Some(("recording://projection", "trait_contract"))
    }

    async fn claim_projection(&self, _generation: i64) -> Result<(), GraphError> {
        self.calls.lock().await.push(Call::Claim);
        Ok(())
    }

    async fn upsert_node(&self, node: &NodeRow) -> Result<(), GraphError> {
        self.record(Call::Upsert(node.clone())).await
    }

    async fn delete_node(&self, key: &NodeKey) -> Result<(), GraphError> {
        self.record(Call::Delete(key.clone())).await
    }

    async fn clear_tags(&self, key: &NodeKey) -> Result<(), GraphError> {
        self.record(Call::ClearTags(key.clone())).await
    }

    async fn upsert_tag(&self, key: &NodeKey, tag: &TagRow) -> Result<(), GraphError> {
        self.record(Call::Tag(key.clone(), tag.clone())).await
    }

    async fn replace_edges(&self, key: &NodeKey, edges: &[EdgeRow]) -> Result<(), GraphError> {
        self.record(Call::Replace(key.clone(), edges.to_vec()))
            .await?;
        let mut stored = self.edges.lock().await;
        stored.retain(|edge| &edge.from != key && &edge.to != key);
        stored.extend(edges.iter().cloned());
        Ok(())
    }

    async fn replace_edges_paged<Pages, E>(&self, key: &NodeKey, pages: Pages) -> Result<(), E>
    where
        Pages: futures_util::Stream<Item = Result<Vec<EdgeRow>, E>> + Send,
        E: From<GraphError> + Send,
    {
        self.record(Call::Replace(key.clone(), Vec::new())).await?;
        self.edges
            .lock()
            .await
            .retain(|edge| &edge.from != key && &edge.to != key);
        pages
            .try_fold(0_usize, |index, edges| async move {
                self.apply_chunk(key, edges, index).await.map_err(E::from)
            })
            .await?;
        Ok(())
    }

    async fn clear(&self) -> Result<(), GraphError> {
        self.record(Call::Clear).await
    }

    fn tag_affiliations(
        &self,
        _key: &NodeKey,
        _limit: u16,
    ) -> impl Future<Output = Result<TagAffiliations, GraphError>> + Send {
        unexpected_read()
    }

    fn neighbors(
        &self,
        _key: &NodeKey,
        _limits: GraphLimits,
    ) -> impl Future<Output = Result<GraphNeighborhood, GraphError>> + Send {
        unexpected_read()
    }

    fn blast_radius(
        &self,
        _key: &NodeKey,
        _limits: GraphLimits,
    ) -> impl Future<Output = Result<GraphNeighborhood, GraphError>> + Send {
        unexpected_read()
    }

    fn path(
        &self,
        _from: &NodeKey,
        _to: &NodeKey,
        _limits: GraphLimits,
    ) -> impl Future<Output = Result<PathResult, GraphError>> + Send {
        unexpected_read()
    }

    fn node_keys(
        &self,
        _after: Option<&NodeKey>,
        _limit: u16,
    ) -> impl Future<Output = Result<Vec<NodeKey>, GraphError>> + Send {
        unexpected_read()
    }

    fn edge_keys(
        &self,
        _after: Option<&EdgeKey>,
        _limit: u16,
    ) -> impl Future<Output = Result<Vec<EdgeKey>, GraphError>> + Send {
        unexpected_read()
    }
}
