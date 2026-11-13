//! Behavior: global_contention_cannot_release_accumulated_rate_tickets_as_a_burst.
use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn global_contention_cannot_release_accumulated_rate_tickets_as_a_burst()
-> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 2,
            per_bucket_concurrency: 2,
            ..config()
        },
        42,
    )?;
    let barrier = Barrier::new(3);
    let starts = Mutex::new(Vec::new());
    let blocker_a = executor.execute("a/region/s3", || async {
        barrier.wait().await;
        sleep(Duration::from_secs(1)).await;
        Ok(())
    });
    let blocker_b = executor.execute("b/region/s3", || async {
        barrier.wait().await;
        sleep(Duration::from_secs(1)).await;
        Ok(())
    });
    let waiting = async {
        barrier.wait().await;
        stream::iter(0..4)
            .map(|_| executor.execute("c/region/ec2", || async { record(&starts) }))
            .buffer_unordered(4)
            .try_collect::<Vec<_>>()
            .await
    };
    bounded_barrier_case(async { tokio::try_join!(blocker_a, blocker_b, waiting) }).await?;
    let observed = recorded(&starts)?;
    assert_eq!(observed.len(), 4);
    spaced(&observed, Duration::from_millis(100));
    assert_eq!(executor.stats().peak_in_flight, 2);
    Ok(())
}
