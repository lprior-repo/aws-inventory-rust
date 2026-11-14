use super::*;

#[test]
pub(super) fn bucket_policies_validate_limits_and_exact_key_bounds() {
    for limits in [
        BucketLimits {
            requests_per_second: 0,
            concurrency: 1,
        },
        BucketLimits {
            requests_per_second: 100_001,
            concurrency: 1,
        },
        BucketLimits {
            requests_per_second: 1,
            concurrency: 0,
        },
        BucketLimits {
            requests_per_second: 1,
            concurrency: 5,
        },
    ] {
        assert!(matches!(
            ApiExecutor::with_bucket_policies(
                config(),
                BTreeMap::from([("account/region/service".into(), limits)]),
            ),
            Err(ApiError::InvalidConfig(_))
        ));
    }
    let limits = BucketLimits {
        requests_per_second: 1,
        concurrency: 1,
    };
    for key in [String::new(), "x".repeat(513)] {
        assert!(matches!(
            ApiExecutor::with_bucket_policies(config(), BTreeMap::from([(key, limits)])),
            Err(ApiError::InvalidConfig(_))
        ));
    }
    assert!(ApiExecutor::with_bucket_policies(config(), BTreeMap::new()).is_ok());
}
