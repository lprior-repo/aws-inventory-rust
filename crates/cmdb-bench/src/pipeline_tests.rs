use super::{
    Command, parse_command,
    pipeline::{self, BenchResult, Workload},
    pipeline_fixture, pipeline_oracle,
};
use inventory_engine::inventory::{BinaryStore, SweepId};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

fn workload(directory: &std::path::Path, rows: usize) -> BenchResult<Workload> {
    let catalog = directory.join("runtime.catalog");
    std::fs::write(&catalog, super::pipeline_test_catalog::bytes()?)?;
    Ok(Workload {
        path: directory.join("inventory.sqlite"),
        catalog,
        pages: 2,
        rows,
        concurrency: 2,
        graph: None,
    })
}

#[test]
fn given_zero_elapsed_clock_when_rating_then_unavailable_not_failure() -> BenchResult {
    // Given a completed phase whose monotonic clock reads zero elapsed time.
    // When formatting its throughput, Then the rate is explicitly unavailable.
    assert_eq!(pipeline::rate(5, std::time::Duration::ZERO)?, "unavailable");
    Ok(())
}

#[test]
fn given_nonzero_elapsed_clock_when_rating_then_fixed_three_decimal_rate() -> BenchResult {
    // Given five resources over two seconds, When formatting, Then 2.500 per second.
    assert_eq!(
        pipeline::rate(5, std::time::Duration::from_secs(2))?,
        "2.500"
    );
    Ok(())
}

#[tokio::test]
async fn given_real_wire_model_when_committing_tags_and_replay_then_exact_durable_state()
-> BenchResult {
    // Given bounded typed table pages and the real checked-in wire decoder model.
    let directory = tempfile::tempdir()?;
    let workload = workload(directory.path(), 3)?;
    // When initial collection, empty/partial/tag-only updates, and duplicate replay run.
    pipeline::run(workload.clone()).await?;
    // Then the production reader and durable checkpoints expose the exact fixture.
    let database = cmdb_view::Database::open(&workload.path).await?;
    assert_eq!(database.counts().await?.resources, 12);
    let store = BinaryStore::open(&workload.path).await?;
    let summary = store
        .sweep_summary(&SweepId::new("cmdb-bench-duplicate_replay".into())?)
        .await?;
    assert_eq!(summary.resources_observed, 12);
    assert_eq!(summary.complete_scopes, 2);
    assert_eq!(summary.work_pending, 0);
    assert!(!store.projection_state().await?.rebuild_required);
    Ok(())
}

#[tokio::test]
async fn given_empty_pages_when_committed_then_checkpoints_complete_without_fake_resources()
-> BenchResult {
    // Given two paginated scopes whose decoded table lists are empty.
    let directory = tempfile::tempdir()?;
    let workload = workload(directory.path(), 0)?;
    // When all three phases commit actual empty pages.
    pipeline::run(workload.clone()).await?;
    // Then scope outcomes still complete while resource and graph work remain zero.
    let store = BinaryStore::open(&workload.path).await?;
    let summary = store
        .sweep_summary(&SweepId::new("cmdb-bench-duplicate_replay".into())?)
        .await?;
    assert_eq!(summary.complete_scopes, 2);
    assert_eq!(summary.resources_observed, 0);
    assert_eq!(store.projection_state().await?.pending, 0);
    Ok(())
}

#[tokio::test]
async fn given_corrupted_record_when_oracle_runs_then_counts_alone_cannot_claim_success()
-> BenchResult {
    // Given an otherwise correct completed workload with unchanged row counts.
    let directory = tempfile::tempdir()?;
    let workload = workload(directory.path(), 3)?;
    pipeline::run(workload.clone()).await?;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(&workload.path))
        .await?;
    // When one persisted binary payload is corrupted without changing counts/hashes.
    sqlx::query("UPDATE benchmark_tables SET record=X'00' WHERE id=(SELECT id FROM benchmark_tables LIMIT 1)").execute(&pool).await?;
    pool.close().await;
    // Then independent fixture bytes/hash validation rejects the state.
    let store = BinaryStore::open(&workload.path).await?;
    let revision = pipeline_oracle::revision(&workload.path).await?;
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
    assert_eq!(
        error.to_string(),
        "typed record bytes/hash disagree with independently generated fixture"
    );
    Ok(())
}

#[test]
fn given_pipeline_arguments_when_parsed_then_bounds_and_empty_page_intent_are_preserved()
-> Result<(), Box<dyn std::error::Error>> {
    // Given explicit empty-page and maximum concurrency parameters.
    let command = parse_command(
        [
            "pipeline",
            "fresh.sqlite",
            "runtime.catalog",
            "100",
            "0",
            "64",
        ]
        .map(str::to_owned)
        .into_iter(),
    )?;
    // When parsed, zero rows remain valid rather than becoming a dummy workload.
    assert!(matches!(
        command,
        Command::Pipeline(Workload {
            pages: 100,
            rows: 0,
            concurrency: 64,
            ..
        })
    ));
    // Then oversized rows, in-flight concurrency, and total resources are refused.
    let bounds = "pages must be 1..100000, rows/page 0..1000, concurrency 1..64";
    [
        (
            [
                "pipeline",
                "fresh.sqlite",
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
                "fresh.sqlite",
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
                "fresh.sqlite",
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
    // Given fixture prefixes that exercise complete-empty, updated, and retained tags.
    // When independently counting desired tag links and scoped vertices,
    // Then zero/small workloads preserve the precise topology rather than rounding.
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
