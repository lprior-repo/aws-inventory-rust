use crate::data::{Graph as DataGraph, Row};

use super::graph::{DagGraph, NodeKind, truncate_label};

pub fn from_data_graph(source: &DataGraph) -> DagGraph {
    let mut dag = DagGraph::default();
    let mut rows: Vec<&Row> = source.nodes.iter().collect();
    rows.sort_by(|left, right| {
        (&left.account, &left.region, &left.kind, &left.id).cmp(&(
            &right.account,
            &right.region,
            &right.kind,
            &right.id,
        ))
    });
    for row in rows {
        add_row(&mut dag, row);
    }
    add_resource_edges(&mut dag, source);
    add_tag_affiliations(&mut dag, source);
    dag
}

fn add_row(dag: &mut DagGraph, row: &Row) {
    let Some((_, _, kind)) = add_containment(dag, row) else {
        return;
    };
    let label = truncate_label(&row.name);
    let Some(resource) = dag.add_node(NodeKind::from_facet(&row.kind), &row.key, label) else {
        return;
    };
    add_containment_edge(dag, kind, resource);
}

fn add_containment(dag: &mut DagGraph, row: &Row) -> Option<(usize, usize, usize)> {
    let account = dag.add_node(
        NodeKind::Account,
        format!("account:{}", row.account),
        row.account.clone(),
    )?;
    let region = dag.add_node(
        NodeKind::Region,
        format!("region:{}/{}", row.account, row.region),
        row.region.clone(),
    )?;
    let kind = dag.add_node(
        NodeKind::Kind,
        format!("kind:{}/{}/{}", row.account, row.region, row.kind),
        row.kind.clone(),
    )?;
    add_containment_edge(dag, account, region);
    add_containment_edge(dag, region, kind);
    Some((account, region, kind))
}

fn add_resource_edges(dag: &mut DagGraph, source: &DataGraph) {
    for edge in &source.edges {
        let from = dag.index_of(&edge.from);
        let to = dag.index_of(&edge.to);
        if let (Some(from), Some(to)) = (from, to) {
            dag.add_overlay_edge(from, to, edge.label.clone(), edge.provenance.clone());
        }
    }
}

fn add_containment_edge(dag: &mut DagGraph, from: usize, to: usize) {
    dag.add_hierarchy_edge(from, to, "contains".to_owned());
}

fn add_tag_affiliations(dag: &mut DagGraph, source: &DataGraph) {
    let Some(root) = source
        .tag_root
        .as_deref()
        .and_then(|key| source.nodes.iter().find(|row| row.key == key))
    else {
        return;
    };
    let Some(resource) = dag.index_of(&root.key) else {
        return;
    };
    source
        .tag_affiliations
        .iter()
        .filter(|entry| entry.tag.account == root.account && entry.tag.region == root.region)
        .for_each(|entry| {
            let tag = &entry.tag;
            let label = truncate_label(&format!("tag: {:?}={:?}", tag.key, tag.value));
            if let Some(node) = dag.add_node(NodeKind::Tag, tag_id(tag), label) {
                dag.add_overlay_edge(resource, node, "has_tag".to_owned(), String::new());
            }
        });
}

fn tag_id(tag: &cmdb_view::ScopedTag) -> String {
    format!(
        "tag:{}:{}:{}:{}:{}:{}:{}:{}",
        tag.account.len(),
        tag.account,
        tag.region.len(),
        tag.region,
        tag.key.len(),
        tag.key,
        tag.value.len(),
        tag.value,
    )
}
