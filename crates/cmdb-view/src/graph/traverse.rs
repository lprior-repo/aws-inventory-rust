use super::{EdgeView, GraphLimits, GraphNode, Subgraph};
use crate::resource::ResourceView;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn build_adjacency(edges: &[EdgeView]) -> BTreeMap<String, Vec<usize>> {
    let mut adjacency: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, edge) in edges.iter().enumerate() {
        adjacency
            .entry(edge.from_id.clone())
            .or_default()
            .push(index);
        if edge.from_id != edge.to_id {
            adjacency.entry(edge.to_id.clone()).or_default().push(index);
        }
    }
    for (id, indices) in &mut adjacency {
        indices.sort_unstable_by(|left, right| {
            edge_key(edges, id, *left).cmp(&edge_key(edges, id, *right))
        });
    }
    adjacency
}

fn edge_key<'a>(
    edges: &'a [EdgeView],
    id: &str,
    index: usize,
) -> (&'a str, &'a str, &'a str, &'a str) {
    let Some(edge) = edges.get(index) else {
        return ("", "", "", "");
    };
    let other = other_endpoint(edge, id);
    (
        other,
        edge.relation.as_str(),
        edge.from_id.as_str(),
        edge.to_id.as_str(),
    )
}

fn other_endpoint<'a>(edge: &'a EdgeView, id: &str) -> &'a str {
    if edge.from_id == id {
        edge.to_id.as_str()
    } else {
        edge.from_id.as_str()
    }
}

struct Walk<'a> {
    included: BTreeSet<&'a str>,
    truncated_nodes: bool,
    truncated_depth: bool,
}

fn walk<'a>(
    root_id: &'a str,
    limits: GraphLimits,
    adjacency: &BTreeMap<String, Vec<usize>>,
    edges: &'a [EdgeView],
) -> Walk<'a> {
    let mut included: BTreeSet<&'a str> = BTreeSet::new();
    included.insert(root_id);
    let (frontier, truncated_nodes, exhausted) =
        expand_levels(&mut included, root_id, limits, adjacency, edges);
    let truncated_depth = !exhausted && boundary_crossed(adjacency, edges, &frontier, &included);
    Walk {
        included,
        truncated_nodes,
        truncated_depth,
    }
}

fn expand_levels<'a>(
    included: &mut BTreeSet<&'a str>,
    root_id: &'a str,
    limits: GraphLimits,
    adjacency: &BTreeMap<String, Vec<usize>>,
    edges: &'a [EdgeView],
) -> (Vec<&'a str>, bool, bool) {
    let mut frontier = vec![root_id];
    let mut truncated = false;
    let mut exhausted = false;
    for _ in 0..limits.depth() {
        let (next, capped) = expand(adjacency, edges, &frontier, included, limits.max_nodes());
        truncated |= capped;
        exhausted = next.is_empty();
        if exhausted {
            break;
        }
        included.extend(next.iter().copied());
        frontier = next.into_iter().collect();
    }
    (frontier, truncated, exhausted)
}

pub(crate) fn neighborhood<'a>(
    root_id: &'a str,
    limits: GraphLimits,
    resources: &'a [ResourceView],
    index: &'a BTreeMap<String, usize>,
    edges: &'a [EdgeView],
    adjacency: &'a BTreeMap<String, Vec<usize>>,
) -> Subgraph<'a> {
    let walk = walk(root_id, limits, adjacency, edges);
    let (selected, truncated_edges) =
        collect_edges(&walk.included, adjacency, edges, limits.max_edges());
    Subgraph {
        root: GraphNode {
            id: root_id,
            resource: lookup(resources, index, root_id),
        },
        nodes: describe(&walk.included, resources, index),
        edges: selected,
        truncated_nodes: walk.truncated_nodes,
        truncated_edges,
        truncated_depth: walk.truncated_depth,
    }
}

fn expand<'a>(
    adjacency: &BTreeMap<String, Vec<usize>>,
    edges: &'a [EdgeView],
    frontier: &[&'a str],
    included: &BTreeSet<&'a str>,
    max_nodes: usize,
) -> (BTreeSet<&'a str>, bool) {
    let mut next: BTreeSet<&'a str> = BTreeSet::new();
    let mut truncated = false;
    for id in frontier {
        let Some(indices) = adjacency.get(*id) else {
            continue;
        };
        if scan_node(indices, edges, id, included, &mut next, max_nodes) {
            truncated = true;
            break;
        }
    }
    (next, truncated)
}

