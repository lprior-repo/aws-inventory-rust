use super::support::{
    TestResult, dual_observation, page, resource, typed_relationship_catalog, unresolved, wire_map,
    work,
};
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_core::inventory::{EdgeSource, Relation, UnresolvedReference};
use inventory_core::record::Value;
use inventory_engine::inventory::{
    BinaryStore, DatabaseUrl, InventoryStore, PgStore, PgWriteConfig, ProjectionAck,
    ProjectionEdgeHeader, ProjectionEdgePage, SchemaRegistry, SweepId,
};
use inventory_harness::postgres::{TestDatabase, admin_url};

const SCOPE: &str = "us-east-1";

#[tokio::test]
async fn sqlite_given_legacy_identity_reference_when_reswept_then_only_its_edge_is_removed()
-> TestResult {
    let (_directory, store) = sqlite_fixture().await?;
    reconcile_upgrade(&store).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_legacy_identity_reference_when_reswept_then_only_its_edge_is_removed()
-> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let store = PgStore::create(
        &DatabaseUrl::parse(database.url())?,
        registry()?,
        PgWriteConfig::default(),
    )
    .await?;
    let result = reconcile_upgrade(&store).await;
    store.close().await;
    database.finish().await?;
    result
}

async fn reconcile_upgrade(store: &impl InventoryStore) -> TestResult {
    observe(store, "legacy", 100, true).await?;
    let deliberate_model = ("explicit", "explicit", EdgeSource::Model);
    let peer = ("peer", "target", EdgeSource::Model);
    let reviewed = ("target", "target", EdgeSource::ReviewedRule);
    let explicit = [deliberate_model, peer, reviewed];
    let legacy = [
        deliberate_model,
        peer,
        ("target", "target", EdgeSource::Model),
        reviewed,
    ];
    assert_edges(store, &legacy).await?;
    for update in store.drain_projection(10).await? {
        let key = update.key.ok_or("expected an incremental update")?;
        assert_eq!(
            store
                .ack_projection(ProjectionAck::new(key, update.revision)?)
                .await?,
            1
        );
    }
    assert!(store.drain_projection(10).await?.is_empty());

    observe(store, "upgrade", 200, false).await?;

    assert_edges(store, &explicit).await?;
    let pending = store.drain_projection(10).await?;
    let updates: Vec<_> = pending
        .into_iter()
        .map(|update| (update.key, update.deleted, update.full_rebuild))
        .collect();
    assert_eq!(updates, [(Some(resource("target")?), false, false)]);
    Ok(())
}

async fn observe(
    store: &impl InventoryStore,
    sweep: &str,
    at: i64,
    emit_identity_reference: bool,
) -> TestResult {
    store.seed_works(&[work(sweep, SCOPE)?]).await?;
    let lease = store
        .claim(&SweepId::new(sweep.to_owned())?, at, 20)
        .await?
        .ok_or("missing self-reference lease")?;
    let mut commit = page(
        lease,
        vec![
            dual_observation("target", "target", "target", "unchanged")?,
            dual_observation("explicit", "explicit", "explicit", "unchanged")?,
            dual_observation("peer", "peer", "target", "unchanged")?,
        ],
        true,
        None,
        at.checked_add(1).ok_or("test timestamp overflow")?,
    );
    commit.unresolved = [
        candidate("explicit", "explicit", EdgeSource::Model)?,
        candidate("peer", "target", EdgeSource::Model)?,
        candidate("target", "target", EdgeSource::ReviewedRule)?,
    ]
    .into_iter()
    .chain(
        emit_identity_reference
            .then(|| candidate("target", "target", EdgeSource::Model))
            .transpose()?,
    )
    .collect();
    let result = store.commit_page(commit).await?;
    if !emit_identity_reference {
        assert_eq!(result.stats.resources_changed, 0);
    }
    Ok(())
}

fn candidate(from: &str, target: &str, source: EdgeSource) -> TestResult<UnresolvedReference> {
    let mut reference = unresolved(from, target)?;
    reference.source = source;
    Ok(reference)
}

async fn assert_edges(
    store: &impl InventoryStore,
    expected: &[(&str, &str, EdgeSource)],
) -> TestResult {
    let edges = expected
        .iter()
        .map(|(from, to, source)| {
            Ok(ProjectionEdgeHeader {
                from: resource(from)?,
                to: resource(to)?,
                relation: Relation::parse("references")?,
                source: source.as_str().to_owned(),
            })
        })
        .collect::<TestResult<_>>()?;
    assert_eq!(
        store.projection_edges(None, 10).await?,
        ProjectionEdgePage { edges, next: None }
    );
    Ok(())
}

async fn sqlite_fixture() -> TestResult<(tempfile::TempDir, BinaryStore)> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("inventory.sqlite");
    let store = BinaryStore::create(&path, registry()?).await?;
    Ok((directory, store))
}

fn registry() -> TestResult<SchemaRegistry> {
    let kind = || Value::Text("AWS::Example::Thing".into());
    let rules = [(0, 1), (0, 4), (1, 4)].map(|(source, path)| {
        wire_map([
            (1, kind()),
            (2, Value::List(vec![Value::Int(path)])),
            (3, kind()),
            (4, Value::List(vec![Value::Int(1)])),
            (5, Value::Text("references".into())),
            (6, Value::Int(0)),
            (7, Value::Int(source)),
        ])
    });
    let bytes = typed_relationship_catalog(rules.into_iter().collect::<TestResult<_>>()?, true)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    Ok(SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?)
}
