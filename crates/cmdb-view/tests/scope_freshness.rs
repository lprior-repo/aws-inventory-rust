//! M7.3: the `scope_freshness` SQL view and the UI-side read query
//! (`Database::scope_freshness`) both report exactly the stored scope outcomes.

#[path = "support/modeled_catalog.rs"]
mod modeled_catalog;
#[path = "support/scope_freshness_script.rs"]
mod script;
#[path = "support/scope_freshness_view.rs"]
mod view;

use cmdb_view::{Database, ScopeFreshness, ScopeOutcome, ScopeOutcomeCounts, ViewError};
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_engine::inventory::{BinaryStore, SchemaRegistry, SweepId, WorkSpec};
use modeled_catalog::{Model, modeled_catalog};
use script::{
    Attempt, Expected, KIND, TestResult, expected, modeled, ordered_script, raw_scope_key, scope,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::path::Path;
use view::{ViewRow, assert_view_ages, assert_view_row, view_rows};

async fn seed(path: &Path) -> TestResult<BinaryStore> {
    let bytes = modeled_catalog(&Model {
        kind: KIND,
        region: "us-east-1",
        table: "scope_freshness_records",
        commit: "scope-freshness",
    })?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let registry = SchemaRegistry::new(&bytes, runtime.schemas().to_vec())?;
    let store = BinaryStore::create(path, registry).await?;
    let ordered = ordered_script();
    for attempt in ordered.iter().filter(|attempt| modeled(attempt)) {
        seed_modeled(&store, attempt).await?;
    }
    seed_raw(path, ordered.iter().filter(|attempt| !modeled(attempt))).await?;
    Ok(store)
}

/// Writes one scope through the engine's own work transitions.
async fn seed_modeled(store: &BinaryStore, attempt: &Attempt) -> TestResult {
    let sweep = SweepId::new(attempt.sweep.to_owned())?;
    store
        .seed_work(WorkSpec::new(
            sweep.clone(),
            scope(attempt)?,
            None,
            vec![1],
        )?)
        .await?;
    let lease = store
        .claim(&sweep, attempt.at, 100)
        .await?
        .ok_or("scope was not claimable")?;
    store
        .finish_scope(lease, attempt.outcome, attempt.at)
        .await?;
    Ok(())
}

/// Writes scopes the test catalog does not model directly into `scope_outcomes`.
async fn seed_raw<'a>(path: &Path, attempts: impl Iterator<Item = &'a Attempt>) -> TestResult {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(path))
        .await?;
    for attempt in attempts {
        sqlx::query(
            "INSERT INTO scope_outcomes(sweep_id,scope_key,outcome,observed_at) VALUES(?1,?2,?3,?4)",
        )
        .bind(attempt.sweep)
        .bind(raw_scope_key(attempt)?)
        .bind(attempt.outcome.as_str())
        .bind(attempt.at)
        .execute(&pool)
        .await?;
    }
    pool.close().await;
    Ok(())
}

fn count(rows: &[Expected]) -> u64 {
    u64::try_from(rows.len()).unwrap_or(u64::MAX)
}

fn outcome_counts(rows: &[Expected]) -> ScopeOutcomeCounts {
    let count_of = |wanted| {
        u64::try_from(rows.iter().filter(|row| row.outcome == wanted).count()).unwrap_or(u64::MAX)
    };
    ScopeOutcomeCounts {
        complete: count_of(ScopeOutcome::Complete),
        partial: count_of(ScopeOutcome::Partial),
        inaccessible: count_of(ScopeOutcome::Inaccessible),
        unsupported: count_of(ScopeOutcome::Unsupported),
        failed: count_of(ScopeOutcome::Failed),
        unobserved: count_of(ScopeOutcome::Unobserved),
    }
}

fn clamped_age(now: i64, at: i64) -> u64 {
    u64::try_from(now.saturating_sub(at)).unwrap_or_default()
}

#[tokio::test]
async fn sql_view_matches_the_stored_scope_outcomes() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("inventory.sqlite");
    let _store = seed(&path).await?;
    let (rows, before, after) = view_rows(&path).await?;
    let expected = expected();
    assert_eq!(rows.len(), expected.len());
    for (row, want) in rows.iter().zip(&expected) {
        assert_view_row(row, want);
        assert_view_ages(row, before, after)?;
    }
    Ok(())
}