fn scan_node<'a>(
    indices: &[usize],
    edges: &'a [EdgeView],
    id: &str,
    included: &BTreeSet<&'a str>,
    next: &mut BTreeSet<&'a str>,
    max_nodes: usize,
) -> bool {
    for &index in indices {
        let Some(other) = new_neighbor(edges, id, index, included, next) else {
            continue;
        };
        if !has_room(included, next, max_nodes) {
            return true;
        }
        next.insert(other);
    }
    false
}

fn new_neighbor<'a>(
    edges: &'a [EdgeView],
    id: &str,
    index: usize,
    included: &BTreeSet<&'a str>,
    next: &BTreeSet<&'a str>,
) -> Option<&'a str> {
    let edge = edges.get(index)?;
    let other = other_endpoint(edge, id);
    (!included.contains(other) && !next.contains(other)).then_some(other)
}

fn has_room(included: &BTreeSet<&str>, next: &BTreeSet<&str>, max_nodes: usize) -> bool {
    included
        .len()
        .checked_add(next.len())
        .is_none_or(|len| len < max_nodes)
}

fn boundary_crossed<'a>(
    adjacency: &BTreeMap<String, Vec<usize>>,
    edges: &'a [EdgeView],
    frontier: &[&'a str],
    included: &BTreeSet<&'a str>,
) -> bool {
    frontier.iter().any(|id| {
        adjacency.get(*id).is_some_and(|indices| {
            indices.iter().any(|&index| {
                edges.get(index).is_some_and(|edge| {
                    let other = other_endpoint(edge, id);
                    !included.contains(other)
                })
            })
        })
    })
}

fn collect_edges<'a>(
    included: &BTreeSet<&'a str>,
    adjacency: &BTreeMap<String, Vec<usize>>,
    edges: &'a [EdgeView],
    max_edges: usize,
) -> (Vec<&'a EdgeView>, bool) {
    let limit = max_edges.saturating_add(1);
    let (mut selected, mut truncated) = scan_edges(included, adjacency, edges, limit);
    truncated |= selected.len() > max_edges;
    selected.truncate(max_edges);
    selected.sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));
    (selected, truncated)
}

fn scan_edges<'a>(
    included: &BTreeSet<&'a str>,
    adjacency: &BTreeMap<String, Vec<usize>>,
    edges: &'a [EdgeView],
    limit: usize,
) -> (Vec<&'a EdgeView>, bool) {
    let mut selected: Vec<&'a EdgeView> = Vec::new();
    let mut seen: BTreeSet<usize> = BTreeSet::new();
    for id in included {
        let Some(indices) = adjacency.get(*id) else {
            continue;
        };
        if scan_node_edges(indices, included, edges, &mut seen, &mut selected, limit) {
            return (selected, true);
        }
    }
    (selected, false)
}

fn scan_node_edges<'a>(
    indices: &[usize],
    included: &BTreeSet<&'a str>,
    edges: &'a [EdgeView],
    seen: &mut BTreeSet<usize>,
    selected: &mut Vec<&'a EdgeView>,
    limit: usize,
) -> bool {
    for &index in indices {
        if !is_eligible(included, edges, index) || !seen.insert(index) {
            continue;
        }
        let Some(edge) = edges.get(index) else {
            continue;
        };
        if selected.len() >= limit {
            return true;
        }
        selected.push(edge);
    }
    false
}

fn is_eligible(included: &BTreeSet<&str>, edges: &[EdgeView], index: usize) -> bool {
    edges.get(index).is_some_and(|edge| {
        included.contains(edge.from_id.as_str()) && included.contains(edge.to_id.as_str())
    })
}

fn describe<'a>(
    included: &BTreeSet<&'a str>,
    resources: &'a [ResourceView],
    index: &'a BTreeMap<String, usize>,
) -> Vec<GraphNode<'a>> {
    included
        .iter()
        .map(|id| GraphNode {
            id,
            resource: lookup(resources, index, id),
        })
        .collect()
}

fn lookup<'a>(
    resources: &'a [ResourceView],
    index: &BTreeMap<String, usize>,
    id: &str,
) -> Option<&'a ResourceView> {
    let position = *index.get(id)?;
    resources.get(position)
}
