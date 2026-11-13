use super::*;

#[tokio::test]
async fn traversal_handles_cycles_and_unresolved_endpoints_deterministically() -> Result<()> {
    let store = graph_fixture().await?;
    let report = graph::report(&store, "graph", "a", None, options(Direction::Both, 4)).await?;
    assert_eq!(rows(&report, "nodes")?.len(), 4);
    assert_eq!(rows(&report, "relationships")?.len(), 4);
    assert_eq!(report["truncated"], false);
    assert_eq!(
        rows(&report, "nodes")?
            .iter()
            .map(|n| n["id"].as_str())
            .collect::<Vec<_>>(),
        vec![Some("a"), Some("b"), Some("c"), Some("missing")]
    );
    let missing = rows(&report, "nodes")?
        .iter()
        .find(|n| n["id"] == "missing")
        .context("missing placeholder")?;
    assert_eq!(missing["observed"], false);
    assert_eq!(missing["kind"], Value::Null);
    let again = graph::report(&store, "graph", "a", None, options(Direction::Both, 4)).await?;
    assert_eq!(report, again);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn traversal_deduplicates_observations_and_preserves_source_precedence() -> Result<()> {
    let store = Store::open(":memory:").await?;
    store.begin_scan("overlap", 1).await?;
    for (source, collector, kind, observed_at) in [
        (Source::Direct, "native", "direct-kind", 10),
        (Source::CloudControl, "broad", "broad-kind", 20),
        (Source::Direct, "native-again", "direct-kind", 10),
    ] {
        let mut node = resource("a", kind, json!({}));
        node.source = source;
        node.observed_at = observed_at;
        let mut relationship = edge("a", "missing", "REFERS_TO");
        relationship.source = source;
        relationship.observed_at = observed_at;
        store
            .apply_page(
                "overlap",
                CollectionPage {
                    scope: scope(source, collector),
                    page_number: 1,
                    next_token: None,
                    resources: vec![node],
                    edges: vec![relationship],
                },
            )
            .await?;
    }
    store
        .finish_scan("overlap", ScanStatus::Partial, 30)
        .await?;
    let report = graph::report(&store, "overlap", "a", None, options(Direction::Both, 4)).await?;
    assert_eq!(rows(&report, "nodes")?.len(), 2);
    assert_eq!(rows(&report, "relationships")?.len(), 1);
    assert_eq!(first(&report["relationships"])["source"], "direct");
    assert_eq!(first(&report["relationships"])["from_kind"], "direct-kind");
    assert_eq!(first(&report["relationships"])["observed_at"], 10);
    assert_eq!(report["status"], "partial");
    assert_eq!(report["truncated"], false);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn traversal_directions_change_distances_and_path_orientation() -> Result<()> {
    let store = graph_fixture().await?;
    let outgoing = graph::report(
        &store,
        "graph",
        "a",
        Some("c"),
        options(Direction::Outgoing, 4),
    )
    .await?;
    assert_eq!(outgoing["path"]["hops"], 2);
    assert_eq!(outgoing["path"]["nodes"], json!(["a", "b", "c"]));
    let incoming = graph::report(
        &store,
        "graph",
        "a",
        Some("c"),
        options(Direction::Incoming, 4),
    )
    .await?;
    assert_eq!(incoming["path"]["hops"], 1);
    assert_eq!(first(&incoming["path"]["steps"])["reversed"], true);
    assert_eq!(incoming["path"]["effective_reachability_inferred"], false);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn path_uses_shortest_hops_with_deterministic_tie_breaking() -> Result<()> {
    let store = Store::open(":memory:").await?;
    snapshot(
        &store,
        "graph",
        vec![
            resource("a", "fixture", json!({})),
            resource("b", "fixture", json!({})),
            resource("c", "fixture", json!({})),
            resource("d", "fixture", json!({})),
        ],
        vec![
            edge("a", "b", "AB"),
            edge("a", "c", "AC"),
            edge("b", "d", "BD"),
            edge("c", "d", "CD"),
        ],
        ScanStatus::Complete,
    )
    .await?;
    let result = graph::report(
        &store,
        "graph",
        "a",
        Some("d"),
        options(Direction::Outgoing, 4),
    )
    .await?;
    assert_eq!(result["path"]["nodes"], json!(["a", "b", "d"]));
    assert_eq!(result["path"]["shortest_for_selected_direction"], true);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn node_and_edge_caps_report_partial_work_without_dangling_graph_rows() -> Result<()> {
    let store = graph_fixture().await?;
    let node_limited = graph::report(
        &store,
        "graph",
        "a",
        Some("missing"),
        TraversalOptions {
            max_nodes: 2,
            ..options(Direction::Both, 4)
        },
    )
    .await?;
    assert_eq!(node_limited["truncation"]["node_limit"], true);
    assert_eq!(node_limited["status"], "partial");
    assert_eq!(node_limited["path"]["result"], "not_found_within_limits");
    assert_eq!(rows(&node_limited, "nodes")?.len(), 2);
    assert_eq!(rows(&node_limited, "relationships")?.len(), 1);
    let edge_limited = graph::report(
        &store,
        "graph",
        "a",
        None,
        TraversalOptions {
            max_edges: 1,
            ..options(Direction::Both, 4)
        },
    )
    .await?;
    assert_eq!(edge_limited["truncation"]["edge_limit"], true);
    assert_eq!(rows(&edge_limited, "relationships")?.len(), 1);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn exact_edge_budget_does_not_report_false_truncation_on_reverse_duplicates() -> Result<()> {
    let store = Store::open(":memory:").await?;
    snapshot(
        &store,
        "graph",
        vec![
            resource("a", "fixture", json!({})),
            resource("b", "fixture", json!({})),
        ],
        vec![edge("a", "b", "LINK")],
        ScanStatus::Complete,
    )
    .await?;
    let result = graph::report(
        &store,
        "graph",
        "a",
        None,
        TraversalOptions {
            max_edges: 1,
            ..options(Direction::Both, 4)
        },
    )
    .await?;
    assert_eq!(result["truncated"], false);
    assert_eq!(result["inspected_edges"], 2);
    assert_eq!(rows(&result, "relationships")?.len(), 1);
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn zero_depth_and_zero_hop_paths_have_explicit_radius_semantics() -> Result<()> {
    let store = graph_fixture().await?;
    let result = graph::report(&store, "graph", "a", None, options(Direction::Both, 0)).await?;
    assert_eq!(rows(&result, "nodes")?.len(), 1);
    assert_eq!(rows(&result, "relationships")?.len(), 0);
    assert_eq!(result["depth_boundary_nodes"], 1);
    let same = graph::report(&store, "graph", "a", Some("a"), options(Direction::Both, 0)).await?;
    assert_eq!(same["path"]["hops"], 0);
    assert_eq!(same["path"]["found"], true);
    let outside = graph::report(
        &store,
        "graph",
        "a",
        Some("c"),
        options(Direction::Outgoing, 1),
    )
    .await?;
    assert_eq!(outside["path"]["found"], false);
    assert_eq!(outside["path"]["result"], "not_found_within_limits");
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn graph_keeps_endpoint_metadata_inside_the_selected_scan() -> Result<()> {
    let store = Store::open(":memory:").await?;
    snapshot(
        &store,
        "old",
        vec![resource("a", "fixture", json!({}))],
        vec![edge("a", "b", "LINK")],
        ScanStatus::Complete,
    )
    .await?;
    snapshot(
        &store,
        "new",
        vec![resource("b", "later-kind", json!({}))],
        vec![],
        ScanStatus::Complete,
    )
    .await?;
    let result = graph::report(&store, "old", "a", None, options(Direction::Both, 4)).await?;
    let target = rows(&result, "nodes")?
        .iter()
        .find(|n| n["id"] == "b")
        .context("target")?;
    assert_eq!(target["observed"], false);
    assert_eq!(target["kind"], Value::Null);
    assert!(
        graph::report(&store, "new", "a", None, options(Direction::Both, 4))
            .await
            .is_err()
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn references_can_be_start_nodes_but_unknown_identifiers_cannot() -> Result<()> {
    let store = graph_fixture().await?;
    let result = graph::report(
        &store,
        "graph",
        "missing",
        Some("a"),
        options(Direction::Incoming, 4),
    )
    .await?;
    assert_eq!(result["path"]["found"], true);
    assert_eq!(result["path"]["nodes"], json!(["missing", "b", "a"]));
    assert!(
        graph::report(
            &store,
            "graph",
            "unknown",
            None,
            options(Direction::Both, 4)
        )
        .await
        .is_err()
    );
    store.close().await;
    Ok(())
}
