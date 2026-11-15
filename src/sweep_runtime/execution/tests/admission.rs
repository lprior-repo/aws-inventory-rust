use super::*;
use super::support::{Fixture,TestResult};
use inventory_core::inventory::ScopeOutcome;
use std::sync::atomic::{AtomicUsize,Ordering};

async fn seed_pair(fixture: &Fixture) -> TestResult {
    for branch in ["FIRST","SECOND"] {
        fixture.store.seed_work(WorkSpec::new(fixture.sweep.clone(),
            fixture.scope.clone().with_branch(branch.to_owned())?,None,Vec::new())?).await?;
    }
    Ok(())
}

async fn reject_second_claim(fixture: &Fixture) -> TestResult<sqlx::SqlitePool> {
    let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new()
            .filename(fixture.directory.path().join("scheduler.sqlite"))).await?;
    sqlx::query("CREATE TRIGGER reject_second_claim BEFORE UPDATE ON work_queue
        WHEN NEW.state=1 AND OLD.state=0 AND EXISTS(
            SELECT 1 FROM work_queue WHERE sweep_id=NEW.sweep_id AND state=1)
        BEGIN SELECT RAISE(ABORT,'second-claim fault'); END")
        .execute(&pool).await?;
    Ok(pool)
}

async fn failed_admission(cancel: bool) -> TestResult {
    // Given two real durable work items and a database fault only on the second claim.
    let fixture = Fixture::new(if cancel { "cancel-second-claim" } else { "fail-second-claim" }).await?;
    seed_pair(&fixture).await?;
    let pool = reject_second_claim(&fixture).await?;
    let executor = ApiExecutor::new(executor_config(4)?)?;
    let calls = AtomicUsize::new(0);
    let mut leases = Vec::new();
    // When the second real SQL claim fails, including cancellation during that admission.
    let error = draining::admission::claim_batch(&fixture.store,&executor,4,&mut leases,|| async {
        let ordinal = calls.fetch_add(1,Ordering::SeqCst);
        if ordinal == 1 && cancel { executor.cancel(); }
        fixture.store.claim(&fixture.sweep,fixture.now,fixture.lease_ms).await.map_err(SweepError::from)
    }).await.err().ok_or("second claim unexpectedly succeeded")?;
    // Then the first lease is terminal before return, with the SQL cause preserved.
    assert!(matches!(&error,SweepError::Store(_)));
    assert!(format!("{error:?}").contains("second-claim fault"));
    assert_eq!(calls.load(Ordering::SeqCst),2);
    assert_eq!(executor.is_cancelled(),cancel);
    assert!(leases.is_empty());
    let before = fixture.store.sweep_summary(&fixture.sweep).await?;
    assert_eq!(before.partial_scopes,1);
    assert_eq!(before.work_pending,1);
    let active: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM work_queue WHERE state=1")
        .fetch_one(&pool).await?;
    assert_eq!(active,0,"successful earlier claim was stranded after admission failed");
    tokio::task::yield_now().await;
    assert_eq!(fixture.store.sweep_summary(&fixture.sweep).await?,before);
    let remaining = fixture.store.claim(&fixture.sweep,now_ms()?,fixture.lease_ms).await?
        .ok_or("unclaimed sibling is not immediately recoverable")?;
    assert_eq!(remaining.generation(),1);
    fixture.store.finish_scope(remaining,ScopeOutcome::Complete,now_ms()?).await?;
    assert_eq!(fixture.store.sweep_summary(&fixture.sweep).await?.work_pending,0);
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_first_claim_committed_when_second_claim_fails_then_prior_lease_is_finalized_before_error_returns() -> TestResult {
    failed_admission(false).await
}

#[tokio::test]
async fn given_first_claim_committed_when_second_claim_fails_during_cancellation_then_prior_lease_is_finalized_before_error_returns() -> TestResult {
    failed_admission(true).await
}

#[tokio::test]
async fn given_impossible_batch_capacity_when_admission_starts_then_reservation_fails_before_any_durable_claim() -> TestResult {
    let fixture = Fixture::new("reserve-before-claim").await?;
    seed_pair(&fixture).await?;
    let executor = ApiExecutor::new(executor_config(4)?)?;
    let calls = AtomicUsize::new(0);
    let mut leases = Vec::new();
    let result = draining::admission::claim_batch(&fixture.store,&executor,usize::MAX,&mut leases,|| async {
        let _ = calls.fetch_add(1,Ordering::SeqCst);
        fixture.store.claim(&fixture.sweep,fixture.now,fixture.lease_ms).await.map_err(SweepError::from)
    }).await;
    assert!(result.is_err());
    assert_eq!(calls.load(Ordering::SeqCst),0);
    assert!(leases.is_empty());
    let first = fixture.store.claim(&fixture.sweep,now_ms()?,fixture.lease_ms).await?
        .ok_or("capacity failure consumed unclaimed work")?;
    assert_eq!(first.generation(),1);
    fixture.store.finish_scope(first,ScopeOutcome::Partial,now_ms()?).await?;
    Ok(())
}
