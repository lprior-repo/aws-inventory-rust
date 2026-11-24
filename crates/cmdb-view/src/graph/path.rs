use super::*;

pub(super) fn convert(
    result: Result<inventory_graph::PathResult, GraphError>,
) -> Result<PathResult, ViewError> {
    Ok(match result {
        Err(GraphError::Truncated) => PathResult::Truncated,
        Ok(inventory_graph::PathResult::NotFound) => PathResult::NotFound,
        Err(error) => return Err(graph_error(error)),
        Ok(inventory_graph::PathResult::SameNode) => PathResult::SameNode,
        Ok(inventory_graph::PathResult::Path(edges)) => {
            PathResult::Path(edges.into_iter().map(graph_edge).collect())
        }
    })
}
