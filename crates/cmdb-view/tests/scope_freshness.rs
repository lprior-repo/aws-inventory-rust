#[path = "support/modeled_catalog.rs"]
mod modeled_catalog;
#[path = "support/scope_freshness_script.rs"]
mod script;
#[path = "support/scope_freshness_view.rs"]
mod view;

use cmdb_view::{
    Database, ScopeFreshness, ScopeFreshnessReport, ScopeOutcome, ScopeOutcomeCounts, ViewError,
};
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_engine::inventory::{
    DatabaseUrl, PgStore, PgWriteConfig, SchemaRegistry, SweepId, WorkSpec,
};
use inventory_harness::postgres::{TestDatabase, admin_url};
use modeled_catalog::{Model, modeled_catalog};
use script::{
    Attempt, Expected, KIND, TestResult, expected, modeled, ordered_script, raw_scope_key, scope,
};
use sqlx::PgPool;
use view::{LatestRow, assert_latest_row, latest_rows};

async fn seed(database: &TestDatabase) -> TestResult<PgStore> {
    let bytes = modeled_catalog(&Model {
        kind: KIND,
        region: "us-east-1",
        table: "scope_freshness_records",
        commit: "scope-freshness",
    })?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let registry = SchemaRegistry::new(&bytes, runtime.schemas().to_vec())?;
    let url = DatabaseUrl::parse(database.url())?;
    let store = PgStore::create(&url, registry, PgWriteConfig::default()).await?;
    let ordered = ordered_script();
    for attempt in ordered.iter().filter(|attempt| modeled(attempt)) {
        seed_modeled(&store, attempt).await?;
    }
    seed_raw(database.url(), ordered.iter().filter(|attempt| !modeled(attempt))).await?;
    Ok(store)
}

async fn seed_modeled(store: &PgStore, attempt: &Attempt) -> TestResult {
    let sweep = SweepId::new(attempt.sweep.to_owned())?;
    store
        .seed_works(&[WorkSpec::new(
            sweep.clone(),
            scope(attempt)?,
            None,
            vec![1],
        )?])
        .await?;
    let lease = store
        .claim(&sweep, attempt.at, 100)
        .await?
        .ok_or("scope was not claimable")?;
    store
        .finish_scope(lease, attempt.outcome, attempt.at, None)
        .await?;
    Ok(())
}

async fn seed_raw<'a>(url: &str, attempts: impl Iterator<Item = &'a Attempt>) -> TestResult {
    let pool = PgPool::connect(url).await?;
    for attempt in attempts {
        sqlx::query(
            "INSERT INTO inventory.scope_outcomes(sweep_id,scope_key,outcome,observed_at) VALUES($1,$2,$3,$4)",
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

fn assert_scope_matches_latest(scope: &ScopeFreshness, latest: &LatestRow) {
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
            latest.account.as_str(),
            latest.region.as_str(),
            latest.service.as_str(),
            latest.operation.as_str(),
            latest.has_parent,
            latest.sweep_id.as_str(),
            latest.outcome.as_str(),
            latest.observed_at,
            latest.last_complete_at,
        )
    );
}

fn assert_report(report: &ScopeFreshnessReport, rows: &[Expected], now: i64) {
    assert_eq!(report.now_ms, now);
    assert_eq!(report.total_scopes, count(rows));
    assert_eq!(report.counts, outcome_counts(rows));
    assert_eq!(report.scopes.len(), rows.len());
    for (scope, want) in report.scopes.iter().zip(rows) {
        assert_scope_matches_stored(scope, want, now);
    }
    assert!(report.scopes.windows(2).all(|pair| {
        pair.first().map(|a| a.observed_at_ms) <= pair.last().map(|b| b.observed_at_ms)
    }));
}

#[tokio::test]
async fn postgres_given_scripted_engine_writes_when_reading_latest_rows_then_they_match_the_script()
-> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let store = seed(&database).await?;
    let pool = PgPool::connect(database.url()).await?;
    let rows = latest_rows(&pool).await?;
    let want_rows = expected();
    assert_eq!(rows.len(), want_rows.len());
    for (row, want) in rows.iter().zip(&want_rows) {
        assert_latest_row(row, want);
    }
    pool.close().await;
    store.close().await;
    database.finish().await?;
    Ok(())
}

#[tokio::test]
async fn postgres_given_scripted_engine_writes_when_reading_ui_report_then_it_agrees_with_latest_rows()
-> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let store = seed(&database).await?;
    let pool = PgPool::connect(database.url()).await?;
    let rows = latest_rows(&pool).await?;
    let want_rows = expected();
    let now = 1_000_000;
    let reader = Database::open(database.url()).await?;
    let report = reader.scope_freshness(now, 1000).await?;
    assert_report(&report, &want_rows, now);
    assert_eq!(rows.len(), report.scopes.len());
    for ((scope, latest), want) in report.scopes.iter().zip(&rows).zip(&want_rows) {
        assert_scope_matches_latest(scope, latest);
        assert_latest_row(latest, want);
    }
    drop(reader);
    pool.close().await;
    store.close().await;
    database.finish().await?;
    Ok(())
}

#[tokio::test]
async fn postgres_given_scripted_engine_writes_when_paging_then_rows_bound_but_totals_do_not()
-> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let store = seed(&database).await?;
    let want_rows = expected();
    let reader = Database::open(database.url()).await?;
    let report = reader.scope_freshness(10_000, 2).await?;
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
    let stalest: Vec<(&str, &str, i64)> = want_rows
        .iter()
        .take(2)
        .map(|row| (row.region, row.operation, row.observed_at_ms))
        .collect();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed, stalest);
    assert_eq!(report.total_scopes, count(&want_rows));
    assert_eq!(report.counts, outcome_counts(&want_rows));
    assert!(matches!(
        reader.scope_freshness(10_000, 0).await,
        Err(ViewError::PageLimit)
    ));
    assert!(matches!(
        reader.scope_freshness(10_000, 1001).await,
        Err(ViewError::PageLimit)
    ));
    drop(reader);
    store.close().await;
    database.finish().await?;
    Ok(())
}

#[tokio::test]
async fn postgres_given_reopened_writer_when_reading_then_the_freshness_report_is_identical()
-> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let store = seed(&database).await?;
    let want_rows = expected();
    let now = 1_000_000;
    let reader = Database::open(database.url()).await?;
    let before = reader.scope_freshness(now, 1000).await?;
    assert_report(&before, &want_rows, now);
    drop(reader);
    store.close().await;
    let url = DatabaseUrl::parse(database.url())?;
    let writer = PgStore::open(&url, PgWriteConfig::default()).await?;
    let reader = Database::open(database.url()).await?;
    let after = reader.scope_freshness(now, 1000).await?;
    assert_report(&after, &want_rows, now);
    assert_eq!(before.total_scopes, after.total_scopes);
    assert_eq!(before.counts, after.counts);
    drop(reader);
    writer.close().await;
    database.finish().await?;
    Ok(())
}
