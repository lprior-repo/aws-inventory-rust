use super::{Bucket, RatePolicy};
use crate::proxy::ProxyError;
use std::{num::NonZeroU32, time::Duration};
use tokio::time::Instant;

#[tokio::test(start_paused = true)]
async fn stale_instant_cannot_rewind_bucket_or_mint_another_refill() -> Result<(), ProxyError> {
    // Given an exhausted bucket that already consumed its one-second refill.
    let start = Instant::now();
    let later = start
        .checked_add(Duration::from_secs(1))
        .ok_or(ProxyError::Timeout)?;
    let rate = NonZeroU32::MIN;
    let mut bucket = Bucket::new(RatePolicy::Limited(rate), start)?;
    assert!(bucket.admit(rate, start)?);
    assert!(bucket.admit(rate, later)?);
    // When a preempted caller presents an older instant, no second refill exists.
    assert!(!bucket.admit(rate, start)?);
    assert_eq!(bucket.updated, later);
    assert!(!bucket.admit(rate, later)?);
    Ok(())
}
