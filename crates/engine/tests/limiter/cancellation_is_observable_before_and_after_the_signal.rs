//! Behavior: cancellation_is_observable_before_and_after_the_signal.
use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn cancellation_is_observable_before_and_after_the_signal() -> TestResult {
    let executor = ApiExecutor::new(config())?;
    assert!(!executor.is_cancelled());
    let beginning = Instant::now();
    let observer = async {
        executor.cancelled().await;
        Instant::now()
    };
    let trigger = async {
        sleep(Duration::from_millis(10)).await;
        executor.cancel();
    };
    let (observed, ()) = tokio::join!(observer, trigger);
    assert!(observed.duration_since(beginning) >= Duration::from_millis(10));
    assert!(observed.duration_since(beginning) <= Duration::from_millis(12));
    assert!(executor.is_cancelled());
    let signalled = Instant::now();
    timeout(Duration::from_millis(1), executor.cancelled()).await?;
    assert_eq!(Instant::now(), signalled);
    assert_eq!(executor.stats().attempts, 0);
    Ok(())
}
