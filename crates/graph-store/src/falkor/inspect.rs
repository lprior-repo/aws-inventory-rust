use super::FalkorDb;
use crate::{GraphError, NodeRow, cypher, response};
use cypher::Inspection;

/// State of the deterministic `bench-<ordinal>` tag fixture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagFixturePhase {
    Initial,
    Updated,
}

async fn count(graph: &FalkorDb, check: Inspection) -> Result<u64, GraphError> {
    decode_count(&graph.read_request(cypher::inspection(check)).await?)
}

fn decode_count(value: &redis::Value) -> Result<u64, GraphError> {
    let [row] = response::rows(value, 1)? else {
        return Err(GraphError::Response);
    };
    let [redis::Value::Int(count)] = response::fields(row, 1)? else {
        return Err(GraphError::Response);
    };
    u64::try_from(*count).map_err(|_| GraphError::Response)
}

pub async fn resource_count(graph: &FalkorDb) -> Result<u64, GraphError> {
    count(graph, Inspection::Resources).await
}

pub async fn tag_count(graph: &FalkorDb) -> Result<u64, GraphError> {
    count(graph, Inspection::Tags).await
}

pub async fn tag_link_count(graph: &FalkorDb) -> Result<u64, GraphError> {
    count(graph, Inspection::TagLinks).await
}

pub async fn relationship_count(graph: &FalkorDb) -> Result<u64, GraphError> {
    count(graph, Inspection::Relationships).await
}

pub async fn node_count(graph: &FalkorDb) -> Result<u64, GraphError> {
    count(graph, Inspection::Nodes).await
}

pub async fn distinct_resource_count(graph: &FalkorDb) -> Result<u64, GraphError> {
    count(graph, Inspection::DistinctResources).await
}

pub async fn invalid_tag_scope_count(graph: &FalkorDb) -> Result<u64, GraphError> {
    count(graph, Inspection::InvalidTagScopes).await
}

pub async fn orphan_tag_count(graph: &FalkorDb) -> Result<u64, GraphError> {
    count(graph, Inspection::OrphanTags).await
}

/// Counts affiliations inconsistent with the named benchmark fixture phase.
pub async fn invalid_tag_assignment_count(
    graph: &FalkorDb,
    phase: TagFixturePhase,
) -> Result<u64, GraphError> {
    count(graph, Inspection::InvalidTagAssignments(phase)).await
}

/// Counts resources with missing, duplicate or unexpected fixture tag keys.
pub async fn invalid_tag_cardinality_count(
    graph: &FalkorDb,
    phase: TagFixturePhase,
) -> Result<u64, GraphError> {
    count(graph, Inspection::InvalidTagCardinality(phase)).await
}

/// Counts absent or mismatched resources among at most 1,000 named expected rows.
pub async fn resource_mismatch_count(
    graph: &FalkorDb,
    expected: &[NodeRow],
) -> Result<u64, GraphError> {
    if expected.is_empty() {
        return Ok(0);
    }
    decode_count(
        &graph
            .read_request(&cypher::inspect_resources(expected)?)
            .await?,
    )
}

/// Runs the real traversal without induced-edge completion for paired benchmarks.
#[cfg(feature = "test-fixtures")]
pub async fn neighborhood_without_completion(
    graph: &FalkorDb,
    root: &crate::NodeKey,
    limits: crate::GraphLimits,
    incoming: bool,
) -> Result<crate::GraphNeighborhood, GraphError> {
    let direction = if incoming {
        cypher::Direction::Incoming
    } else {
        cypher::Direction::Both
    };
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        crate::traversal::read::<false>(graph, root, limits, direction),
    )
    .await
    .map_err(|_| GraphError::Timeout)?
}

/// Runs the real shortest-path search without induced-edge completion.
#[cfg(feature = "test-fixtures")]
pub async fn path_without_completion(
    graph: &FalkorDb,
    from: &crate::NodeKey,
    to: &crate::NodeKey,
    limits: crate::GraphLimits,
) -> Result<crate::PathResult, GraphError> {
    if from == to {
        return Ok(crate::PathResult::SameNode);
    }
    let neighborhood = neighborhood_without_completion(graph, from, limits, false).await?;
    crate::traversal::shortest(&neighborhood, from, to, limits)
}

#[cfg(test)]
mod tests;
