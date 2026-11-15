//! Real schema-15 writer/reader contracts. A retained DatabaseSnapshot pins table,
//! detail, and one-hop edge reads to its opening SQLite transaction. The public
//! API has no session revision token; record contents distinguish the two states.
//! GraphConnection's native multi-hop traversal uses separate frontier queries
//! and is currently NOT cross-hop snapshot-consistent. These tests do not claim it is.

#[path = "database/tag_snapshot_contract.rs"]
mod tag_snapshot_contract;

use cmdb_view::{
    Database, EdgePageRequest, GraphEdge, GraphNode, PageRequest, ResourceFilter,
};
use inventory_core::{
    catalog::{CATALOG_VERSION, CatalogKey, runtime::RuntimeCatalog},
    inventory::{
        AccountId, Edge, EdgeSource, Kind, Observation, OperationId, Region, Relation,
        ResourceId, ResourceKey, ScopeKey, ServiceId,
    },
    record::{Exclusions, FieldId, Key, Record, Value},
};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
use std::{collections::BTreeMap, error::Error, sync::Arc, time::Duration};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
const ACCOUNT: &str = "111111111111";
const REGION: &str = "us-east-1";
const KIND: &str = "AWS::Example::Thing";

fn wire(fields: impl IntoIterator<Item = (u32, Value)>) -> TestResult<Value> {
    Ok(Value::Map(fields.into_iter().map(|(id, value)| {
        Ok((Key::Field(FieldId::new(id)?), value))
    }).collect::<Result<BTreeMap<_, _>, inventory_core::record::RecordError>>()?))
}

fn catalog_map(fields: impl IntoIterator<Item = (CatalogKey, Value)>) -> TestResult<Value> {
    Ok(Value::Map(fields.into_iter().map(|(key, value)| {
        Ok((Key::Field(key.field()?), value))
    }).collect::<TestResult<BTreeMap<_, _>>>()?))
}

fn catalog() -> TestResult<Vec<u8>> {
    let plan = wire([
        (1, Value::Text("example".into())), (2, Value::Text("ListThings".into())),
        (3, Value::Text(KIND.into())), (4, Value::Int(0)),
        (5, wire([(1, Value::Int(0))])?),
        (6, wire([(1, Value::Int(0)), (2, Value::List(vec![Value::Int(1)]))])?),
        (7, Value::List(vec![])),
        (8, wire([(1, wire([(1, Value::Int(0))])?), (2, Value::List(vec![]))])?),
        (9, Value::List(vec![])),
        (10, wire([
            (1, Value::Text("https://example.invalid".into())),
            (2, Value::Text(REGION.into())), (3, Value::Text("example".into())),
            (4, Value::Int(0)), (5, Value::Int(0)),
        ])?),
        (11, wire([
            (1, Value::Text("native reader contract".into())),
            (2, Value::Text("test".into())), (3, Value::Text("2026-11-15".into())),
        ])?),
        (12, wire([(1, Value::Int(0))])?), (13, Value::Int(5)),
    ])?;
    let column = wire([(1, Value::List(vec![Value::Int(1)])), (2, Value::Int(3))])?;
    // A valid SQL keyword identifier must be quoted by BOTH writer and reader.
    let schema = wire([
        (1, Value::Text(KIND.into())), (2, Value::Text("select".into())),
        (3, Value::List(vec![column])), (4, Value::Null),
    ])?;
    let coverage = wire([
        (1, Value::Null), (2, Value::Text(KIND.into())), (3, Value::Bool(true)),
        (4, Value::Null), (5, Value::Bool(true)),
    ])?;
    let decision = wire([
        (1, Value::Text("example".into())), (2, Value::Text("ListThings".into())),
        (3, Value::Int(0)),
    ])?;
    let runtime = wire([
        (1, Value::Int(2)), (2, Value::List(vec![plan])),
        (3, Value::List(vec![schema])), (4, Value::List(vec![])),
        (5, Value::List(vec![coverage])), (6, Value::List(vec![decision])),
    ])?;
    let operation = catalog_map([
        (CatalogKey::Name, Value::Text("ListThings".into())),
        (CatalogKey::Flags, Value::Int(31)), (CatalogKey::Input, Value::Int(-1)),
        (CatalogKey::Output, Value::Int(-14)),
    ])?;
    let service = catalog_map([
        (CatalogKey::Name, Value::Text("example".into())),
        (CatalogKey::ApiVersion, Value::Text("1".into())),
        (CatalogKey::ServiceShape, Value::Text("n#example".into())),
        (CatalogKey::Protocols, Value::List(vec![Value::Text("aws.protocols#restJson1".into())])),
        (CatalogKey::Operations, Value::List(vec![operation])),
        (CatalogKey::Shapes, Value::List(vec![])),
        (CatalogKey::ShapeOffsets, Value::Bytes(vec![])),
        (CatalogKey::Resources, Value::List(vec![])),
    ])?;
    let offsets = (0_u32..8).flat_map(|index| [index * 2, (index + 1) * 2])
        .flat_map(u32::to_be_bytes).collect();
    let root = catalog_map([
        (CatalogKey::Format, Value::Int(CATALOG_VERSION)),
        (CatalogKey::ModelsCommit, Value::Text("native-contract-fixture".into())),
        (CatalogKey::FieldNames, Value::Bytes(b"a1a2a3a4a5a6a7a8".to_vec())),
        (CatalogKey::FieldNameOffsets, Value::Bytes(offsets)),
        (CatalogKey::Services, Value::List(vec![Value::Bytes(Record::encode(&service)?.into_bytes())])),
    ])?;
    let Value::Map(mut fields) = root else { return Err("catalog root must be a map".into()); };
    fields.insert(Key::Field(FieldId::new(60_000)?), runtime);
    Ok(Record::encode(&Value::Map(fields))?.into_bytes())
}

