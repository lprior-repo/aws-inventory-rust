use super::*;
use super::support::{Fixture,TestResult,page};
use inventory_core::inventory::ScopeOutcome;
use inventory_engine::limiter::ApiError;
use std::{sync::{Arc,atomic::{AtomicBool,Ordering}},time::Duration};
use tokio::sync::oneshot;

struct Dropped(Arc<AtomicBool>);
impl Drop for Dropped {
    fn drop(&mut self) { self.0.store(true,Ordering::SeqCst); }
}

#[tokio::test]
async fn given_admitted_sql_page_when_cancellation_arrives_then_supervisor_waits_for_commit_and_no_effects_follow_return() -> TestResult {
    let fixture = Fixture::new("cancel-admitted-commit").await?;
    let lease = fixture.seed().await?;
    let owned = vec![lease.clone()];
    let executor = ApiExecutor::new(executor_config(4)?)?;
    let committed = AtomicBool::new(false);
    let (admitted_tx,admitted_rx) = oneshot::channel();
    let (release_tx,release_rx) = oneshot::channel();
    let work = async {
        admitted_tx.send(()).map_err(|_| SweepError::Operation("admission observer lost"))?;
        release_rx.await.map_err(|_| SweepError::Operation("commit gate lost"))?;
        fixture.store.commit_page(page(lease,now_ms()?,None)).await?;
        committed.store(true,Ordering::SeqCst);
        Ok(SweepSummary::default())
    };
    let collection = draining::settle_batch(&fixture.store,&executor,owned,work);
    let signal = async {
        admitted_rx.await.map_err(std::io::Error::other)?;
        release_tx.send(()).map_err(|_| std::io::Error::other("commit waiter lost"))
    };
    let result = supervise(&executor,collection,signal).await;
    assert!(matches!(result,Err(SweepError::Api(ApiError::Cancelled))));
    assert!(committed.load(Ordering::SeqCst),"supervisor returned before admitted SQL completed");
    fixture.outcome(ScopeOutcome::Complete).await?;
    let before = fixture.store.sweep_summary(&fixture.sweep).await?;
    tokio::task::yield_now().await;
    assert_eq!(fixture.store.sweep_summary(&fixture.sweep).await?,before);
    Ok(())
}

#[tokio::test]
async fn given_stalled_admitted_page_when_cancel_drain_deadline_expires_then_owned_lease_is_finalized_before_return() -> TestResult {
    let fixture = Fixture::new("cancel-drain-deadline").await?;
    let lease = fixture.seed().await?;
    let owned = vec![lease.clone()];
    let executor = ApiExecutor::new(executor_config(4)?)?;
    let dropped = Arc::new(AtomicBool::new(false));
    let mark = dropped.clone();
    let (admitted_tx,admitted_rx) = oneshot::channel();
    let work = async move {
        let _lease = lease;
        let _guard = Dropped(mark);
        admitted_tx.send(()).map_err(|_| SweepError::Operation("admission observer lost"))?;
        std::future::pending::<Result<SweepSummary,SweepError>>().await
    };
    let collection = draining::settle_batch(&fixture.store,&executor,owned,work);
    let signal = async { admitted_rx.await.map_err(std::io::Error::other) };
    tokio::time::pause();
    let drive = async {
        executor.cancelled().await;
        tokio::time::advance(Duration::from_millis(LEASE_MARGIN_MS.checked_add(1).ok_or("test clock overflow")?)).await;
        tokio::time::resume();
        Ok::<(),Box<dyn std::error::Error>>(())
    };
    let (result,driven) = tokio::join!(supervise(&executor,collection,signal),drive);
    driven?;
    assert!(matches!(result,Err(SweepError::Api(ApiError::Exhausted(_)))));
    assert!(dropped.load(Ordering::SeqCst),"admitted page survived supervisor return");
    fixture.outcome(ScopeOutcome::Partial).await?;
    let before = fixture.store.sweep_summary(&fixture.sweep).await?;
    tokio::task::yield_now().await;
    assert_eq!(fixture.store.sweep_summary(&fixture.sweep).await?,before);
    Ok(())
}
