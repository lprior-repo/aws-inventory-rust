use super::buffer;
use crate::{EdgeRow, GraphError, GraphLimits, NodeKey};

struct Step<'a> {
    node: &'a NodeKey,
    parent: Option<(usize, usize)>,
}

pub(super) fn shortest(
    edges: &[EdgeRow],
    from: &NodeKey,
    to: &NodeKey,
    limits: GraphLimits,
) -> Result<Option<Vec<EdgeRow>>, GraphError> {
    let mut steps = buffer(limits.nodes())?;
    steps.push(Step {
        node: from,
        parent: None,
    });
    (0..limits.depth()).try_for_each(|_| extend(&mut steps, edges, limits.nodes()))?;
    steps
        .iter()
        .position(|step| step.node == to)
        .map(|index| reconstruct(&steps, edges, index, limits.depth()))
        .transpose()
}

fn extend<'a>(
    steps: &mut Vec<Step<'a>>,
    edges: &'a [EdgeRow],
    bound: usize,
) -> Result<(), GraphError> {
    let width = steps.len();
    edges.iter().enumerate().try_for_each(|(index, edge)| {
        append(steps, width, index, &edge.from, &edge.to, bound)?;
        append(steps, width, index, &edge.to, &edge.from, bound)
    })
}

fn append<'a>(
    steps: &mut Vec<Step<'a>>,
    width: usize,
    edge: usize,
    from: &'a NodeKey,
    to: &'a NodeKey,
    bound: usize,
) -> Result<(), GraphError> {
    if steps.iter().any(|step| step.node == to) {
        return Ok(());
    }
    if let Some(parent) = steps.iter().take(width).position(|step| step.node == from) {
        if steps.len() == bound {
            return Err(GraphError::Limits);
        }
        steps.push(Step {
            node: to,
            parent: Some((parent, edge)),
        });
    }
    Ok(())
}

fn reconstruct(
    steps: &[Step<'_>],
    edges: &[EdgeRow],
    index: usize,
    depth: u8,
) -> Result<Vec<EdgeRow>, GraphError> {
    let (_, mut path) = (0..depth).try_fold(
        (index, buffer(usize::from(depth))?),
        |(index, mut path), _| {
            let step = steps.get(index).ok_or(GraphError::Response)?;
            match step.parent {
                Some((parent, edge)) => {
                    path.push(edges.get(edge).ok_or(GraphError::Response)?.clone());
                    Ok((parent, path))
                }
                None => Ok::<_, GraphError>((index, path)),
            }
        },
    )?;
    path.reverse();
    Ok(path)
}
