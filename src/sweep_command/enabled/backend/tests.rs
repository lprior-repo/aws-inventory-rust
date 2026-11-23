use super::*;
use clap::Parser;
use inventory_engine::inventory::{PendingProjection, ProjectionAck};
use inventory_harness::postgres::{TestDatabase, admin_url};
use std::time::Instant;

#[derive(Parser)]
struct Options {
    #[command(flatten)]
    sweep: SweepArgs,
}

fn options(database: &str) -> Result<SweepArgs> {
    Ok(Options::try_parse_from(["test", "--database", database])?.sweep)
}

#[test]
fn given_repeated_service_selectors_when_parsed_then_order_and_qualification_are_preserved() -> Result<()> {
    let parsed = Options::try_parse_from([
        "test", "--database", "unused", "--service", "ec2:DescribeVpcs", "--service", "sts",
    ])?;
    assert_eq!(parsed.sweep.services, ["ec2:DescribeVpcs", "sts"]);
    assert!(options("unused")?.services.is_empty());
    Ok(())
}

#[test]
fn given_database_password_when_debugged_or_clap_rejects_another_value_then_secret_is_hidden()
-> Result<()> {
    let database = "postgres://user:super-secret@localhost/inventory";
    let options = options(database)?;
    assert!(!format!("{options:?}").contains("super-secret"));
    let error =
        Options::try_parse_from(["test", "--database", database, "--concurrency", "invalid"])
            .err()
            .context("invalid concurrency was accepted")?;
    assert!(!error.to_string().contains("super-secret"));
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL and the locally generated catalog"]
async fn given_full_catalog_when_native_cli_store_is_created_and_resumed_then_schema_and_rebuild_round_trip()
-> Result<()> {
    let database = TestDatabase::create(&admin_url()?).await?;
    let result = native_contract(database.url()).await;
    database.finish().await?;
    result
}

async fn native_contract(url: &str) -> Result<()> {
    let mut options = options(url)?;
    let bytes = read_catalog(&options.catalog)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    options.resume = true;
    let error = open_postgres(&options, &catalog)
        .await
        .err()
        .context("missing schema resumed")?;
    assert_eq!(
        error.to_string(),
        "cannot resume a missing inventory schema"
    );
    options.resume = false;
    create_native(&options, &catalog).await?;
    options.resume = true;
    reopen_native(&options, &catalog).await
}

async fn create_native(options: &SweepArgs, catalog: &RuntimeCatalog<'_>) -> Result<()> {
    let started = Instant::now();
    let store = open_postgres(options, catalog).await?;
    eprintln!(
        "P11 committed_catalog kinds={} create_ms={}",
        store.registry().schemas().len(),
        started.elapsed().as_millis()
    );
    let result = rebuild_contract(&store).await;
    store.close().await;
    result
}

async fn reopen_native(options: &SweepArgs, catalog: &RuntimeCatalog<'_>) -> Result<()> {
    let started = Instant::now();
    let store = open_postgres(options, catalog).await?;
    assert_eq!(store.registry().schemas().len(), catalog.schemas().len());
    assert!(
        store
            .registry()
            .schemas()
            .iter()
            .all(|schema| schema.table().as_str().len() <= 63)
    );
    eprintln!(
        "P11 committed_catalog kinds={} reopen_ms={}",
        store.registry().schemas().len(),
        started.elapsed().as_millis()
    );
    store.close().await;
    Ok(())
}

async fn marker(store: &PgStore) -> Result<PendingProjection> {
    store.request_graph_rebuild().await?;
    let mut rows = store.drain_projection(1).await?;
    assert_eq!(rows.len(), 1);
    let marker = rows.pop().context("rebuild marker is missing")?;
    assert!(marker.full_rebuild && marker.key.is_none());
    Ok(marker)
}

async fn rebuild_contract(store: &PgStore) -> Result<()> {
    let old = marker(store).await?;
    let current = marker(store).await?;
    assert!(current.revision > old.revision);
    verify_acknowledgments(store, old, current).await
}

async fn verify_acknowledgments(
    store: &PgStore,
    old: PendingProjection,
    current: PendingProjection,
) -> Result<()> {
    assert_eq!(acknowledge(store, old).await?, 0);
    assert_eq!(acknowledge(store, current).await?, 1);
    let state = store.projection_state().await?;
    assert_eq!(state.pending, 0);
    assert!(!state.rebuild_required);
    Ok(())
}

async fn acknowledge(store: &PgStore, marker: PendingProjection) -> Result<u64> {
    Ok(store
        .ack_projection(ProjectionAck {
            key: None,
            revision: marker.revision,
        })
        .await?)
}
