use super::*;

#[test]
pub(super) fn rational_policy_maps_accept_the_boundary_and_never_truncate_overflow() {
    let at_limit = (0..MAX_BUCKETS)
        .map(|index| (format!("period-{index}"), period_policy(1, 1_000, 1)))
        .collect();
    assert!(ApiExecutor::with_rate_policies(config(), at_limit).is_ok());

    let too_many = (0..=MAX_BUCKETS)
        .map(|index| (format!("period-{index}"), period_policy(1, 1_000, 1)))
        .collect();
    assert!(matches!(
        ApiExecutor::with_rate_policies(config(), too_many),
        Err(ApiError::InvalidConfig(_))
    ));
}
