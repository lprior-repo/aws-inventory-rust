use makepad_widgets::{Rect, dvec2};

use super::super::builder;
use super::super::graph::{MAX_NODES, NodeKind};
use super::super::layout::DagLayout;
use super::super::view::{MAX_SCALE, MIN_SCALE, ViewTransform};
use crate::data::{Data, Graph};

fn projected_row(account: &str, region: &str, kind: &str, id: &str, key: &str) -> crate::data::Row {
    crate::data::Row {
        account: account.to_owned(),
        region: region.to_owned(),
        kind: kind.to_owned(),
        id: id.to_owned(),
        name: id.to_owned(),
        key: key.to_owned(),
        record_bytes: Some(1),
        content_hash: Some(String::new()),
        tags: Vec::new(),
        tags_truncated: false,
        tag_observation: None,
    }
}

#[test]
fn live_neighbor_projection_uses_full_resource_keys_and_hierarchy() {
    let mut data = Data::default();
    let source_row = projected_row(
        "111111111111",
        "us-east-1",
        "AWS::EC2::VPC",
        "vpc-a",
        "key-vpc",
    );
    let target_row = projected_row(
        "111111111111",
        "us-east-1",
        "AWS::EC2::Subnet",
        "subnet-a",
        "key-subnet",
    );
    data.replace_page(vec![source_row.clone(), target_row.clone()], 2);
    data.set_graph(
        crate::data::Graph {
            nodes: vec![source_row, target_row],
            edges: vec![crate::data::Edge {
                from: "key-vpc".to_owned(),
                to: "key-subnet".to_owned(),
                label: "CONTAINS".to_owned(),
                provenance: "collector".to_owned(),
            }],
            truncated: false,
            ..crate::data::Graph::default()
        },
        None,
    );
    let graph = builder::from_data_graph(&data.visual_graph());
    assert_eq!(graph.len(), 6);
    assert_eq!(graph.edges().len(), 6);
    assert_eq!(graph.dropped_nodes(), 0);
    assert_eq!(graph.dropped_edges(), 0);
}

#[test]
fn empty_neighbor_projection_keeps_selected_row_containment() {
    let mut data = Data::default();
    data.replace_page(
        vec![projected_row(
            "111111111111",
            "us-east-1",
            "AWS::EC2::VPC",
            "vpc-a",
            "key-vpc",
        )],
        1,
    );
    let graph = builder::from_data_graph(&data.visual_graph());
    assert_eq!(graph.len(), 4);
    assert_eq!(graph.edges().len(), 3);
    assert_eq!(graph.index_of("key-vpc"), Some(3));
}

#[test]
fn fit_centres_and_zoom_keeps_the_anchor() {
    let rect = Rect {
        pos: dvec2(10.0, 20.0),
        size: dvec2(800.0, 600.0),
    };
    let fitted = ViewTransform::fit(rect, 400.0, 300.0);
    assert!(fitted.scale >= MIN_SCALE && fitted.scale <= MAX_SCALE);
    assert!(fitted.offset.x > 0.0 && fitted.offset.y > 0.0);

    let anchor = dvec2(300.0, 250.0);
    let before = fitted.to_world(rect, anchor);
    let zoomed = fitted.zoomed(2.0, anchor, rect);
    let after = zoomed.to_world(rect, anchor);
    assert!((before.0 - after.0).abs() < 0.5);
    assert!((before.1 - after.1).abs() < 0.5);

    let panned = zoomed.panned(dvec2(15.0, -10.0));
    assert!((panned.offset.x - zoomed.offset.x - 15.0).abs() < f64::EPSILON);
}

fn native_tag_graph(account: &str, region: &str) -> Graph {
    let mut root = projected_row(account, region, "AWS::EC2::VPC", "vpc-a", "root");
    root.tags.push(cmdb_view::ResourceTag {
        key: "sql-only".to_owned(),
        value: "not-native".to_owned(),
    });
    Graph {
        nodes: vec![
            root,
            projected_row(account, region, "AWS::EC2::VPC", "vpc-b", "neighbor"),
        ],
        tag_root: Some("root".to_owned()),
        tag_affiliations: vec![cmdb_view::TagAffiliation {
            tag: cmdb_view::ScopedTag {
                account: account.to_owned(),
                region: region.to_owned(),
                key: "Environment".to_owned(),
                value: "production".to_owned(),
            },
            provenance: "resource-tags".to_owned(),
        }],
        ..Graph::default()
    }
}