/// The UI's read model against what was stored, with its own clock.
fn assert_scope_matches_stored(scope: &ScopeFreshness, want: &Expected, now: i64) {
    assert_eq!(scope.account, want.account);
    assert_eq!(scope.region, want.region);
    assert_eq!(scope.service, want.service);
    assert_eq!(scope.operation, want.operation);
    assert_eq!(scope.has_parent, want.has_parent);
    assert_eq!(scope.sweep_id, want.sweep_id);
    assert_eq!(scope.outcome, want.outcome);
    assert_eq!(scope.observed_at_ms, want.observed_at_ms);
    assert_eq!(scope.last_complete_at_ms, want.last_complete_at_ms);
    assert_eq!(scope.age_ms, clamped_age(now, want.observed_at_ms));
    assert_eq!(
        scope.complete_age_ms,
        want.last_complete_at_ms.map(|at| clamped_age(now, at))
    );
}

/// The UI's read model against the SQL view, row for row.
fn assert_scope_matches_view(scope: &ScopeFreshness, view: &ViewRow) {
    assert_eq!(
        (
            scope.account.as_str(),
            scope.region.as_str(),
            scope.service.as_str(),
            scope.operation.as_str(),
            scope.has_parent,
            scope.sweep_id.as_str(),
            scope.outcome.as_str(),
            scope.observed_at_ms,
            scope.last_complete_at_ms,
        ),
        (
            view.account.as_str(),
            view.region.as_str(),
            view.service.as_str(),
            view.operation.as_str(),
            view.has_parent,
            view.sweep_id.as_str(),
            view.outcome.as_str(),
            view.observed_at,
            view.last_complete_at,
        )
    );
}

#[tokio::test]
async fn ui_read_query_agrees_with_the_sql_view_and_stored_outcomes() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("inventory.sqlite");
    let _store = seed(&path).await?;
    let (rows, _, _) = view_rows(&path).await?;
    let expected = expected();
    let now = 1_000_000;
    let database = Database::open(&path).await?;
    let report = database.scope_freshness(now, 1000).await?;

    assert_eq!(report.now_ms, now);
    assert_eq!(report.total_scopes, count(&expected));
    assert_eq!(report.counts, outcome_counts(&expected));
    assert_eq!(report.scopes.len(), expected.len());
    for ((scope, want), view) in report.scopes.iter().zip(&expected).zip(&rows) {
        assert_scope_matches_stored(scope, want, now);
        assert_scope_matches_view(scope, view);
    }
    // Stalest first.
    assert!(report.scopes.windows(2).all(|pair| {
        pair.first().map(|a| a.observed_at_ms) <= pair.last().map(|b| b.observed_at_ms)
    }));
    Ok(())
}

#[tokio::test]
async fn read_query_bounds_rows_but_not_totals_and_rejects_bad_limits() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("inventory.sqlite");
    let _store = seed(&path).await?;
    let expected = expected();
    let database = Database::open(&path).await?;
    let report = database.scope_freshness(10_000, 2).await?;
    let listed: Vec<(&str, &str, i64)> = report
        .scopes
        .iter()
        .map(|scope| {
            (
                scope.region.as_str(),
                scope.operation.as_str(),
                scope.observed_at_ms,
            )
        })
        .collect();
    let stalest: Vec<(&str, &str, i64)> = expected
        .iter()
        .take(2)
        .map(|row| (row.region, row.operation, row.observed_at_ms))
        .collect();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed, stalest);
    assert_eq!(report.total_scopes, count(&expected));
    assert_eq!(report.counts, outcome_counts(&expected));
    assert!(matches!(
        database.scope_freshness(10_000, 0).await,
        Err(ViewError::PageLimit)
    ));
    assert!(matches!(
        database.scope_freshness(10_000, 1001).await,
        Err(ViewError::PageLimit)
    ));
    Ok(())
}

#[tokio::test]
async fn reopening_the_writer_restores_a_dropped_view_for_existing_databases() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("inventory.sqlite");
    drop(seed(&path).await?);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(&path))
        .await?;
    sqlx::query("DROP VIEW scope_freshness")
        .execute(&pool)
        .await?;
    pool.close().await;
    let database = Database::open(&path).await?;
    assert!(matches!(
        database.scope_freshness(1, 1).await,
        Err(ViewError::Sql(_))
    ));
    let _store = BinaryStore::open(&path).await?;
    let report = database.scope_freshness(1, 1000).await?;
    assert_eq!(report.total_scopes, count(&expected()));
    Ok(())
}
