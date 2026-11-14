use super::*;

#[test]
pub(super) fn rational_policy_validates_fractional_floor_ceiling_and_payload_bounds() {
    for policy in [
        period_policy(0, 1_000, 1),
        period_policy(100_001, 1_000, 1),
        period_policy(1, 0, 1),
        period_policy(1, MAX_RATE_PERIOD_MS.saturating_add(1), 1),
        period_policy(101, 1, 1),
        period_policy(1, 1_000, 0),
        period_policy(1, 1_000, 5),
        period_policy(u32::MAX, u64::MAX, usize::MAX),
    ] {
        assert!(matches!(
            ApiExecutor::with_rate_policies(
                config(),
                BTreeMap::from([("bounded".into(), policy),])
            ),
            Err(ApiError::InvalidConfig(_))
        ));
    }
    for policy in [
        period_policy(1, MAX_RATE_PERIOD_MS, 1),
        period_policy(100, 1, 1),
    ] {
        assert!(
            ApiExecutor::with_rate_policies(
                config(),
                BTreeMap::from([("bounded".into(), policy),])
            )
            .is_ok()
        );
    }
}
