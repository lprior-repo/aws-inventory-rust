use proptest::prelude::*;

use super::*;

#[test]
fn given_new_when_no_call_then_initial_state_is_correct() -> Result<(), Box<dyn std::error::Error>>
{
    // Given
    let rate = Seeking::new(10, 1, 100);

    // When (none)

    // Then
    assert_eq!(rate.current_mrps, 10_000);
    assert_eq!(rate.floor_mrps, 1_000);
    assert_eq!(rate.ceiling_mrps, 100_000);
    assert_eq!(rate.hold, None);
    let interval = rate
        .interval_ns()
        .map_err(|e| format!("interval_ns: {e}"))?;
    assert_eq!(interval, 100_000_000);
    Ok(())
}

#[test]
fn given_new_when_recover_then_rate_rises() -> Result<(), Box<dyn std::error::Error>> {
    // Given
    let mut rate = Seeking::new(10, 1, 100);
    let t0 = Instant::now();

    // When
    rate.recover(t0);

    // Then
    assert_eq!(rate.current_mrps, 10_100);
    Ok(())
}

#[test]
fn given_minimum_start_when_recover_then_rate_doubles() -> Result<(), Box<dyn std::error::Error>> {
    // Given
    let mut rate = Seeking::new(1, 1, 100);
    let t0 = Instant::now();

    // When
    rate.recover(t0);

    // Then
    assert_eq!(rate.current_mrps, 2_000);
    Ok(())
}

#[test]
fn given_start_equals_ceiling_when_recover_then_rate_stays_at_ceiling()
-> Result<(), Box<dyn std::error::Error>> {
    // Given
    let mut rate = Seeking::new(10, 1, 10);
    let t0 = Instant::now();

    // When
    rate.recover(t0);

    // Then
    assert_eq!(rate.current_mrps, 10_000);
    Ok(())
}

#[test]
fn given_new_when_throttle_then_rate_drops() -> Result<(), Box<dyn std::error::Error>> {
    // Given
    let mut rate = Seeking::new(10, 1, 100);
    let t0 = Instant::now();

    // When
    rate.throttle(t0);

    // Then
    assert_eq!(rate.current_mrps, 7_000);
    let hold = rate.hold.ok_or("hold should be Some after throttle")?;
    assert_eq!(hold.throttled_mrps, 10_000);
    assert_eq!(hold.since, t0);
    let interval = rate
        .interval_ns()
        .map_err(|e| format!("interval_ns: {e}"))?;
    assert_eq!(interval, 142_857_143);
    Ok(())
}

#[test]
fn given_minimum_start_when_throttle_then_rate_stays_at_floor()
-> Result<(), Box<dyn std::error::Error>> {
    // Given
    let mut rate = Seeking::new(1, 1, 100);
    let t0 = Instant::now();

    // When
    rate.throttle(t0);

    // Then
    assert_eq!(rate.current_mrps, 1_000);
    let hold = rate.hold.ok_or("hold should be Some after throttle")?;
    assert_eq!(hold.throttled_mrps, 1_000);
    Ok(())
}

#[test]
fn given_throttled_when_repeated_recover_then_hold_limits_and_expires()
-> Result<(), Box<dyn std::error::Error>> {
    // Given
    let mut rate = Seeking::new(10, 1, 100);
    let t0 = Instant::now();
    rate.throttle(t0);

    let t_plus_1s = t0
        .checked_add(Duration::from_secs(1))
        .ok_or("clock overflow")?;

    // When: 100 recover calls at t0 + 1s
    for _ in 0..100 {
        rate.recover(t_plus_1s);
    }

    // Then: current should be 9_000 (limited by hold)
    assert_eq!(rate.current_mrps, 9_000);
    assert!(rate.hold.is_some());

    // When: recover at t0 + 59999ms (still within hold)
    let t_plus_59999ms = t0
        .checked_add(Duration::from_millis(59_999))
        .ok_or("clock overflow")?;
    rate.recover(t_plus_59999ms);

    // Then: still at 9_000, hold still active
    assert_eq!(rate.current_mrps, 9_000);
    assert!(rate.hold.is_some());

    // When: recover at t0 + 60s (hold expired)
    let t_plus_60s = t0
        .checked_add(Duration::from_secs(60))
        .ok_or("clock overflow")?;
    rate.recover(t_plus_60s);

    // Then: hold cleared, rate rises above 9_000
    assert!(rate.hold.is_none());
    assert_eq!(rate.current_mrps, 9_111);
    Ok(())
}

#[test]
fn given_high_rate_when_throttle_then_step_is_raised_to_one()
-> Result<(), Box<dyn std::error::Error>> {
    // Given
    let mut rate = Seeking::new(100_000, 1, 100_000);
    let t0 = Instant::now();
    rate.throttle(t0);

    // When: recover after hold expires
    let t_plus_60s = t0
        .checked_add(Duration::from_secs(60))
        .ok_or("clock overflow")?;
    rate.recover(t_plus_60s);

    // Then: step is 1 (because 1_000_000 / 70_000_000 is 0)
    assert!(rate.hold.is_none());
    assert_eq!(rate.current_mrps, 70_000_001);
    Ok(())
}

proptest! {
    #[test]
    fn given_any_bounds_when_any_calls_are_applied_then_the_rate_stays_within_them(
        mut bounds in proptest::array::uniform3(1_u32..=u32::MAX),
        throttles in proptest::collection::vec(any::<bool>(), 0..200),
    ) {
        bounds.sort_unstable();
        let [floor, start, ceiling] = bounds;
        let mut rate = Seeking::new(start, floor, ceiling);
        let now = Instant::now();
        for throttle in throttles {
            if throttle {
                rate.throttle(now);
            } else {
                rate.recover(now);
            }
            prop_assert!(rate.current_mrps >= rate.floor_mrps);
            prop_assert!(rate.current_mrps <= rate.ceiling_mrps);
            prop_assert!(rate.interval_ns().is_ok_and(|interval| interval >= 1));
        }
    }
}
