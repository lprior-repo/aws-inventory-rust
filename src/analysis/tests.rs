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

#[test]
fn direction_display_matches_cli_value_names() {
    assert_eq!(Direction::Outgoing.to_string(), "outgoing");
    assert_eq!(Direction::Incoming.to_string(), "incoming");
    assert_eq!(Direction::Both.to_string(), "both");
}

mod diff_scenarios;
mod finding_scenarios;
mod limits;
mod traversal;
