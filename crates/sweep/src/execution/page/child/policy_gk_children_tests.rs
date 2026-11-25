mod cases;
use crate::consumer_fixture as fixture;
mod parent_assertions;
mod scenario;

use super::super::{consumer_index::ConsumerIndex, cursor};
use super::policy_dfgk_tests::{complete_df::json_to_cbor, project_children, projected_references};
use super::*;
use crate::{execution::routing, record::policy_dfgk_tests::fixture as decoder};
use catalog_tool as fixture_catalog;
use fixture::{TestResult, compile};
use inventory_core::catalog::runtime::{RuntimeCatalog, RuntimeProtocol};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, WorkLease};
use serde_json::{Value as Json, json};
use std::collections::BTreeSet;

struct Parent {
    binding: String,
    id: String,
    kind: &'static str,
    row: Json,
}

struct Case {
    service: &'static str,
    date: &'static str,
    root: &'static str,
    root_items: &'static str,
    operation: &'static str,
    items: &'static str,
    input: &'static str,
    token: &'static str,
    kind: &'static str,
    relation: &'static str,
    forbidden: &'static [&'static str],
}

struct Scenario<'a, 'catalog> {
    directory: tempfile::TempDir,
    store: BinaryStore,
    sweep: SweepId,
    account: AccountId,
    home: RegionId,
    consumers: ConsumerIndex<'a, 'catalog>,
}

async fn exercise(case: Case, parents: [Parent; 2]) -> TestResult {
    // Given: the embedded policy and canonical map compile against the real pinned service model.
    let operations = [case.root, case.operation]
        .into_iter()
        .chain(case.forbidden.iter().copied())
        .collect::<Vec<_>>();
    let compiled = compile(case.service, case.date, &operations)?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let root = catalog
        .lookup(case.service, case.root)
        .ok_or("root plan absent")?;
    let child = catalog
        .lookup(case.service, case.operation)
        .ok_or("child plan absent")?;
    assert_eq!(child.kind().as_str(), case.kind);
    for forbidden in case.forbidden {
        assert!(
            catalog.lookup(case.service, forbidden).is_none(),
            "forbidden detail approved: {forbidden}"
        );
    }
    let scenario = Scenario::new(&catalog).await?;
    scenario.seed(root).await?;
    let lease = scenario
        .store
        .claim(&scenario.sweep, 1000, 10000)
        .await?
        .ok_or("root work missing")?;
    let (request, _) = scenario.request(root, &lease)?;
    scenario.apply(&catalog, root, lease, &request,
        json!({(cases::wire(&case, case.root_items)):parents.iter().map(|parent| parent.row.clone()).collect::<Vec<_>>()}), 2).await?;
    let scenario =
        consume_children(scenario, &catalog, child, &compiled.fields, &case, &parents).await?;
    // Then: duplicate child-local IDs survive as separate queryable SQLite nodes and parent edges.
    let reader = scenario.reader().await?;
    assert_persisted(&reader, &compiled.fields, &case, &parents).await?;
    reader.close().await;
    Ok(())
}

