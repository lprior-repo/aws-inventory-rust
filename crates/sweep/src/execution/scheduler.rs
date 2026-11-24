//! Generic lease-claim / page-processing scheduler.
//! Keeps at most `concurrency` page futures in flight, claiming more work
//! as pages complete.

use futures_util::{StreamExt, stream::FuturesUnordered};
use std::future::Future;

use super::{SweepError, SweepSummary, add_writes};

/// Most leases one claim may ask for.
pub(super) const MAX_CLAIM: usize = 256;

async fn try_claim<L, ClaimFut, PageFut>(
    in_flight: &mut FuturesUnordered<PageFut>,
    concurrency: usize,
    queue_empty: &mut bool,
    first_error: &mut Option<SweepError>,
    may_claim: &mut impl FnMut() -> bool,
    claim: &mut impl FnMut(u16) -> ClaimFut,
    process: &mut impl FnMut(L) -> PageFut,
) where
    ClaimFut: Future<Output = Result<Vec<L>, SweepError>>,
    PageFut: Future<Output = Result<SweepSummary, SweepError>>,
{
    while first_error.is_none() && !*queue_empty && in_flight.len() < concurrency && may_claim() {
        let want = (concurrency.saturating_sub(in_flight.len())).min(MAX_CLAIM);
        let want = u16::try_from(want).unwrap_or(256);
        match claim(want).await {
            Err(e) => {
                if first_error.is_none() {
                    *first_error = Some(e);
                }
            }
            Ok(leases) => {
                if leases.is_empty() {
                    *queue_empty = true;
                }
                for lease in leases {
                    in_flight.push(process(lease));
                }
            }
        }
    }
}

/// Schedule lease claims and page processing at bounded concurrency.
/// Returns when the queue is empty and all in-flight pages have finished.
/// Errors are deferred: once any error is observed no new work is claimed,
/// but every already-admitted page is still polled to completion.
pub(super) async fn run<L, ClaimFut, PageFut>(
    concurrency: usize,
    mut may_claim: impl FnMut() -> bool,
    mut claim: impl FnMut(u16) -> ClaimFut,
    mut process: impl FnMut(L) -> PageFut,
    mut completed: impl FnMut(usize),
) -> Result<SweepSummary, SweepError>
where
    ClaimFut: Future<Output = Result<Vec<L>, SweepError>>,
    PageFut: Future<Output = Result<SweepSummary, SweepError>>,
{
    let mut in_flight = FuturesUnordered::new();
    let mut summary = SweepSummary::default();
    let mut first_error: Option<SweepError> = None;
    let mut queue_empty = false;

    loop {
        try_claim(
            &mut in_flight,
            concurrency,
            &mut queue_empty,
            &mut first_error,
            &mut may_claim,
            &mut claim,
            &mut process,
        )
        .await;

        if in_flight.is_empty() {
            break;
        }

        if let Some(result) = in_flight.next().await {
            match result {
                Ok(page) => {
                    if first_error.is_none()
                        && let Err(e) = add_writes(&mut summary, page)
                    {
                        first_error = Some(e);
                    }
                }
                Err(e) => {
                    if first_error.is_none() {
                        first_error = Some(e);
                    }
                }
            }
            queue_empty = false;
            completed(in_flight.len());
        }
    }

    match first_error {
        Some(e) => Err(e),
        None => Ok(summary),
    }
}

#[cfg(test)]
#[path = "scheduler/tests.rs"]
mod tests;
