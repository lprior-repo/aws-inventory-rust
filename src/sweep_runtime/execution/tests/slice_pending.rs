use super::support::{Fixture, TestResult, page};
use super::*;

#[tokio::test]
async fn given_queued_work_when_slice_is_exhausted_then_summary_reports_it_as_remaining() -> TestResult {
    let fixture = Fixture::new("slice-remaining").await?;
    fixture.seed().await?;
    let mut summary = SweepSummary::default();
    draining::apply_persisted_summary(&fixture.store, &fixture.sweep, &mut summary, true).await?;
    assert_eq!(summary.work_remaining, 1);
    assert_eq!(summary.outcomes.complete, 0);
    Ok(())
}

#[tokio::test]
async fn given_queued_work_when_slice_was_not_exhausted_then_pending_work_is_an_error() -> TestResult {
    let fixture = Fixture::new("slice-unexpected-pending").await?;
    fixture.seed().await?;
    let mut summary = SweepSummary::default();
    let result =
        draining::apply_persisted_summary(&fixture.store, &fixture.sweep, &mut summary, false)
            .await;
    assert!(matches!(result, Err(SweepError::Operation(_))));
    assert_eq!(summary.work_remaining, 0);
    Ok(())
}

#[tokio::test]
async fn given_drained_sweep_when_slice_budget_also_expired_then_nothing_is_remaining() -> TestResult {
    let fixture = Fixture::new("slice-drained").await?;
    let lease = fixture.seed().await?;
    fixture
        .store
        .commit_page(page(lease, fixture.now, None))
        .await?;
    let mut summary = SweepSummary::default();
    draining::apply_persisted_summary(&fixture.store, &fixture.sweep, &mut summary, true).await?;
    assert_eq!(summary.work_remaining, 0);
    assert_eq!(summary.outcomes.complete, 1);
    Ok(())
}
