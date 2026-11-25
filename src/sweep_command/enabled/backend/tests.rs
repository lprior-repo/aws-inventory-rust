use super::*;
use clap::Parser;
use inventory_engine::inventory::{PendingProjection, ProjectionAck};
use inventory_harness::postgres::{TestDatabase, admin_url};
use std::time::Instant;

#[test]
fn given_copy_min_rows_unset_when_configured_then_copy_is_off() -> Result<()> {
    let config = postgres_config_value(None)?;
    let default_config = PgWriteConfig::default();
    assert_eq!(config, default_config);
    Ok(())
}

#[test]
fn given_copy_min_rows_one_when_configured_then_copy_enabled_at_one_row() -> Result<()> {
    let config = postgres_config_value(Some("1".to_owned()))?;
    let expected = PgWriteConfig::default().with_copy_min_rows(std::num::NonZeroUsize::MIN);
    assert_eq!(config, expected);
    Ok(())
}

#[test]
fn given_copy_min_rows_zero_when_configured_then_error() -> Result<()> {
    let error = postgres_config_value(Some("0".to_owned()))
        .err()
        .context("zero should have been rejected")?;
    assert!(
        error
            .to_string()
            .contains("INVENTORY_POSTGRES_COPY_MIN_ROWS"),
        "error must name the variable"
    );
    Ok(())
}

#[test]
fn given_copy_min_rows_non_numeric_when_configured_then_error() -> Result<()> {
    let error = postgres_config_value(Some("abc".to_owned()))
        .err()
        .context("non-numeric should have been rejected")?;
    assert!(
        error
            .to_string()
            .contains("INVENTORY_POSTGRES_COPY_MIN_ROWS"),
        "error must name the variable"
    );
    Ok(())
}

#[derive(Parser)]
struct Options {
    #[command(flatten)]
    sweep: SweepArgs,
}

fn options(database: &str) -> Result<SweepArgs> {
    Ok(Options::try_parse_from(["test", "--database", database])?.sweep)
}

#[test]
fn given_repeated_service_selectors_when_parsed_then_order_and_qualification_are_preserved()
-> Result<()> {
    let parsed = Options::try_parse_from([
        "test",
        "--database",
        "unused",
        "--service",
        "ec2:DescribeVpcs",
        "--service",
        "sts",
    ])?;
    assert_eq!(parsed.sweep.services, ["ec2:DescribeVpcs", "sts"]);
    assert!(options("unused")?.services.is_empty());
    Ok(())
}

#[test]
fn given_transport_flags_when_parsed_then_sweep_receives_the_pool_settings() -> Result<()> {
    let parsed = Options::try_parse_from([
        "test",
        "--database",
        "unused",
        "--idle-timeout-ms",
        "10000",
        "--pool-idle-per-host",
        "32",
    ])?;
    let config = config(&parsed.sweep);
    assert_eq!(
        config.idle_timeout,
        Some(std::time::Duration::from_secs(10))
    );
    assert_eq!(config.pool_idle_per_host, Some(32));
    Ok(())
}

#[test]
fn given_child_transport_environment_when_parsed_then_sweep_receives_the_pool_settings()
-> Result<()> {
    const TEST_NAME: &str =
        "given_child_transport_environment_when_parsed_then_sweep_receives_the_pool_settings";
    const MARKER: &str = "INVENTORY_TEST_TRANSPORT_ENV";
    if std::env::var_os(MARKER).is_some() {
        let parsed = options("unused")?;
        let config = config(&parsed);
        assert_eq!(
            config.idle_timeout,
            Some(std::time::Duration::from_secs(10))
        );
        assert_eq!(config.pool_idle_per_host, Some(32));
        return Ok(());
    }
    let output = std::process::Command::new(std::env::current_exe()?)
        .arg(TEST_NAME)
        .env(MARKER, "1")
        .env("INVENTORY_SWEEP_IDLE_TIMEOUT_MS", "10000")
        .env("INVENTORY_SWEEP_POOL_IDLE_PER_HOST", "32")
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    Ok(())
}

#[tokio::test]
async fn given_unknown_service_selector_when_sweep_runs_then_exit_two_precedes_database_creation()
-> Result<()> {
    assert_unknown_selector("unknown-service").await?;
    assert_unknown_selector("ec2:NoSuchOperation").await
}

async fn assert_unknown_selector(selector: &str) -> Result<()> {
    let root = tempfile::tempdir()?;
    let database = root.path().join("inventory.bin");
    let options = Options::try_parse_from([
        "test",
        "--database",
        database
            .to_str()
            .context("test database path must be UTF-8")?,
        "--service",
        selector,
    ])?
    .sweep;
    assert_eq!(run(&options).await?, std::process::ExitCode::from(2));
    assert!(!database.exists());
    assert_eq!(std::fs::read_dir(root.path())?.count(), 0);
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

#[test]
fn given_child_database_environment_when_parsed_then_database_argument_is_optional() -> Result<()> {
    const MARKER: &str = "INVENTORY_TEST_DATABASE_ENV";
    const DATABASE: &str = "postgres://user:private-password@localhost/inventory";
    if std::env::var_os(MARKER).is_some() {
        let parsed = Options::try_parse_from(["test"])?;
        assert_eq!(parsed.sweep.database, DATABASE);
        assert!(!format!("{:?}", parsed.sweep).contains("private-password"));
        return Ok(());
    }
    let output = std::process::Command::new(std::env::current_exe()?)
        .arg("given_child_database_environment_when_parsed_then_database_argument_is_optional")
        .env(MARKER, "1")
        .env("INVENTORY_SWEEP_DATABASE", DATABASE)
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
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
