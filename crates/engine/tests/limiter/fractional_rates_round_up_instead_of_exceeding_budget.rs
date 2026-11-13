//! Behavior: fractional_rates_round_up_instead_of_exceeding_budget.
use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn fractional_rates_round_up_instead_of_exceeding_budget() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            requests_per_second: 3,
            ..config()
        },
        42,
    )?;
    let starts = Mutex::new(Vec::new());
    for _ in 0..4 {
        executor
            .execute("account/region/lambda", || async { record(&starts) })
            .await?;
    }
    spaced(&recorded(&starts)?, Duration::from_nanos(333_333_334));
    Ok(())
}
