use super::*;

#[test]
pub(super) fn bucket_policy_map_has_a_hard_entry_bound() {
    let policies = (0..=MAX_BUCKETS)
        .map(|index| {
            (
                format!("bucket-{index}"),
                BucketLimits {
                    requests_per_second: 1,
                    concurrency: 1,
                },
            )
        })
        .collect();
    assert!(matches!(
        ApiExecutor::with_bucket_policies(config(), policies),
        Err(ApiError::InvalidConfig(_))
    ));
}
