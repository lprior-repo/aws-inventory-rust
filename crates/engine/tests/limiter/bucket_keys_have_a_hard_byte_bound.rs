use super::*;

#[tokio::test(start_paused = true)]
pub(super) async fn bucket_keys_have_a_hard_byte_bound() -> TestResult {
    let executor = ApiExecutor::with_seed(config(), 42)?;
    for key in [String::new(), "x".repeat(513)] {
        assert!(matches!(
            executor.execute(&key, || async { Ok(()) }).await,
            Err(ApiError::InvalidConfig(_))
        ));
    }
    executor
        .execute(&"x".repeat(512), || async { Ok(()) })
        .await?;
    assert_eq!(executor.stats().active_buckets, 1);
    assert_eq!(executor.stats().attempts, 1);
    Ok(())
}
