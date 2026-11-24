use super::{TestResult, postgres_support as pg};
use cmdb_view::{ScopeFreshnessReport, ScopeOutcome, ScopeOutcomeCounts};

async fn outcomes(fixture: &pg::Fixture) -> TestResult {
    fixture
        .commit("five", "111111111111", "ap-east-1", 0, &["shared"], 50)
        .await?;
    fixture
        .commit("six", "111111111111", "ap-south-1", 0, &["shared"], 60)
        .await?;
    // Keep each scope's own observation rather than carried unobserved history.
    sqlx::query("DELETE FROM inventory.scope_outcomes WHERE (sweep_id,observed_at) NOT IN (('one',10),('two',20),('three',30),('four',40),('five',50),('six',60))")
        .execute(&fixture.pool).await?;
    sqlx::query("UPDATE inventory.scope_outcomes SET outcome=CASE sweep_id WHEN 'two' THEN 'inaccessible' WHEN 'three' THEN 'failed' WHEN 'five' THEN 'unobserved' WHEN 'six' THEN 'unsupported' ELSE outcome END").execute(&fixture.pool).await?;
    sqlx::query("INSERT INTO inventory.scope_outcomes SELECT 'later',scope_key,'partial',70,NULL FROM inventory.scope_outcomes WHERE sweep_id='one'").execute(&fixture.pool).await?;
    Ok(())
}

fn assert_totals(report: &ScopeFreshnessReport) {
    assert_eq!(report.now_ms, 100);
    assert_eq!(report.total_scopes, 6);
    assert_eq!(
        report.counts,
        ScopeOutcomeCounts {
            complete: 1,
            partial: 1,
            inaccessible: 1,
            unsupported: 1,
            failed: 1,
            unobserved: 1
        },
        "{:?}",
        report.scopes
    );
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_mixed_stored_outcomes_when_reading_then_latest_states_and_complete_ages_are_exact()
-> TestResult {
    let fixture = pg::Fixture::new().await?;
    outcomes(&fixture).await?;
    let reader = fixture.reader().await?;
    let report = reader.scope_freshness(100, 1000).await?;
    assert_totals(&report);
    assert_observed_outcomes(&report);
    assert_ages(&report);
    for scope in &report.scopes {
        assert_eq!(scope.service, "example");
        assert!(!scope.has_parent);
    }
    drop(reader);
    fixture.finish().await
}

fn assert_observed_outcomes(report: &ScopeFreshnessReport) {
    let actual = report
        .scopes
        .iter()
        .map(|scope| (scope.region.as_str(), scope.outcome, scope.observed_at_ms))
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        vec![
            ("us-east-1", ScopeOutcome::Inaccessible, 20),
            ("eu-west-1", ScopeOutcome::Failed, 30),
            ("us-west-2", ScopeOutcome::Complete, 40),
            ("ap-east-1", ScopeOutcome::Unobserved, 50),
            ("ap-south-1", ScopeOutcome::Unsupported, 60),
            ("global", ScopeOutcome::Partial, 70),
        ]
    );
}

fn assert_ages(report: &ScopeFreshnessReport) {
    let actual = report
        .scopes
        .iter()
        .map(|scope| {
            (
                scope.sweep_id.as_str(),
                scope.last_complete_at_ms,
                scope.age_ms,
                scope.complete_age_ms,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        vec![
            ("two", None, 80, None),
            ("three", None, 70, None),
            ("four", Some(40), 60, Some(60)),
            ("five", None, 50, None),
            ("six", None, 40, None),
            ("later", Some(10), 30, Some(90)),
        ]
    );
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_bounded_report_when_reading_then_oldest_rows_do_not_bound_global_totals()
-> TestResult {
    let fixture = pg::Fixture::new().await?;
    outcomes(&fixture).await?;
    let reader = fixture.reader().await?;
    let report = reader.scope_freshness(100, 2).await?;
    assert_totals(&report);
    assert_eq!(
        report
            .scopes
            .iter()
            .map(|scope| (
                scope.account.as_str(),
                scope.region.as_str(),
                scope.operation.as_str(),
                scope.observed_at_ms
            ))
            .collect::<Vec<_>>(),
        vec![
            ("111111111111", "us-east-1", "ListThings0", 20),
            ("222222222222", "eu-west-1", "ListThings0", 30)
        ]
    );
    drop(reader);
    fixture.finish().await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_fixture_when_test_returns_early_then_drop_removes_database_with_open_pools()
-> TestResult {
    let fixture = pg::Fixture::new().await?;
    let name = fixture
        .url()
        .rsplit('/')
        .next()
        .ok_or("missing database name")?
        .to_owned();
    let failure: TestResult = async move {
        let _owned_fixture = fixture;
        Err("intentional early fixture exit".into())
    }
    .await;
    assert_eq!(
        failure.err().ok_or("expected fixture error")?.to_string(),
        "intentional early fixture exit"
    );
    let pool = sqlx::PgPool::connect(&inventory_harness::postgres::admin_url()?).await?;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_database WHERE datname=$1")
        .bind(name)
        .fetch_one(&pool)
        .await?;
    assert_eq!(count, 0);
    pool.close().await;
    Ok(())
}

async fn assert_corrupt_scope_is_stored_row(scope_key: &[u8]) -> TestResult {
    let fixture = pg::Fixture::new().await?;
    sqlx::query("INSERT INTO inventory.scope_outcomes(sweep_id,scope_key,outcome,observed_at,detail) VALUES('corrupt',$1,'complete',0,NULL)")
        .bind(scope_key).execute(&fixture.pool).await?;
    let reader = fixture.reader().await?;
    let error = reader
        .scope_freshness(100, 1000)
        .await
        .err()
        .ok_or("expected corrupt stored scope")?;
    drop(reader);
    fixture.finish().await?;
    assert_eq!(error.to_string(), "invalid stored resource row");
    assert!(matches!(error, cmdb_view::ViewError::StoredRow));
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_short_scope_key_when_reading_freshness_then_stored_row_is_returned()
-> TestResult {
    assert_corrupt_scope_is_stored_row(&[0, 0]).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_overflowing_scope_size_when_reading_freshness_then_stored_row_is_returned()
-> TestResult {
    assert_corrupt_scope_is_stored_row(&[255, 255, 255, 255]).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_truncated_scope_field_when_reading_freshness_then_stored_row_is_returned()
-> TestResult {
    assert_corrupt_scope_is_stored_row(&[0, 0, 0, 10, b'a']).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_non_utf8_scope_field_when_reading_freshness_then_stored_row_is_returned()
-> TestResult {
    assert_corrupt_scope_is_stored_row(&[0, 0, 0, 1, 255]).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_invalid_scope_identity_when_reading_freshness_then_stored_row_is_returned()
-> TestResult {
    assert_corrupt_scope_is_stored_row(&[0; 17]).await
}
