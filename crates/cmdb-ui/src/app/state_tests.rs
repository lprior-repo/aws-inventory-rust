use super::{apply_graph_data, apply_snapshot_data};
use crate::data::{Counts, Data, Edge, Graph, Row};

pub(super) fn row(id: &str) -> Row {
    Row {
        account: "111111111111".to_owned(),
        region: "us-east-1".to_owned(),
        kind: "AWS::EC2::VPC".to_owned(),
        id: id.to_owned(),
        name: id.to_owned(),
        key: format!("key-{id}"),
        record_bytes: Some(8),
        content_hash: Some("stored-hash".to_owned()),
        tags: vec![cmdb_view::ResourceTag {
            key: "Environment".to_owned(),
            value: id.to_owned(),
        }],
        tags_truncated: false,
        tag_observation: None,
    }
}

pub(super) fn graph(root: &Row) -> Graph {
    Graph {
        nodes: vec![root.clone()],
        edges: vec![Edge {
            from: root.key.clone(),
            to: "neighbor".to_owned(),
            label: "CONTAINS".to_owned(),
            provenance: "collector".to_owned(),
        }],
        truncated: true,
        tag_root: Some(root.key.clone()),
        tag_affiliations: vec![cmdb_view::TagAffiliation {
            tag: cmdb_view::ScopedTag {
                account: root.account.clone(),
                region: root.region.clone(),
                key: "Environment".to_owned(),
                value: root.id.clone(),
            },
            provenance: "resource-tags".to_owned(),
        }],
        tags_truncated: true,
    }
}

#[test]
fn given_failed_truncated_graph_when_inventory_recovers_then_details_and_graph_state_are_current() {
    let mut data = Data::default();
    let old = row("old");
    data.replace_page(vec![old.clone()], 1);
    data.set_graph(graph(&old), Some("connection refused".to_owned()));
    data.set_loading(true);
    data.set_error(Some("old database failure".to_owned()));
    data.set_tag(Some(cmdb_view::TagFilter {
        key: "Environment".to_owned(),
        value: None,
    }));

    apply_snapshot_data(
        &mut data,
        Counts {
            resources: 9,
            ..Counts::default()
        },
        vec![row("new")],
        3,
        None,
    );

    assert_eq!(data.visible_len(), 1);
    assert_eq!(data.selected_row().map(|row| row.id.as_str()), Some("new"));
    assert_eq!(data.status_line(), "3 matching · 1 loaded");
    assert_eq!(data.total_len(), 9);
    assert!(!data.graph_status().contains("offline"));
    assert!(!data.graph_status().contains("truncated"));
    assert!(data.visual_graph().edges.is_empty());
    assert!(data.visual_graph().tag_affiliations.is_empty());
    assert_eq!(
        data.filter().tag.as_ref().map(|tag| tag.key.as_str()),
        Some("Environment")
    );
}

#[test]
fn given_visible_inventory_when_database_fails_then_old_rows_are_not_presented_as_current() {
    let mut data = Data::default();
    let old = row("old");
    apply_snapshot_data(
        &mut data,
        Counts {
            resources: 1,
            ..Counts::default()
        },
        vec![old.clone()],
        1,
        None,
    );
    data.set_graph(graph(&old), None);

    apply_snapshot_data(
        &mut data,
        Counts {
            resources: 99,
            ..Counts::default()
        },
        vec![row("partial")],
        99,
        Some("schema unavailable".to_owned()),
    );

    assert_eq!(data.visible_len(), 0);
    assert!(data.selected_row().is_none());
    assert_eq!(data.total_len(), 0);
    assert_eq!(data.status_line(), "Database error · schema unavailable");
    assert!(data.visual_graph().nodes.is_empty());
    assert!(data.visual_graph().edges.is_empty());

    apply_snapshot_data(
        &mut data,
        Counts {
            resources: 1,
            ..Counts::default()
        },
        vec![row("recovered")],
        1,
        None,
    );
    assert_eq!(data.status_line(), "1 matching · 1 loaded");
    assert_eq!(
        data.selected_row().map(|row| row.id.as_str()),
        Some("recovered")
    );
}

#[test]
fn given_partial_graph_failure_when_consumed_then_no_dependency_or_tag_fallback_is_rendered() {
    let mut data = Data::default();
    let selected = row("current");
    data.replace_page(vec![selected.clone()], 1);

    assert!(apply_graph_data(
        &mut data,
        graph(&selected),
        Some("detail read timed out".to_owned())
    ));

    let visible = data.visual_graph();
    assert_eq!(visible.nodes, vec![selected.clone()]);
    assert!(visible.edges.is_empty());
    assert!(visible.tag_affiliations.is_empty());
    assert!(!visible.truncated && !visible.tags_truncated);
    assert!(data.graph_status().contains("detail read timed out"));

    let recovered = Graph {
        nodes: vec![selected.clone()],
        tag_root: Some(selected.key),
        ..Graph::default()
    };
    assert!(apply_graph_data(&mut data, recovered, None));
    assert!(!data.graph_status().contains("timed out"));
}

#[test]
fn given_newer_selection_when_an_old_root_arrives_then_current_rows_and_graph_are_unchanged() {
    let mut data = Data::default();
    let current = row("current");
    data.replace_page(vec![current.clone()], 1);
    assert!(apply_graph_data(&mut data, graph(&current), None));
    let accepted = data.visual_graph();

    assert!(!apply_graph_data(&mut data, graph(&row("old")), None));

    assert_eq!(data.visual_graph(), accepted);
    assert_eq!(data.selected_row(), Some(&current));
    assert_eq!(data.visible_row(0), Some(&current));
}

#[test]
fn given_cleared_selection_when_old_graph_completes_then_the_graph_stays_empty() {
    let mut data = Data::default();
    let selected = row("old");
    data.replace_page(vec![selected.clone()], 1);
    assert!(data.clear_selection());

    assert!(!apply_graph_data(&mut data, graph(&selected), None));

    assert!(data.selected_row().is_none());
    assert!(data.visual_graph().nodes.is_empty());
    assert!(data.visual_graph().tag_affiliations.is_empty());
}
