use super::*;

struct Step<'a> {
    node: &'a GraphNode,
    parent: Option<(usize, usize)>,
}

pub(super) fn shortest(
    edges: &[GraphEdge],
    from: &GraphNode,
    to: &GraphNode,
    depth: u8,
) -> Result<Option<Vec<GraphEdge>>, ViewError> {
    let mut steps = vec![Step { node: from, parent: None }];
    (0..depth).for_each(|_| {
        let width = steps.len();
        edges.iter().enumerate().for_each(|(index, edge)| {
            append(&mut steps, width, index, &edge.from, &edge.to);
            append(&mut steps, width, index, &edge.to, &edge.from);
        });
    });
    steps.iter().position(|step| step.node == to)
        .map(|index| reconstruct(&steps, edges, index, depth)).transpose()
}


fn append<'a>(
    steps: &mut Vec<Step<'a>>,
    width: usize,
    edge: usize,
    from: &'a GraphNode,
    to: &'a GraphNode,
) {
    if steps.iter().any(|step| step.node == to) {
        return;
    }
    if let Some(parent) = steps.iter().take(width).position(|step| step.node == from) {
        steps.push(Step { node: to, parent: Some((parent, edge)) });
    }
}

fn reconstruct(
    steps: &[Step<'_>],
    edges: &[GraphEdge],
    index: usize,
    depth: u8,
) -> Result<Vec<GraphEdge>, ViewError> {
    let (_, mut path) = (0..depth).try_fold((index, Vec::new()), |(index, mut path), _| {
        let step = steps.get(index).ok_or(ViewError::GraphResponse)?;
        match step.parent {
            Some((parent, edge)) => {
                path.push(edges.get(edge).ok_or(ViewError::GraphResponse)?.clone());
                Ok((parent, path))
            }
            None => Ok::<_, ViewError>((index, path)),
        }
    })?;
    path.reverse();
    Ok(path)
}
#[cfg(test)]
#[path = "path/tests.rs"]
mod tests;
