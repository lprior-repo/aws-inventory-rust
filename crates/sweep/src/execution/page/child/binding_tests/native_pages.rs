use super::persistence::{Edge, Resource, apply, assert_state, wire};
use super::*;
use crate::execution::draining;
use inventory_engine::inventory::{BinaryStore, SchemaRegistry};
use std::{collections::BTreeSet, path::Path};
mod lattice;
mod mail;

async fn seed(
    store: &BinaryStore,
    catalog: &RuntimeCatalog<'_>,
    sweep: &SweepId,
    reverse: bool,
) -> TestResult {
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let mut roots = Vec::new();
    for plan in catalog.plans() {
        draining::append_plan_work(
            &account,
            &home,
            sweep,
            plan,
            &[Region::Regional(home.clone())],
            &mut roots,
        )?;
    }
    let expected = if catalog.lookup("workmail", "ListOrganizations").is_some() {
        vec!["ListOrganizations"]
    } else {
        vec!["ListServiceNetworks", "ListServices"]
    };
    let mut actual = roots
        .iter()
        .map(|work| work.scope().operation().as_str())
        .collect::<Vec<_>>();
    actual.sort_unstable();
    assert_eq!(
        actual, expected,
        "dependent FromAny work was incorrectly admitted as a root"
    );
    if reverse {
        roots.reverse();
    }
    store.seed_works(&roots).await?;
    Ok(())
}

async fn pages(
    path: &Path,
    catalog: &RuntimeCatalog<'_>,
    sweep: &SweepId,
    reverse: bool,
) -> TestResult<BTreeSet<String>> {
    let is_mail = catalog.lookup("workmail", "ListOrganizations").is_some();
    let mut seen = BTreeSet::new();
    for _ in 0..if is_mail { 42 } else { 8 } {
        let store = BinaryStore::open_with_catalog(path, catalog.raw_bytes()).await?;
        let lease = store
            .claim(sweep, 1001, 10000)
            .await?
            .ok_or("discovered native page absent after restart")?;
        let plan = catalog
            .lookup(
                lease.scope().service().as_str(),
                lease.scope().operation().as_str(),
            )
            .ok_or("native plan absent")?;
        let (request, wire) = wire(plan, &lease)?;
        let signature = format!(
            "{}|{}|{}",
            plan.operation_name(),
            wire.uri(),
            std::str::from_utf8(wire.body())?
        );
        assert!(
            seen.insert(signature),
            "duplicate native request or cursor crossed parents"
        );
        let body = if is_mail {
            mail::page(plan, &lease, &wire, reverse)?
        } else {
            lattice::page(plan, &lease, &wire, reverse)?
        };
        apply(&store, catalog, plan, lease, &request, body).await?;
    }
    let store = BinaryStore::open_with_catalog(path, catalog.raw_bytes()).await?;
    assert!(
        store.claim(sweep, 1001, 10000).await?.is_none(),
        "extra native Cartesian or detail work remained"
    );
    Ok(seen)
}

async fn snapshot(
    path: &Path,
    catalog: &RuntimeCatalog<'_>,
) -> TestResult<Vec<(String, String, String, String, Vec<u8>)>> {
    let reader = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(path)
                .read_only(true),
        )
        .await?;
    let mut records = Vec::new();
    for schema in catalog.schemas() {
        let table = schema.table().as_str().replace('"', "\"\"");
        let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT ");
        query
            .push_bind(schema.kind().as_str())
            .push(",account,region,id,record FROM \"")
            .push(table)
            .push("\" ORDER BY account,region,id");
        let rows: Vec<(String, String, String, String, Vec<u8>)> =
            query.build_query_as().fetch_all(&reader).await?;
        records.extend(rows);
    }
    records.sort_unstable();
    reader.close().await;
    Ok(records)
}

pub(super) async fn run(catalog: &RuntimeCatalog<'_>) -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("native-assembly.db");
    let store = BinaryStore::create(
        &path,
        SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?,
    )
    .await?;
    let first = SweepId::new("native-first".to_owned())?;
    seed(&store, catalog, &first, false).await?;
    drop(store);
    let seen = pages(&path, catalog, &first, false).await?;
    let (resources, edges): (BTreeSet<Resource>, BTreeSet<Edge>) =
        if catalog.lookup("workmail", "ListOrganizations").is_some() {
            mail::expected()
        } else {
            lattice::expected()
        };
    assert_state(&path, catalog, &resources, &edges).await?;
    let original = snapshot(&path, catalog).await?;
    assert_eq!(
        original
            .iter()
            .map(|(kind, account, region, id, _)| {
                (account.clone(), region.clone(), kind.clone(), id.clone())
            })
            .collect::<BTreeSet<Resource>>(),
        resources,
        "canonical tables must contain every expected native resource"
    );
    let second = SweepId::new("native-reordered".to_owned())?;
    let store = BinaryStore::open_with_catalog(&path, catalog.raw_bytes()).await?;
    seed(&store, catalog, &second, true).await?;
    drop(store);
    assert_eq!(
        pages(&path, catalog, &second, true).await?,
        seen,
        "wire request identities changed with producer order"
    );
    assert_state(&path, catalog, &resources, &edges).await?;
    assert_eq!(
        snapshot(&path, catalog).await?,
        original,
        "canonical native keys or records changed with producer order"
    );
    Ok(())
}