async fn consume_children<'a, 'catalog>(
    mut scenario: Scenario<'a, 'catalog>,
    catalog: &RuntimeCatalog<'_>,
    child: &CallablePlan<'_>,
    fields: &fixture_catalog::smithy::Registry,
    case: &Case,
    parents: &[Parent; 2],
) -> TestResult<Scenario<'a, 'catalog>> {
    let input = fields.id(case.input).ok_or("parent binding field absent")?;
    let token = fields.id(case.token).ok_or("continuation field absent")?;
    let mut pages = 0_u32;
    // Exactly two parents, each with an empty continuation page and two populated pages.
    for _ in 0..6 {
        let lease = scenario
            .store
            .claim(&scenario.sweep, 1000, 10000)
            .await?
            .ok_or("child continuation/fanout lost")?;
        assert_eq!(
            lease.scope().operation().as_str(),
            case.operation,
            "a forbidden detail operation was scheduled"
        );
        let page = cursor::Cursor::decode(lease.token())?.pages;
        let (request, wire) = scenario.request(child, &lease)?;
        let value =
            record::path_value(request.root(), &[input])?.ok_or("actual parent binding missing")?;
        let ValueRef::Text(parent) = value else {
            return Err("parent binding is not text".into());
        };
        let owner = parents
            .iter()
            .find(|owner| owner.binding == parent)
            .ok_or("child bound to an unknown parent")?;
        assert_eq!(
            lease
                .scope()
                .parent()
                .ok_or("child lost its canonical parent")?
                .id
                .as_str(),
            owner.id
        );
        let encoded = parent.replace(':', "%3A").replace('/', "%2F");
        assert!(
            wire.contains(parent) || wire.contains(&encoded),
            "real wire omitted bound parent: {wire}"
        );
        let expected_token = match page {
            0 => None,
            1 => Some("empty-continuation"),
            2 => Some("data-continuation"),
            _ => return Err("unexpected continuation count".into()),
        };
        let actual_token = record::path_value(request.root(), &[token])?
            .map(|value| value.to_value())
            .transpose()?;
        assert_eq!(
            actual_token,
            expected_token.map(|value| Value::Text(value.to_owned()))
        );
        if let Some(token) = expected_token {
            assert!(
                wire.contains(token),
                "real wire omitted exact opaque token: {wire}"
            );
        }
        let body = cases::response(case, owner, page)?;
        scenario
            .apply(catalog, child, lease, &request, body, 0)
            .await?;
        // Drop the writer and reload persisted work/cursor state before the next claim.
        scenario = scenario.reopen().await?;
        pages = pages.checked_add(1).ok_or("test page count overflow")?;
    }
    assert_eq!(pages, 6);
    assert!(
        scenario
            .store
            .claim(&scenario.sweep, 1000, 10000)
            .await?
            .is_none(),
        "unexpected detail work remained"
    );
    Ok(scenario)
}

type Edge = (String, String, String, String, String);

async fn assert_persisted(
    reader: &sqlx::SqlitePool,
    fields: &fixture_catalog::smithy::Registry,
    case: &Case,
    parents: &[Parent; 2],
) -> TestResult {
    let mut expected_nodes = parents
        .iter()
        .map(|parent| (parent.kind.to_owned(), parent.id.clone()))
        .collect::<BTreeSet<_>>();
    let mut expected_edges = BTreeSet::new();
    for parent in parents {
        for local in ["shared", "later"] {
            let id = cases::expected_id(case, parent, local);
            expected_nodes.insert((case.kind.to_owned(), id.clone()));
            expected_edges.insert((
                case.kind.to_owned(),
                id,
                parent.kind.to_owned(),
                parent.id.clone(),
                case.relation.to_owned(),
            ));
        }
    }
    let nodes: Vec<(String, String)> =
        sqlx::query_as("SELECT kind,id FROM resources ORDER BY kind,id")
            .fetch_all(reader)
            .await?;
    let edges: Vec<Edge> = sqlx::query_as("SELECT from_kind,from_id,to_kind,to_id,relation FROM edges ORDER BY from_kind,from_id,to_kind,to_id,relation").fetch_all(reader).await?;
    assert_eq!(
        nodes.into_iter().collect::<BTreeSet<_>>(),
        expected_nodes,
        "{}",
        case.operation
    );
    assert_eq!(
        edges.into_iter().collect::<BTreeSet<_>>(),
        expected_edges,
        "{}",
        case.operation
    );
    let saved: Vec<(String, Vec<u8>)> =
        sqlx::query_as("SELECT id,record FROM resource_facets WHERE kind=? ORDER BY id")
            .bind(case.kind)
            .fetch_all(reader)
            .await?;
    assert_eq!(saved.len(), 4, "child-local IDs collapsed across parents");
    for (id, bytes) in saved {
        let record = Record::from_bytes(bytes)?;
        cases::assert_metadata(case, fields, &id, &record)?;
    }
    Ok(())
}
