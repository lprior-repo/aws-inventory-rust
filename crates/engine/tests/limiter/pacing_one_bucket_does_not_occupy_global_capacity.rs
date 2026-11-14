use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn pacing_one_bucket_does_not_occupy_global_capacity() -> TestResult {
    let executor = ApiExecutor::with_seed(
        ExecutorConfig {
            global_concurrency: 1,
            per_bucket_concurrency: 1,
            requests_per_second: 1,
            ..config()
        },
        42,
    )?;
    executor.execute("slow", || async { Ok(()) }).await?;
    let beginning = Instant::now();
    let healthy = Mutex::new(Vec::new());
    let paced = executor.execute("slow", || async { Ok(()) });
    let unrelated = async {
        sleep(Duration::from_millis(10)).await;
        executor
            .execute("healthy", || async { record(&healthy) })
            .await
    };
    let (paced_result, healthy_result) = tokio::join!(paced, unrelated);
    paced_result?;
    healthy_result?;
    let observed = recorded(&healthy)?;
    assert_eq!(observed.len(), 1);
    assert!(
        observed
            .first()
            .is_some_and(|at| at.duration_since(beginning) < Duration::from_millis(20))
    );
    assert!(Instant::now().duration_since(beginning) >= Duration::from_secs(1));
    Ok(())
}