#[test]
fn given_native_tags_when_projecting_then_only_the_requested_resource_gets_affiliation_links()
-> Result<(), Box<dyn std::error::Error>> {
    // Given one native affiliation and two resources, with a different SQL-only tag.
    let source = native_tag_graph("111111111111", "us-east-1");
    // When the actual graph surface projection is constructed.
    let graph = builder::from_data_graph(&source);
    // Then there is one distinct tag vertex and one affiliation, never a resource clique.
    let tags: Vec<_> = graph
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| node.kind() == NodeKind::Tag)
        .collect();
    assert_eq!(tags.len(), 1);
    let (tag_index, tag) = tags.first().ok_or("native tag vertex")?;
    assert!(!tag.kind().is_resource());
    assert!(tag.label().starts_with("tag:"));
    let root = graph.index_of("root").ok_or("root vertex")?;
    let neighbor = graph.index_of("neighbor").ok_or("neighbor vertex")?;
    let links: Vec<_> = graph
        .edges()
        .iter()
        .filter(|edge| edge.to == *tag_index)
        .collect();
    assert_eq!(links.len(), 1);
    // Tag affiliation edges are overlay edges with relation "has_tag".
    assert_eq!(
        links.first().map(|edge| (edge.from, edge.overlay)),
        Some((root, true))
    );
    assert!(
        !graph
            .edges()
            .iter()
            .any(|edge| edge.from == root && edge.to == neighbor)
    );
    assert_eq!(graph.len(), 6);
    assert_eq!(graph.edges().len(), 5);
    Ok(())
}

#[test]
fn given_identical_tag_text_when_scopes_differ_then_tag_vertex_identities_do_not_alias()
-> Result<(), Box<dyn std::error::Error>> {
    // Given identical key/value text in different account or region scopes.
    let source = native_tag_graph("111111111111", "us-east-1");
    let other_account = native_tag_graph("222222222222", "us-east-1");
    let other_region = native_tag_graph("111111111111", "eu-west-1");
    // When each native affiliation is projected.
    let tag_id = |source: &Graph| -> Result<String, &'static str> {
        builder::from_data_graph(source)
            .nodes()
            .iter()
            .find(|node| node.kind() == NodeKind::Tag)
            .map(|node| node.id().to_owned())
            .ok_or("tag vertex")
    };
    // Then stable scoped identity is separate even though labels match.
    assert_ne!(tag_id(&source)?, tag_id(&other_account)?);
    assert_ne!(tag_id(&source)?, tag_id(&other_region)?);
    Ok(())
}

#[test]
fn given_missing_or_foreign_tag_root_when_projecting_then_no_affiliation_is_invented()
-> Result<(), Box<dyn std::error::Error>> {
    // Given native-looking tag data not bound to the displayed resource scope.
    let mut source = native_tag_graph("111111111111", "us-east-1");
    source
        .tag_affiliations
        .first_mut()
        .ok_or("affiliation")?
        .tag
        .account = "222222222222".to_owned();
    // When projected, foreign-scope data cannot attach to the root.
    let graph = builder::from_data_graph(&source);
    assert_eq!(
        graph
            .nodes()
            .iter()
            .filter(|node| node.kind() == NodeKind::Tag)
            .count(),
        0
    );
    // Nor can affiliations without a requested root be attached to arbitrary neighbors.
    source.tag_root = None;
    let graph = builder::from_data_graph(&source);
    assert_eq!(
        graph
            .nodes()
            .iter()
            .filter(|node| node.kind() == NodeKind::Tag)
            .count(),
        0
    );
    assert_eq!(graph.edges().len(), 4);
    Ok(())
}

#[test]
fn given_more_native_tags_than_render_capacity_when_projecting_then_vertices_stay_bounded() {
    // Given a native affiliation fanout exceeding the graph renderer's hard cap.
    let mut source = native_tag_graph("111111111111", "us-east-1");
    source.tag_affiliations = (0..MAX_NODES.saturating_add(10))
        .map(|index| cmdb_view::TagAffiliation {
            tag: cmdb_view::ScopedTag {
                account: "111111111111".to_owned(),
                region: "us-east-1".to_owned(),
                key: format!("key-{index}"),
                value: "value".to_owned(),
            },
            provenance: "resource-tags".to_owned(),
        })
        .collect();
    // When projected, resource hierarchy plus tag vertices share the same fixed budget.
    let graph = builder::from_data_graph(&source);
    // Then overflow is explicit and the layout never receives excess vertices.
    assert_eq!(graph.len(), MAX_NODES);
    assert_eq!(graph.dropped_nodes(), 15);
    assert!(graph.edges().len() <= super::super::graph::MAX_EDGES);
    assert_eq!(DagLayout::compute(&graph).node_count(), MAX_NODES);
}
