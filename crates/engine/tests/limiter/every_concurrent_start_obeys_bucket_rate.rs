use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn every_concurrent_start_obeys_bucket_rate() -> TestResult {
    let executor = ApiExecutor::with_seed(config(), 42)?;
    let starts = Mutex::new(Vec::new());
    let results = stream::iter(0..6)
        .map(|_| executor.execute("account/region/ec2", || async { record(&starts) }))
        .buffer_unordered(6)
        .collect::<Vec<_>>()
        .await;
    for result in results {
        result?;
    }
    let observed = recorded(&starts)?;
    assert_eq!(observed.len(), 6);
    spaced(&observed, Duration::from_millis(100));
    assert_eq!(executor.stats().attempts, 6);
    assert_eq!(executor.stats().successes, 6);
    assert_eq!(executor.stats().in_flight, 0);
    Ok(())
}
