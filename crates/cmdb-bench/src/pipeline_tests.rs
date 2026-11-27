use super::{
    Command, parse_command,
    pipeline::{self, BenchDatabase, BenchResult, Workload},
    pipeline_fixture, pipeline_oracle,
};
use inventory_engine::inventory::{DatabaseUrl, InventoryStore, PgStore, PgWriteConfig, SweepId};
use inventory_harness::postgres::{TestDatabase, admin_url};
use sqlx::{AssertSqlSafe, PgPool};

fn workload(directory: &std::path::Path, url: &str, rows: usize) -> BenchResult<Workload> {
    let catalog = directory.join("runtime.catalog");
    std::fs::write(&catalog, super::pipeline_test_catalog::bytes()?)?;
    Ok(Workload {
        database: BenchDatabase::parse(url)?,
        catalog,
        pages: 2,
        rows,
        concurrency: 2,
        graph: None,
    })
}

async fn open_store(url: &str) -> BenchResult<PgStore> {
    Ok(PgStore::open(&DatabaseUrl::parse(url)?, PgWriteConfig::default()).await?)
}

async fn resource_count(url: &str) -> BenchResult<i64> {
    let pool = PgPool::connect(url).await?;
    let count = sqlx::query_scalar("SELECT COUNT(*) FROM inventory.resources")
        .fetch_one(&pool)
        .await?;
    pool.close().await;
    Ok(count)
}

#[test]
fn given_zero_elapsed_clock_when_rating_then_unavailable_not_failure() -> BenchResult {
    assert_eq!(pipeline::rate(5, std::time::Duration::ZERO)?, "unavailable");
    Ok(())
}

#[test]
fn given_nonzero_elapsed_clock_when_rating_then_fixed_three_decimal_rate() -> BenchResult {
    assert_eq!(
        pipeline::rate(5, std::time::Duration::from_secs(2))?,
        "2.500"
    );
    Ok(())
}

#[tokio::test]
async fn given_real_wire_model_when_committing_tags_and_replay_then_exact_durable_state()
-> BenchResult {
    let directory = tempfile::tempdir()?;
    let database = TestDatabase::create(&admin_url()?).await?;
    let outcome = wire_model_state(directory.path(), database.url()).await;
    let cleanup = database.finish().await;
    outcome?;
    cleanup?;
    Ok(())
}

#[tokio::test]
async fn given_empty_pages_when_committed_then_checkpoints_complete_without_fake_resources()
-> BenchResult {
    let directory = tempfile::tempdir()?;
    let database = TestDatabase::create(&admin_url()?).await?;
    let outcome = empty_page_state(directory.path(), database.url()).await;
    let cleanup = database.finish().await;
    outcome?;
    cleanup?;
    Ok(())
}

#[tokio::test]
async fn given_corrupted_record_when_oracle_runs_then_counts_alone_cannot_claim_success()
-> BenchResult {
    let directory = tempfile::tempdir()?;
    let database = TestDatabase::create(&admin_url()?).await?;
    let outcome = corrupted_record_state(directory.path(), database.url()).await;
    let cleanup = database.finish().await;
    outcome?;
    cleanup?;
    Ok(())
}

async fn wire_model_state(directory: &std::path::Path, url: &str) -> BenchResult {
    pipeline::run(workload(directory, url, 3)?).await?;
    assert_eq!(resource_count(url).await?, 12);
    let store = open_store(url).await?;
    let summary = store
        .sweep_summary(&SweepId::new("cmdb-bench-duplicate_replay".into())?)
        .await?;
    assert_eq!(summary.resources_observed, 12);
    assert_eq!(summary.complete_scopes, 2);
    assert_eq!(summary.work_pending, 0);
    assert!(!store.projection_state().await?.rebuild_required);
    store.close().await;
    Ok(())
}

async fn empty_page_state(directory: &std::path::Path, url: &str) -> BenchResult {
    pipeline::run(workload(directory, url, 0)?).await?;
    let store = open_store(url).await?;
    let summary = store
        .sweep_summary(&SweepId::new("cmdb-bench-duplicate_replay".into())?)
        .await?;
    assert_eq!(summary.complete_scopes, 2);
    assert_eq!(summary.resources_observed, 0);
    assert_eq!(store.projection_state().await?.pending, 0);
    store.close().await;
    Ok(())
}

async fn corrupted_record_state(directory: &std::path::Path, url: &str) -> BenchResult {
    let workload = workload(directory, url, 3)?;
    pipeline::run(workload.clone()).await?;
    let pool = PgPool::connect(url).await?;
    let table: String = sqlx::query_scalar(
        "SELECT table_name FROM inventory.kinds WHERE kind='AWS::DynamoDB::Table'",
    )
    .fetch_one(&pool)
    .await?;
    let corrupt = format!(
        "UPDATE inventory.\"{table}\" SET record=decode('00','hex') WHERE ctid=(SELECT ctid FROM inventory.\"{table}\" LIMIT 1)"
    );
    sqlx::query(AssertSqlSafe(corrupt.as_str()))
        .execute(&pool)
        .await?;
    pool.close().await;
    let store = open_store(url).await?;
    let revision = pipeline_oracle::revision(url).await?;
    let error = pipeline_oracle::check(
        &workload,
        &store,
        &SweepId::new("cmdb-bench-duplicate_replay".into())?,
        pipeline::Phase::Replay,
        revision,
    )
    .await
    .err()
    .ok_or("corrupted record escaped the exact-state oracle")?;
    store.close().await;
    assert_eq!(
        error.to_string(),
        "typed record bytes/hash disagree with independently generated fixture"
    );
    Ok(())
}

