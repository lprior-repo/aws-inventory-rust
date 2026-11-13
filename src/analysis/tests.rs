use super::{Direction, TraversalOptions, diff, findings, graph};
use anyhow::{Context, Result};
use inventory_core::{
    CollectionFailure, CollectionPage, CollectionScope, Edge, Resource, ScanStatus, Source,
};
use inventory_engine::store::Store;
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn resource(id: &str, kind: &str, properties: Value) -> Resource {
    Resource {
        id: id.into(),
        account_id: "111111111111".into(),
        region: "us-east-1".into(),
        kind: kind.into(),
        name: Some(id.into()),
        properties,
        tags: BTreeMap::new(),
        source: Source::Direct,
        observed_at: 10,
    }
}

fn edge(from: &str, to: &str, relation: &str) -> Edge {
    Edge {
        from_id: from.into(),
        to_id: to.into(),
        relation: relation.into(),
        source: Source::Direct,
        observed_at: 10,
        properties: json!({}),
    }
}

fn scope(source: Source, name: &str) -> CollectionScope {
    CollectionScope {
        account_id: "111111111111".into(),
        region: "us-east-1".into(),
        collector: name.into(),
        parent_id: None,
        source,
    }
}

async fn snapshot(
    store: &Store,
    scan: &str,
    resources: Vec<Resource>,
    edges: Vec<Edge>,
    status: ScanStatus,
) -> Result<()> {
    store.begin_scan(scan, 1).await?;
    let source = resources
        .first()
        .map(|r| r.source)
        .unwrap_or(Source::Direct);
    store
        .apply_page(
            scan,
            CollectionPage {
                scope: scope(source, "fixture"),
                page_number: 1,
                next_token: None,
                resources,
                edges,
            },
        )
        .await?;
    store.finish_scan(scan, status, 20).await?;
    Ok(())
}

fn options(direction: Direction, max_depth: usize) -> TraversalOptions {
    TraversalOptions {
        direction,
        max_depth,
        max_nodes: 100,
        max_edges: 100,
    }
}

fn rows<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .with_context(|| format!("missing {key} array"))
}

#[tokio::test]
async fn diff_ignores_observation_time_and_object_key_order() -> Result<()> {
    let before = Store::open(":memory:").await?;
    let after = Store::open(":memory:").await?;
    let first = resource(
        "a",
        "fixture",
        serde_json::from_str("{\"a\":1,\"b\":[2,3]}")?,
    );
    let mut second = resource(
        "a",
        "fixture",
        serde_json::from_str("{\"b\":[2,3],\"a\":1}")?,
    );
    second.observed_at = 11;
    let old_edge = edge("a", "b", "REFERS_TO");
    let mut new_edge = old_edge.clone();
    new_edge.observed_at = 12;
    snapshot(
        &before,
        "before",
        vec![first],
        vec![old_edge],
        ScanStatus::Complete,
    )
    .await?;
    snapshot(
        &after,
        "after",
        vec![second],
        vec![new_edge],
        ScanStatus::Complete,
    )
    .await?;
    let result = diff::report(&before, &after, "before", "after", 100, 100).await?;
    assert_eq!(result["resources"]["counts"]["unchanged"], 1);
    assert_eq!(result["resources"]["counts"]["changed"], 0);
    assert_eq!(result["relationships"]["counts"]["unchanged"], 1);
    assert_eq!(result["resources"]["rows"], json!([]));
    before.close().await;
    after.close().await;
    Ok(())
}

#[tokio::test]
async fn diff_reports_tags_arrays_and_provider_timestamps_as_data() -> Result<()> {
    let before = Store::open(":memory:").await?;
    let after = Store::open(":memory:").await?;
    let first = resource("a", "fixture", json!({"items":[1,2],"updated_at":1}));
    let mut second = resource("a", "fixture", json!({"items":[2,1],"updated_at":2}));
    second.tags.insert("owner".into(), "synthetic".into());
    snapshot(&before, "before", vec![first], vec![], ScanStatus::Complete).await?;
    snapshot(&after, "after", vec![second], vec![], ScanStatus::Complete).await?;
    let result = diff::report(&before, &after, "before", "after", 100, 100).await?;
    assert_eq!(result["resources"]["counts"]["changed"], 1);
    assert_eq!(
        result["resources"]["rows"][0]["changed_fields"],
        json!(["properties", "tags"])
    );
    before.close().await;
    after.close().await;
    Ok(())
}

