//! Behavior: validates_every_configuration_bound.
use super::*;

#[test]
pub(super) fn validates_every_configuration_bound() {
    let normal = config();
    let invalid = [
        ExecutorConfig {
            global_concurrency: 0,
            ..normal
        },
        ExecutorConfig {
            global_concurrency: 4_097,
            ..normal
        },
        ExecutorConfig {
            per_bucket_concurrency: 0,
            ..normal
        },
        ExecutorConfig {
            per_bucket_concurrency: 5,
            ..normal
        },
        ExecutorConfig {
            requests_per_second: 0,
            ..normal
        },
        ExecutorConfig {
            requests_per_second: 100_001,
            ..normal
        },
        ExecutorConfig {
            max_attempts: 0,
            ..normal
        },
        ExecutorConfig {
            max_attempts: 11,
            ..normal
        },
        ExecutorConfig {
            attempt_timeout_ms: 0,
            ..normal
        },
        ExecutorConfig {
            attempt_timeout_ms: 300_001,
            ..normal
        },
        ExecutorConfig {
            max_backoff_ms: 0,
            ..normal
        },
        ExecutorConfig {
            max_backoff_ms: 300_001,
            ..normal
        },
    ];
    for value in invalid {
        assert!(matches!(
            ApiExecutor::new(value),
            Err(ApiError::InvalidConfig(_))
        ));
    }
    assert!(ApiExecutor::new(ExecutorConfig::default()).is_ok());
}