#[test]
fn given_pipeline_arguments_when_parsed_then_bounds_and_empty_page_intent_are_preserved()
-> Result<(), Box<dyn std::error::Error>> {
    let command = parse_command(
        [
            "pipeline",
            "postgres://postgres@127.0.0.1:5432/awsinv_benchrip",
            "runtime.catalog",
            "100",
            "0",
            "64",
        ]
        .map(str::to_owned)
        .into_iter(),
    )?;
    assert!(matches!(
        command,
        Command::Pipeline(Workload {
            pages: 100,
            rows: 0,
            concurrency: 64,
            ..
        })
    ));
    let bounds = "pages must be 1..100000, rows/page 0..1000, concurrency 1..64";
    [
        (
            [
                "pipeline",
                "postgres://postgres@127.0.0.1:5432/awsinv_benchrip",
                "runtime.catalog",
                "100",
                "1001",
                "1",
            ],
            bounds,
        ),
        (
            [
                "pipeline",
                "postgres://postgres@127.0.0.1:5432/awsinv_benchrip",
                "runtime.catalog",
                "100",
                "1",
                "65",
            ],
            bounds,
        ),
        (
            [
                "pipeline",
                "postgres://postgres@127.0.0.1:5432/awsinv_benchrip",
                "runtime.catalog",
                "100000",
                "1000",
                "64",
            ],
            "workload exceeds 10000000 resources",
        ),
    ]
    .into_iter()
    .try_for_each(
        |(args, expected)| -> Result<(), Box<dyn std::error::Error>> {
            let error = parse_command(args.map(str::to_owned).into_iter())
                .err()
                .ok_or("out-of-bound workload was accepted")?;
            assert_eq!(error.to_string(), expected);
            Ok(())
        },
    )?;
    Ok(())
}

#[test]
fn given_tag_workload_sizes_then_vertex_and_link_oracles_cover_empty_and_short_prefixes()
-> BenchResult {
    assert_eq!(
        pipeline_fixture::expected_tags(0, pipeline::Phase::TagUpdate)?,
        0
    );
    assert_eq!(
        pipeline_fixture::expected_tags(6, pipeline::Phase::Initial)?,
        12
    );
    assert_eq!(
        pipeline_fixture::expected_tags(6, pipeline::Phase::TagUpdate)?,
        8
    );
    assert_eq!(
        pipeline_fixture::expected_tag_vertices(1, pipeline::Phase::TagUpdate),
        0
    );
    assert_eq!(
        pipeline_fixture::expected_tag_vertices(2, pipeline::Phase::TagUpdate),
        2
    );
    assert_eq!(
        pipeline_fixture::expected_tag_vertices(3, pipeline::Phase::TagUpdate),
        3
    );
    Ok(())
}

#[test]
fn given_database_url_with_password_when_debugged_then_dsn_is_redacted() -> BenchResult {
    let database =
        BenchDatabase::parse("postgres://postgres:secret@127.0.0.1:5432/awsinv_benchrip")?;
    assert_eq!(format!("{database:?}"), "BenchDatabase(redacted)");
    Ok(())
}

#[tokio::test]
async fn given_occupied_target_when_rerun_then_rejected_without_changing_state() -> BenchResult {
    let directory = tempfile::tempdir()?;
    let database = TestDatabase::create(&admin_url()?).await?;
    let outcome = occupied_rerun_state(directory.path(), database.url()).await;
    let cleanup = database.finish().await;
    outcome?;
    cleanup?;
    Ok(())
}

async fn occupied_rerun_state(directory: &std::path::Path, url: &str) -> BenchResult {
    pipeline::run(workload(directory, url, 3)?).await?;
    assert_eq!(resource_count(url).await?, 12);
    pipeline::run(workload(directory, url, 3)?)
        .await
        .err()
        .ok_or("occupied target accepted a second fresh pipeline")?;
    assert_eq!(resource_count(url).await?, 12);
    Ok(())
}

#[tokio::test]
async fn given_concurrent_fresh_initialization_when_both_join_then_exactly_one_admitted()
-> BenchResult {
    let directory = tempfile::tempdir()?;
    let database = TestDatabase::create(&admin_url()?).await?;
    let outcome = concurrent_join_state(directory.path(), database.url()).await;
    let cleanup = database.finish().await;
    outcome?;
    cleanup?;
    Ok(())
}

async fn concurrent_join_state(directory: &std::path::Path, url: &str) -> BenchResult {
    let (first, second) = tokio::join!(
        pipeline::run(workload(directory, url, 3)?),
        pipeline::run(workload(directory, url, 3)?),
    );
    assert_eq!(usize::from(first.is_ok()) + usize::from(second.is_ok()), 1);
    assert_eq!(resource_count(url).await?, 12);
    Ok(())
}