#[tokio::test]
async fn diff_absence_is_not_deletion_even_for_complete_scans() -> Result<()> {
    let before = Store::open(":memory:").await?;
    let after = Store::open(":memory:").await?;
    snapshot(
        &before,
        "before",
        vec![resource("a", "fixture", json!({}))],
        vec![edge("a", "missing", "REFERS_TO")],
        ScanStatus::Complete,
    )
    .await?;
    snapshot(
        &after,
        "after",
        vec![resource("b", "fixture", json!({}))],
        vec![],
        ScanStatus::Complete,
    )
    .await?;
    let result = diff::report(&before, &after, "before", "after", 100, 100).await?;
    assert_eq!(result["resources"]["counts"]["newly_observed"], 1);
    assert_eq!(result["resources"]["counts"]["no_longer_observed"], 1);
    assert_eq!(result["relationships"]["counts"]["no_longer_observed"], 1);
    assert_eq!(
        result["resources"]["rows"][0]["change"],
        "no_longer_observed"
    );
    assert_eq!(result["resources"]["rows"][0]["deletion_inferred"], false);
    assert_eq!(result["resources"]["rows"][1]["after"]["id"], "b");
    before.close().await;
    after.close().await;
    Ok(())
}

#[tokio::test]
async fn diff_keeps_partial_inputs_and_source_changes_explicit() -> Result<()> {
    let before = Store::open(":memory:").await?;
    let after = Store::open(":memory:").await?;
    let first = resource("a", "fixture", json!({"x":1}));
    let mut second = first.clone();
    second.source = Source::CloudControl;
    snapshot(&before, "before", vec![first], vec![], ScanStatus::Complete).await?;
    snapshot(&after, "after", vec![second], vec![], ScanStatus::Partial).await?;
    let result = diff::report(&before, &after, "before", "after", 100, 100).await?;
    assert_eq!(result["status"], "partial");
    assert_eq!(result["after_scan"]["status"], "partial");
    assert_eq!(result["resources"]["counts"]["source_changed"], 1);
    assert_eq!(
        result["resources"]["rows"][0]["changed_fields"],
        json!(["source"])
    );
    before.close().await;
    after.close().await;
    Ok(())
}