async fn fixture() -> TestResult<(tempfile::TempDir, BinaryStore, Database)> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("inventory.sqlite");
    let bytes = catalog()?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let registry = SchemaRegistry::new(&bytes, runtime.schemas().to_vec())?;
    let store = BinaryStore::create(&path, registry).await?;
    let database = Database::open(&path).await?;
    Ok((directory, store, database))
}

fn key(account: &str, region: &str, id: &str) -> TestResult<ResourceKey> {
    Ok(ResourceKey {
        account: AccountId::parse(account)?, region: Region::parse(region)?,
        kind: Kind::parse(KIND)?, id: ResourceId::parse(id)?,
    })
}

fn node(account: &str, region: &str, id: &str) -> GraphNode {
    GraphNode { account: account.into(), region: region.into(), kind: KIND.into(), id: id.into() }
}

fn record(value: &str) -> TestResult<Record> {
    Record::encode(&wire([(1, Value::Text(value.into()))])?).map_err(Into::into)
}

fn expected_edge(account: &str, region: &str, from: &str, to: &str) -> GraphEdge {
    GraphEdge {
        from: node(account, region, from), to: node(account, region, to),
        relation: "references".into(), provenance: "model".into(),
    }
}

async fn commit(
    store: &BinaryStore, sweep: &str, location: (&str, &str),
    rows: &[(&str, &str)], links: &[(&str, &str)], at: i64,
) -> TestResult {
    let (account, region) = location;
    let scope = ScopeKey::new(
        AccountId::parse(account)?, Region::parse(region)?, ServiceId::parse("example")?,
        OperationId::parse("ListThings")?, None,
    )?;
    let sweep = SweepId::new(sweep.into())?;
    store.seed_work(WorkSpec::new(sweep.clone(), scope, None, vec![1])?).await?;
    let lease = store.claim(&sweep, at, 100).await?.ok_or("missing fixture lease")?;
    let observations = rows.iter().map(|(id, value)| {
        Ok(Observation::new(key(account, region, id)?, record(value)?,
            &Exclusions::none(), Some(Arc::from(*value)))?)
    }).collect::<TestResult<Vec<_>>>()?;
    let edges = links.iter().map(|(from, to)| {
        Ok(Edge { from: key(account, region, from)?, to: key(account, region, to)?,
            relation: Relation::parse("references")?, source: EdgeSource::Model })
    }).collect::<TestResult<Vec<_>>>()?;
    store.commit_page(PageCommit {
        lease, observations, edges, unresolved: vec![], tags: vec![], next_token: None,
        continuation_digest: None, children: vec![], done: true, observed_at_ms: at,
    }).await?;
    Ok(())
}

fn page(limit: usize) -> PageRequest { PageRequest { limit, after: None } }
fn edges(limit: usize) -> EdgePageRequest { EdgePageRequest { limit, after: None } }
fn filter(account: &str, region: &str) -> ResourceFilter {
    ResourceFilter { account: Some(account.into()), region: Some(region.into()),
        kind: Some(KIND.into()), text: None, tag: None }
}

