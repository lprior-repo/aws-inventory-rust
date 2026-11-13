//! Behavior: bucket_policy_json_rejects_unknown_fields.
use super::*;

#[test]
pub(super) fn bucket_policy_json_rejects_unknown_fields() {
    let invalid = r#"{"requests_per_second":10,"concurrency":2,"concurency":3}"#;
    assert!(serde_json::from_str::<BucketLimits>(invalid).is_err());
}
