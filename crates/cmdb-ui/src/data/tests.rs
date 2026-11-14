use super::*;

#[test]
fn fixture_spans_three_accounts_two_regions_four_kinds() {
    let data = Data::synthetic();
    assert_eq!(data.total_len(), 24);
    for account in ACCOUNTS {
        assert!(data.account_count(account) > 0, "missing {account}");
    }
    for region in REGIONS {
        assert!(data.region_count(region) > 0, "missing {region}");
    }
    for kind in KINDS {
        assert!(data.kind_count(kind) > 0, "missing {kind}");
    }
}

#[test]
fn selectors_narrow_the_visible_set() {
    let mut data = Data::synthetic();
    data.set_account(Some("prod"));
    assert_eq!(data.visible_len(), data.account_count("prod"));
    data.set_kind(Some("s3"));
    assert!(
        data.visible_row(0)
            .is_some_and(|row| row.account == "prod" && row.kind == "s3")
    );
    data.set_query("backup");
    assert_eq!(data.visible_len(), 1);
    assert!(data.filter_label().contains("backup"));
}

#[test]
fn graph_scopes_nodes_and_edges_to_the_filter() {
    let mut data = Data::synthetic();
    let all = data.graph();
    assert_eq!(all.nodes.len(), data.total_len());
    assert_eq!(all.edges.len(), data.edge_count());
    assert!(!all.edges.is_empty());

    data.set_account(Some("sandbox"));
    let filtered = data.graph();
    assert_eq!(filtered.nodes.len(), data.visible_len());
    assert!(!filtered.edges.is_empty());
    for edge in &filtered.edges {
        assert!(filtered.nodes.iter().any(|node| node.id == edge.from));
        assert!(filtered.nodes.iter().any(|node| node.id == edge.to));
    }
}

#[test]
fn selection_follows_the_visible_set() {
    let mut data = Data::synthetic();
    assert_eq!(data.selected_row().map(|row| row.account), Some("prod"));
    assert!(data.select_visible(3));
    data.set_account(Some("sandbox"));
    assert_eq!(data.selected_row().map(|row| row.account), Some("sandbox"));
}

#[test]
fn selection_can_be_addressed_by_id() {
    let mut data = Data::synthetic();
    assert_eq!(data.visible_index_of("db-prod-orders"), Some(3));
    assert!(data.clear_selection());
    assert!(data.selected_row().is_none());
    assert!(!data.clear_selection());
}