#[tokio::test]
async fn given_retained_session_when_writer_commits_then_table_detail_and_edges_keep_opening_state() -> TestResult {
    tokio::time::timeout(Duration::from_secs(15), async {
        // Given: pin before any consumer read, not merely before the second page.
        let (_directory, store, db) = fixture().await?;
        commit(&store, "initial", (ACCOUNT, REGION),
            &[("root", "revision-one"), ("old", "old-target"), ("new", "new-target")],
            &[("root", "old")], 100).await?;
        let mut retained = db.snapshot().await?;
        // When: record and outgoing neighborhood change in one production commit.
        commit(&store, "update", (ACCOUNT, REGION), &[("root", "revision-two")],
            &[("root", "new")], 200).await?;
        let table = retained.page(&filter(ACCOUNT, REGION), page(10)).await?;
        let neighborhood = retained.edges_page(&node(ACCOUNT, REGION, "root"), edges(10)).await?;
        let detail = retained.detail(&node(ACCOUNT, REGION, "root")).await?.ok_or("old root missing")?;
        // Then: exact old table/record/edge sets, contrasted with a fresh session.
        assert_eq!(table.total_matching, 3);
        assert_eq!(table.resources.iter().map(|row| (row.key.id.as_str(), row.name.as_deref()))
            .collect::<Vec<_>>(), vec![("new", Some("new-target")), ("old", Some("old-target")), ("root", Some("revision-one"))]);
        assert_eq!(detail.record, record("revision-one")?);
        assert_eq!(detail.key, key(ACCOUNT, REGION, "root")?);
        assert_eq!(detail.content_hash, detail.record.content_hash(&Exclusions::none())?);
        assert_eq!(neighborhood.total_matching, 1);
        assert_eq!(neighborhood.edges, vec![expected_edge(ACCOUNT, REGION, "root", "old")]);
        let fresh = db.detail(&node(ACCOUNT, REGION, "root")).await?.ok_or("new root missing")?;
        assert_eq!(fresh.record, record("revision-two")?);
        assert_ne!(fresh.content_hash, detail.content_hash);
        assert_eq!(db.edges_page(&node(ACCOUNT, REGION, "root"), edges(10)).await?.edges,
            vec![expected_edge(ACCOUNT, REGION, "root", "new")]);
        retained.finish().await?;
        Ok(())
    }).await?
}

#[tokio::test]
async fn given_escaped_resource_ids_and_keyword_table_when_reading_then_filters_remain_literal() -> TestResult {
    tokio::time::timeout(Duration::from_secs(15), async {
        // Given: SQL/Cypher-looking values are real identifiers, not query fragments.
        let (_directory, store, db) = fixture().await?;
        let hostile = "root'\\%_`) OR 1=1 --";
        commit(&store, "hostile", (ACCOUNT, REGION),
            &[(hostile, "MiXeD%_needle"), ("ordinary", "mixedXXneedle"), ("leaf", "leaf")],
            &[(hostile, "leaf")], 100).await?;
        // When: wildcard-looking text and escaped identity reach the public readers.
        let mut literal = filter(ACCOUNT, REGION);
        literal.text = Some("mixed%_NEEDLE".into());
        let found = db.page(&literal, page(10)).await?;
        let detail = db.detail(&node(ACCOUNT, REGION, hostile)).await?.ok_or("escaped identity missing")?;
        let neighborhood = db.edges_page(&node(ACCOUNT, REGION, hostile), edges(10)).await?;
        // Then: literal substring semantics, exact identity/record, no expansion.
        assert_eq!(found.total_matching, 1);
        assert_eq!(found.resources.iter().map(|row| &row.key).collect::<Vec<_>>(), vec![&key(ACCOUNT, REGION, hostile)?]);
        assert_eq!(detail.record, record("MiXeD%_needle")?);
        assert_eq!(neighborhood.total_matching, 1);
        assert_eq!(neighborhood.edges, vec![expected_edge(ACCOUNT, REGION, hostile, "leaf")]);
        literal.text = Some("' OR 1=1 --".into());
        assert_eq!(db.page(&literal, page(10)).await?.total_matching, 0);
        assert_eq!(db.counts().await?.resources, 3);
        Ok(())
    }).await?
}

