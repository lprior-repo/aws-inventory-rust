//! Behavior: rational_policy_json_preserves_integer_shape_and_rejects_mixed_or_unknown_fields.
use super::*;

#[test]
pub(super) fn rational_policy_json_preserves_integer_shape_and_rejects_mixed_or_unknown_fields()
-> TestResult {
    let integer =
        serde_json::from_str::<BucketPolicy>(r#"{"requests_per_second":5,"concurrency":2}"#)?;
    assert_eq!(
        integer,
        BucketPolicy::PerSecond(BucketLimits {
            requests_per_second: 5,
            concurrency: 2
        })
    );
    let fractional =
        serde_json::from_str::<BucketPolicy>(r#"{"requests":1,"period_ms":2000,"concurrency":2}"#)?;
    assert_eq!(fractional, period_policy(1, 2_000, 2));
    for text in [
        r#"{"requests_per_second":5,"requests":1,"period_ms":2000,"concurrency":2}"#,
        r#"{"requests":1,"period_ms":2000,"concurrency":2,"period_seconds":2}"#,
        r#"{"requests_per_second":5,"concurrency":2,"concurency":2}"#,
        r#"{"requests_per_second":0.5,"concurrency":2}"#,
        r#"{"requests":1.5,"period_ms":2000,"concurrency":2}"#,
    ] {
        assert!(serde_json::from_str::<BucketPolicy>(text).is_err());
    }
    Ok(())
}
