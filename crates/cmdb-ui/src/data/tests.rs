use super::*;

fn row(account: &str, kind: &str, id: &str) -> Row {
    Row {
        account: account.to_owned(),
        region: "us-east-1".to_owned(),
        kind: kind.to_owned(),
        id: id.to_owned(),
        name: id.to_owned(),
        key: format!("{account}:{kind}:{id}"),
        record_bytes: Some(8),
        content_hash: Some("stored-hash".to_owned()),
    }
}

#[test]
fn page_replacement_selects_first_and_preserves_visible_selection() {
    let mut data = Data::default();
    data.replace_page(
        vec![
            row("111111111111", "AWS::EC2::VPC", "vpc-a"),
            row("111111111111", "AWS::EC2::Subnet", "subnet-a"),
        ],
        2,
    );
    assert_eq!(
        data.selected_row().map(|value| value.id.as_str()),
        Some("vpc-a")
    );
    assert!(data.select_visible(1));
    data.replace_page(
        vec![
            row("111111111111", "AWS::EC2::VPC", "vpc-a"),
            row("111111111111", "AWS::EC2::Subnet", "subnet-a"),
        ],
        2,
    );
    assert_eq!(
        data.selected_row().map(|value| value.id.as_str()),
        Some("subnet-a")
    );
}

#[test]
fn page_replacement_drops_selection_that_disappears() {
    let mut data = Data::default();
    data.replace_page(vec![row("111111111111", "AWS::EC2::VPC", "vpc-a")], 1);
    assert_eq!(
        data.selected_row().map(|value| value.id.as_str()),
        Some("vpc-a")
    );
    data.replace_page(vec![row("222222222222", "AWS::EC2::VPC", "vpc-b")], 1);
    assert_eq!(
        data.selected_row().map(|value| value.account.as_str()),
        Some("222222222222")
    );
}

#[test]
fn filters_are_owned_and_query_state_is_normalized() {
    let mut data = Data::default();
    let account = String::from("111111111111");
    data.set_account(Some(&account));
    data.set_region(Some("us-east-1"));
    data.set_kind(Some("AWS::EC2::VPC"));
    data.set_query("  VPC-Alpha  ");
    assert_eq!(data.filter().account.as_deref(), Some("111111111111"));
    assert_eq!(data.filter().region.as_deref(), Some("us-east-1"));
    assert_eq!(data.filter().kind.as_deref(), Some("AWS::EC2::VPC"));
    assert_eq!(data.filter().query, "VPC-Alpha");
}

#[test]
fn empty_database_status_is_distinct_from_loading_and_error() {
    let mut data = Data::default();
    data.set_loading(true);
    assert_eq!(data.status_line(), "Loading inventory…");
    data.set_loading(false);
    data.set_counts(Counts::default());
    assert_eq!(data.status_line(), "Database opened · no resources yet");
    data.set_error(Some("schema version 13".to_owned()));
    assert!(data.status_line().contains("schema version 13"));
}

#[test]
fn graph_state_reports_actual_edges_and_offline_error_separately() {
    let mut data = Data::default();
    data.set_graph(Graph::default(), Some("connection refused".to_owned()));
    assert!(data.graph_status().contains("Graph offline"));
    data.set_graph(
        Graph {
            nodes: vec![
                row("111111111111", "AWS::EC2::VPC", "vpc-a"),
                row("111111111111", "AWS::EC2::Subnet", "subnet-a"),
            ],
            edges: vec![Edge {
                from: "vpc-key".to_owned(),
                to: "subnet-key".to_owned(),
                label: "CONTAINS".to_owned(),
                provenance: "collector".to_owned(),
            }],
            truncated: false,
        },
        None,
    );
    assert_eq!(
        data.graph_status(),
        "2 nodes · 1 relationships · provenance: collector"
    );
    let mut truncated = data.visual_graph();
    truncated.truncated = true;
    data.set_graph(truncated, None);
    assert!(data.graph_status().contains("neighborhood truncated"));
}

#[test]
fn graph_neighbor_can_become_the_selected_resource_without_fabricated_sql_metadata() {
    let mut data = Data::default();
    data.replace_page(vec![row("111111111111", "AWS::EC2::VPC", "vpc-a")], 1);
    let neighbor = Row {
        account: "111111111111".to_owned(),
        region: "us-east-1".to_owned(),
        kind: "AWS::EC2::Subnet".to_owned(),
        id: "subnet-a".to_owned(),
        name: "subnet-a".to_owned(),
        key: "neighbor-key".to_owned(),
        record_bytes: None,
        content_hash: None,
    };
    data.set_graph(
        Graph {
            nodes: vec![neighbor],
            edges: Vec::new(),
            truncated: false,
        },
        None,
    );
    assert!(data.select_key("neighbor-key"));
    let selected = data.selected_row();
    assert_eq!(selected.map(|value| value.id.as_str()), Some("subnet-a"));
    assert_eq!(selected.and_then(|value| value.record_bytes), None);
}