#[tokio::test]
async fn diff_detail_cap_does_not_clip_comparison_counts() -> Result<()> {
    let before = Store::open(":memory:").await?;
    let after = Store::open(":memory:").await?;
    snapshot(&before, "before", vec![], vec![], ScanStatus::Complete).await?;
    snapshot(
        &after,
        "after",
        vec![
            resource("a", "fixture", json!({})),
            resource("b", "fixture", json!({})),
        ],
        vec![],
        ScanStatus::Complete,
    )
    .await?;
    let result = diff::report(&before, &after, "before", "after", 1, 100).await?;
    assert_eq!(result["resources"]["counts"]["newly_observed"], 2);
    assert_eq!(result["resources"]["details_truncated"], true);
    assert_eq!(result["resources"]["comparison_truncated"], false);
    assert_eq!(
        result["resources"]["rows"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(result["status"], "complete");
    before.close().await;
    after.close().await;
    Ok(())
}

#[tokio::test]
async fn diff_comparison_cap_has_real_probe_and_partial_status() -> Result<()> {
    let before = Store::open(":memory:").await?;
    let after = Store::open(":memory:").await?;
    snapshot(&before, "before", vec![], vec![], ScanStatus::Complete).await?;
    snapshot(
        &after,
        "after",
        vec![
            resource("a", "fixture", json!({})),
            resource("b", "fixture", json!({})),
        ],
        vec![],
        ScanStatus::Complete,
    )
    .await?;
    let limited = diff::report(&before, &after, "before", "after", 100, 1).await?;
    assert_eq!(limited["resources"]["counts"]["examined"], 1);
    assert_eq!(limited["resources"]["comparison_truncated"], true);
    assert_eq!(limited["status"], "partial");
    let exact = diff::report(&before, &after, "before", "after", 100, 2).await?;
    assert_eq!(exact["resources"]["comparison_truncated"], false);
    before.close().await;
    after.close().await;
    Ok(())
}

#[tokio::test]
async fn diff_rejects_running_snapshots_and_invalid_limits() -> Result<()> {
    let before = Store::open(":memory:").await?;
    let after = Store::open(":memory:").await?;
    before.begin_scan("before", 1).await?;
    snapshot(&after, "after", vec![], vec![], ScanStatus::Complete).await?;
    assert!(
        diff::report(&before, &after, "before", "after", 100, 100)
            .await
            .is_err()
    );
    assert!(
        diff::report(&before, &after, "before", "after", 0, 100)
            .await
            .is_err()
    );
    assert!(
        diff::report(&before, &after, "before", "after", 100, 0)
            .await
            .is_err()
    );
    assert!(
        diff::report(&before, &after, "before", "after", 100, 1_000_001)
            .await
            .is_err()
    );
    before.close().await;
    after.close().await;
    Ok(())
}

#[tokio::test]
async fn public_diff_uses_two_readers_without_one_pool_deadlock() -> Result<()> {
    let path = std::env::temp_dir().join(format!(
        "inventory-analysis-{}.sqlite",
        uuid::Uuid::new_v4()
    ));
    let store = Store::open(&path).await?;
    snapshot(
        &store,
        "before",
        vec![resource("a", "fixture", json!({"x":1}))],
        vec![],
        ScanStatus::Complete,
    )
    .await?;
    snapshot(
        &store,
        "after",
        vec![resource("a", "fixture", json!({"x":2}))],
        vec![],
        ScanStatus::Complete,
    )
    .await?;
    store.close().await;
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        super::diff(path.clone(), "before".into(), "after".into(), 100, 100),
    )
    .await??;
    assert_eq!(result["resources"]["counts"]["changed"], 1);
    std::fs::remove_file(&path)?;
    Ok(())
}

async fn graph_fixture() -> Result<Store> {
    let store = Store::open(":memory:").await?;
    snapshot(
        &store,
        "graph",
        vec![
            resource("a", "fixture", json!({})),
            resource("b", "fixture", json!({})),
            resource("c", "fixture", json!({})),
        ],
        vec![
            edge("a", "b", "AB"),
            edge("b", "c", "BC"),
            edge("c", "a", "CA"),
            edge("b", "missing", "REFERS_TO"),
        ],
        ScanStatus::Complete,
    )
    .await?;
    Ok(store)
}

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
    assert_eq!(report["relationships"][0]["source"], "direct");
    assert_eq!(report["relationships"][0]["from_kind"], "direct-kind");
    assert_eq!(report["relationships"][0]["observed_at"], 10);
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
    assert_eq!(incoming["path"]["steps"][0]["reversed"], true);
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

#[tokio::test]
async fn graph_rejects_bad_caps_running_scans_and_handles_quoted_ids() -> Result<()> {
    let store = Store::open(":memory:").await?;
    snapshot(
        &store,
        "quoted'scan",
        vec![resource("a' OR 1=1 --", "fixture", json!({}))],
        vec![],
        ScanStatus::Partial,
    )
    .await?;
    let result = graph::report(
        &store,
        "quoted'scan",
        "a' OR 1=1 --",
        None,
        options(Direction::Both, 1),
    )
    .await?;
    assert_eq!(result["status"], "partial");
    assert_eq!(rows(&result, "nodes")?.len(), 1);
    assert!(
        graph::report(
            &store,
            "quoted'scan",
            "a",
            None,
            options(Direction::Both, 17)
        )
        .await
        .is_err()
    );
    assert!(
        graph::report(
            &store,
            "quoted'scan",
            "a",
            None,
            TraversalOptions {
                max_nodes: 0,
                ..options(Direction::Both, 1)
            }
        )
        .await
        .is_err()
    );
    assert!(
        graph::report(
            &store,
            "quoted'scan",
            "a",
            None,
            TraversalOptions {
                max_edges: 4001,
                ..options(Direction::Both, 1)
            }
        )
        .await
        .is_err()
    );
    store.begin_scan("running", 1).await?;
    assert!(
        graph::report(&store, "running", "a", None, options(Direction::Both, 1))
            .await
            .is_err()
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn findings_use_declared_facts_and_do_not_treat_missing_reads_as_false() -> Result<()> {
    let store = Store::open(":memory:").await?;
    let resources = vec![
        resource(
            "lb",
            "aws_elbv2_load_balancer",
            json!({"scheme":"internet-facing"}),
        ),
        resource(
            "listener",
            "aws_elbv2_listener",
            json!({"load_balancer_arn":"lb","port":80,"protocol":"HTTP","default_actions":[{"type":"redirect"}]}),
        ),
        resource(
            "routes",
            "aws_vpc_route_table",
            json!({"route_table_id":"rtb-fixture","routes":[
            {"destination_cidr_block":"0.0.0.0/0","gateway_id":"igw-fixture","state":"active"},
            {"destination_ipv6_cidr_block":"::/0","gateway_id":"igw-fixture","state":"blackhole"},
            {"destination_cidr_block":"0.0.0.0/0","nat_gateway_id":"nat-fixture","state":"active"}]}),
        ),
        resource(
            "public",
            "aws_s3_bucket",
            json!({"policy_status":{"configured":true,"is_public":true},"public_access_block":{"configured":true,"block_public_policy":true}}),
        ),
        resource(
            "unknown",
            "aws_s3_bucket",
            json!({"policy_status":null,"unavailable_sections":["policy_status"]}),
        ),
        resource(
            "absent",
            "aws_s3_bucket",
            json!({"policy_status":{"configured":false}}),
        ),
        resource(
            "inconsistent",
            "aws_s3_bucket",
            json!({"policy_status":{"is_public":true}}),
        ),
    ];
    snapshot(
        &store,
        "facts",
        resources,
        vec![edge("lb", "missing", "IN_VPC")],
        ScanStatus::Complete,
    )
    .await?;
    let result = findings::report(&store, "facts", 100).await?;
    assert_eq!(
        result["rules"]["internet_facing_http_listener"]["rows"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        result["rules"]["default_route_to_internet_gateway"]["rows"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert_eq!(
        result["rules"]["blackhole_route"]["rows"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        result["rules"]["s3_policy_reported_public"]["rows"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        result["rules"]["s3_policy_reported_public"]["rows"][0]["bucket_arn"],
        "public"
    );
    assert_eq!(
        result["rules"]["unresolved_architecture_reference"]["rows"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn findings_preserve_collection_gaps_and_real_row_truncation() -> Result<()> {
    let store = Store::open(":memory:").await?;
    store.begin_scan("partial", 1).await?;
    store
        .apply_page(
            "partial",
            CollectionPage {
                scope: scope(Source::Direct, "fixture"),
                page_number: 1,
                next_token: None,
                resources: vec![resource("a", "fixture", json!({}))],
                edges: vec![edge("a", "x", "LINK"), edge("a", "y", "LINK")],
            },
        )
        .await?;
    store
        .record_failure(
            "partial",
            &CollectionFailure {
                scope: scope(Source::Direct, "denied"),
                category: "denied".into(),
                message: "synthetic denial".into(),
            },
        )
        .await?;
    store
        .finish_scan("partial", ScanStatus::Partial, 20)
        .await?;
    let result = findings::report(&store, "partial", 1).await?;
    assert_eq!(result["status"], "partial");
    assert_eq!(result["truncated"], true);
    assert_eq!(
        result["rules"]["unresolved_architecture_reference"]["truncated"],
        true
    );
    assert_eq!(result["rules"]["collection_scope_gap"]["truncated"], false);
    assert_eq!(
        result["rules"]["collection_scope_gap"]["rows"][0]["coverage_status"],
        "failed"
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn findings_never_join_a_listener_from_a_different_scan() -> Result<()> {
    let store = Store::open(":memory:").await?;
    snapshot(
        &store,
        "lb",
        vec![resource(
            "lb",
            "aws_elbv2_load_balancer",
            json!({"scheme":"internet-facing"}),
        )],
        vec![],
        ScanStatus::Complete,
    )
    .await?;
    snapshot(
        &store,
        "listener",
        vec![resource(
            "listener",
            "aws_elbv2_listener",
            json!({"load_balancer_arn":"lb","protocol":"HTTP"}),
        )],
        vec![],
        ScanStatus::Complete,
    )
    .await?;
    let result = findings::report(&store, "lb", 100).await?;
    assert_eq!(
        result["rules"]["internet_facing_http_listener"]["rows"],
        json!([])
    );
    assert!(findings::report(&store, "lb", 0).await.is_err());
    assert!(findings::report(&store, "lb", 1001).await.is_err());
    store.close().await;
    Ok(())
}

#[test]
fn direction_display_matches_cli_value_names() {
    assert_eq!(Direction::Outgoing.to_string(), "outgoing");
    assert_eq!(Direction::Incoming.to_string(), "incoming");
    assert_eq!(Direction::Both.to_string(), "both");
}
