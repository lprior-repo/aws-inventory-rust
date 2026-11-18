#[path = "graph/path.rs"]
mod path;
#[path = "graph/query.rs"]
mod query;
#[path = "graph/response.rs"]
mod response;
#[path = "graph/tags.rs"]
mod tags;
use crate::ViewError;
use inventory_core::inventory::{AccountId, Kind, Region, ResourceId, ResourceKey};
pub use tags::{ScopedTag, TagAffiliation, TagAffiliations};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNode {
    pub account: String,
    pub region: String,
    pub kind: String,
    pub id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphEdge {
    pub from: GraphNode,
    pub to: GraphNode,
    pub relation: String,
    pub provenance: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNeighborhood {
    pub edges: Vec<GraphEdge>,
    pub truncated: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphLimits {
    pub depth: u8,
    pub nodes: usize,
    pub edges: usize,
}

pub struct GraphConnection {
    client: redis::Client,
    graph: String,
}

impl GraphConnection {
    pub fn connect(endpoint: &str, graph: &str) -> Result<Self, ViewError> {
        if graph.is_empty() || graph.chars().any(char::is_control) {
            return Err(ViewError::GraphName);
        }
        let client = redis::Client::open(endpoint).map_err(ViewError::Graph)?;
        Ok(Self {
            client,
            graph: graph.to_owned(),
        })
    }
    /// Live native tag membership, separate from AWS dependency traversal.
    /// This does not share SQLite's retained snapshot or pin a native revision.
    pub async fn tag_affiliations(
        &self,
        node: &GraphNode,
        limit: usize,
    ) -> Result<TagAffiliations, ViewError> {
        if limit == 0 || limit > crate::MAX_PAGE_LIMIT {
            return Err(ViewError::PageLimit);
        }
        tags::read(&self.client, &self.graph, node, limit).await
    }

    pub async fn neighbors(
        &self,
        node: &GraphNode,
        limits: GraphLimits,
    ) -> Result<GraphNeighborhood, ViewError> {
        validate_limits(limits)?;
        query::neighborhood(&self.client, &self.graph, node, limits).await
    }

    pub async fn blast_radius(
        &self,
        node: &GraphNode,
        limits: GraphLimits,
    ) -> Result<GraphNeighborhood, ViewError> {
        self.neighbors(node, limits).await
    }

    pub async fn path(
        &self,
        from: &GraphNode,
        to: &GraphNode,
        limits: GraphLimits,
    ) -> Result<GraphNeighborhood, ViewError> {
        validate_limits(limits)?;
        key_digest(to)?;
        let neighborhood = self.neighbors(from, limits).await?;
        match path::shortest(&neighborhood.edges, from, to, limits.depth)? {
            Some(edges) => Ok(GraphNeighborhood {
                edges,
                truncated: neighborhood.truncated,
            }),
            None => Ok(GraphNeighborhood {
                edges: Vec::new(),
                truncated: neighborhood.truncated,
            }),
        }
    }
}

fn validate_limits(limits: GraphLimits) -> Result<(), ViewError> {
    if limits.depth == 0
        || limits.depth > 3
        || limits.edges == 0
        || limits.edges > 2000
        || limits.nodes == 0
        || limits.nodes > 500
    {
        return Err(ViewError::GraphLimits);
    }
    Ok(())
}

fn key_digest(node: &GraphNode) -> Result<String, ViewError> {
    let key = ResourceKey {
        account: AccountId::parse(&node.account)?,
        region: Region::parse(&node.region)?,
        kind: Kind::parse(&node.kind)?,
        id: ResourceId::parse(&node.id)?,
    };
    let mut digest = String::with_capacity(64);
    key.digest().as_bytes().iter().try_for_each(|byte| {
        std::fmt::Write::write_fmt(&mut digest, format_args!("{byte:02x}"))
            .map_err(|_| ViewError::GraphResponse)
    })?;
    Ok(digest)
}