#[tokio::test]
async fn given_shared_ids_in_three_scopes_when_reading_then_tables_details_and_edges_do_not_leak() -> TestResult {
    tokio::time::timeout(Duration::from_secs(15), async {
        // Given: identical resource IDs, with different account/region and contents.
        let (_directory, store, db) = fixture().await?;
        commit(&store, "east", (ACCOUNT, REGION), &[("shared", "east"), ("leaf", "east-leaf")], &[("shared", "leaf")], 100).await?;
        commit(&store, "west", (ACCOUNT, "us-west-2"), &[("shared", "west"), ("leaf", "west-leaf")], &[("leaf", "shared")], 200).await?;
        commit(&store, "other", ("222222222222", REGION), &[("shared", "other"), ("leaf", "other-leaf")], &[("shared", "leaf")], 300).await?;
        // When: select precisely one identity, including its incoming edges.
        let selected = node(ACCOUNT, "us-west-2", "shared");
        let table = db.page(&filter(ACCOUNT, "us-west-2"), page(10)).await?;
        let detail = db.detail(&selected).await?.ok_or("west resource missing")?;
        let neighborhood = db.edges_page(&selected, edges(10)).await?;
        // Then: neither same-account other-region nor other-account same-region leaks.
        assert_eq!(table.total_matching, 2);
        assert_eq!(table.resources.iter().map(|row| row.key.clone()).collect::<Vec<_>>(),
            vec![key(ACCOUNT, "us-west-2", "leaf")?, key(ACCOUNT, "us-west-2", "shared")?]);
        assert_eq!(detail.record, record("west")?);
        assert_eq!(neighborhood.total_matching, 1);
        assert_eq!(neighborhood.edges, vec![expected_edge(ACCOUNT, "us-west-2", "leaf", "shared")]);
        let absent_kind = GraphNode { kind: "AWS::Example::Other".into(), ..selected };
        assert_eq!(db.detail(&absent_kind).await?, None);
        assert_eq!(db.edges_page(&absent_kind, edges(10)).await?.edges, vec![]);
        let hostile_scope = ResourceFilter { account: Some("' OR 1=1 --".into()), ..ResourceFilter::default() };
        assert_eq!(db.page(&hostile_scope, page(10)).await?.resources, vec![]);
        Ok(())
    }).await?
}

#[tokio::test]
async fn given_unsorted_native_rows_when_limits_truncate_then_repeated_pages_have_exact_stable_prefixes() -> TestResult {
    tokio::time::timeout(Duration::from_secs(15), async {
        // Given: input insertion order differs from full resource/edge identity order.
        let (_directory, store, db) = fixture().await?;
        commit(&store, "bounded", (ACCOUNT, REGION),
            &[("z", "z"), ("root", "root"), ("b", "b"), ("a", "a")],
            &[("root", "z"), ("root", "b"), ("root", "a")], 100).await?;
        let mut retained = db.snapshot().await?;
        // When: table and neighborhood are both truncated below their cardinality.
        let first = retained.page(&ResourceFilter::default(), page(2)).await?;
        let repeated = retained.page(&ResourceFilter::default(), page(2)).await?;
        let first_edges = retained.edges_page(&node(ACCOUNT, REGION, "root"), edges(2)).await?;
        let repeated_edges = retained.edges_page(&node(ACCOUNT, REGION, "root"), edges(2)).await?;
        // Then: deterministic exact prefixes, real continuation, and exact tails.
        assert_eq!(first, repeated);
        assert_eq!(first.total_matching, 4);
        assert_eq!(first.resources.iter().map(|row| row.key.id.as_str()).collect::<Vec<_>>(), vec!["a", "b"]);
        assert_eq!(first.next.as_ref().map(|cursor| (&*cursor.account, &*cursor.region, &*cursor.kind, &*cursor.id)), Some((ACCOUNT, REGION, KIND, "b")));
        let tail = retained.page(&ResourceFilter::default(), PageRequest { limit: 2, after: first.next }).await?;
        assert_eq!(tail.resources.iter().map(|row| row.key.id.as_str()).collect::<Vec<_>>(), vec!["root", "z"]);
        assert_eq!((tail.total_matching, tail.next), (4, None));
        assert_eq!(first_edges, repeated_edges);
        assert_eq!(first_edges.total_matching, 3);
        assert_eq!(first_edges.edges, vec![expected_edge(ACCOUNT, REGION, "root", "a"), expected_edge(ACCOUNT, REGION, "root", "b")]);
        assert_eq!(first_edges.next.as_ref().map(|cursor| &cursor.edge), first_edges.edges.last());
        let last = retained.edges_page(&node(ACCOUNT, REGION, "root"), EdgePageRequest { limit: 2, after: first_edges.next }).await?;
        assert_eq!(last.edges, vec![expected_edge(ACCOUNT, REGION, "root", "z")]);
        assert_eq!((last.total_matching, last.next), (3, None));
        retained.finish().await?;
        Ok(())
    }).await?
}
