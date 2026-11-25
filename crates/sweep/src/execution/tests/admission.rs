use super::support::{Fixture, TestResult};
use super::*;
use inventory_core::inventory::ScopeOutcome;

async fn seed_pair(fixture: &Fixture) -> TestResult {
    for ordinal in 0..2 {
        fixture
            .store
            .seed_work(WorkSpec::new(
                fixture.sweep.clone(),
                fixture.scope_for(ordinal)?,
                None,
                Vec::new(),
            )?)
            .await?;
    }
    Ok(())
}

async fn reject_second_claim(fixture: &Fixture) -> TestResult<sqlx::SqlitePool> {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(fixture.directory.path().join("scheduler.sqlite")),
        )
        .await?;
    sqlx::query(
        "CREATE TRIGGER reject_second_claim BEFORE UPDATE ON work_queue
        WHEN NEW.state=1 AND OLD.state=0 AND EXISTS(
            SELECT 1 FROM work_queue WHERE sweep_id=NEW.sweep_id AND state=1)
        BEGIN SELECT RAISE(ABORT,'second-claim fault'); END",
    )
    .execute(&pool)
    .await?;
    Ok(pool)
}

async fn recover_complete(fixture: &Fixture) -> TestResult<i64> {
    let lease = fixture
        .store
        .claim(&fixture.sweep, now_ms()?, fixture.lease_ms)
        .await?
        .ok_or("released work is not immediately recoverable")?;
    let generation = lease.generation();
    fixture
        .store
        .finish_scope(lease, ScopeOutcome::Complete, now_ms()?, None)
        .await?;
    Ok(generation)
}

#[tokio::test]
async fn given_first_claim_committed_when_the_batch_claim_fails_then_the_first_lease_is_released_before_the_error_returns()
-> TestResult {
    let fixture = Fixture::new("fail-batch-claim").await?;
    seed_pair(&fixture).await?;
    let pool = reject_second_claim(&fixture).await?;
    let error =
        draining::admission::claim_some(&fixture.store, &fixture.sweep, fixture.lease_ms, 4, true)
            .await
            .err()
            .ok_or("batch claim unexpectedly succeeded")?;
    assert!(matches!(&error, SweepError::Store(_)));
    assert!(format!("{error:?}").contains("second-claim fault"));
    let before = fixture.store.sweep_summary(&fixture.sweep).await?;
    assert_eq!(before.partial_scopes, 0);
    assert_eq!(before.work_pending, 2);
    let active: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM work_queue WHERE state=1")
        .fetch_one(&pool)
        .await?;
    assert_eq!(
        active, 0,
        "successful earlier claim was stranded after the batch claim failed"
    );
    let generations = [
        recover_complete(&fixture).await?,
        recover_complete(&fixture).await?,
    ];
    assert!(generations.contains(&1));
    assert!(generations.contains(&3));
    assert_eq!(
        fixture
            .store
            .sweep_summary(&fixture.sweep)
            .await?
            .work_pending,
        0
    );
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_two_pending_scopes_when_claimed_with_and_without_depth_then_the_limit_holds_and_depth_is_reported()
-> TestResult {
    let fixture = Fixture::new("claim-some-limits").await?;
    seed_pair(&fixture).await?;
    let first =
        draining::admission::claim_some(&fixture.store, &fixture.sweep, fixture.lease_ms, 1, true)
            .await?;
    let rest =
        draining::admission::claim_some(&fixture.store, &fixture.sweep, fixture.lease_ms, 4, false)
            .await?;
    let empty =
        draining::admission::claim_some(&fixture.store, &fixture.sweep, fixture.lease_ms, 4, true)
            .await?;
    assert_eq!(first.len(), 1);
    assert_eq!(
        first.first().and_then(|lease| lease.pending_work()),
        Some(2)
    );
    assert_eq!(rest.len(), 1);
    assert_ne!(
        first.first().map(|lease| lease.scope()),
        rest.first().map(|lease| lease.scope())
    );
    assert!(empty.is_empty());
    Ok(())
}

#[test]
fn given_pages_in_flight_when_a_claim_returns_nothing_then_progress_reports_an_empty_queue()
-> TestResult {
    let (sender, mut receiver) = tokio::sync::mpsc::channel(4);
    let progress = crate::control::Progress::new(Some(sender));
    let (depth, in_flight) = draining::admission::queue_report(None, 3, 0);
    progress.queue(depth, in_flight);
    let (depth, in_flight) = draining::admission::queue_report(Some(40), 3, 5);
    progress.queue(depth, in_flight);
    assert_eq!(
        receiver.try_recv()?,
        crate::control::ProgressEvent::Queue {
            pending: 0,
            in_flight: 3
        }
    );
    assert_eq!(
        receiver.try_recv()?,
        crate::control::ProgressEvent::Queue {
            pending: 32,
            in_flight: 8
        }
    );
    Ok(())
}
