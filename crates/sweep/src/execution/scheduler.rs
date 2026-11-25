use futures_util::{StreamExt, stream::FuturesUnordered};
use std::future::Future;

use super::{SweepError, SweepSummary, add_writes};

const MAX_CLAIM: u16 = 256;

#[derive(Default)]
struct ScheduleState {
    queue_empty: bool,
    first_error: Option<SweepError>,
    summary: SweepSummary,
}

impl ScheduleState {
    fn record_page(&mut self, result: Result<SweepSummary, SweepError>) {
        self.queue_empty = false;
        if self.first_error.is_none() {
            self.first_error = result
                .and_then(|page| add_writes(&mut self.summary, page))
                .err();
        }
    }
}

async fn try_claim<L, ClaimFut, PageFut>(
    in_flight: &mut FuturesUnordered<PageFut>,
    concurrency: usize,
    state: &mut ScheduleState,
    may_claim: &mut impl FnMut() -> bool,
    claim: &mut impl FnMut(u16) -> ClaimFut,
    process: &mut impl FnMut(L) -> PageFut,
    completed: &mut impl FnMut(usize),
) where
    ClaimFut: Future<Output = Result<Vec<L>, SweepError>>,
    PageFut: Future<Output = Result<SweepSummary, SweepError>>,
{
    while state.first_error.is_none()
        && !state.queue_empty
        && in_flight.len() < concurrency
        && may_claim()
    {
        let before = in_flight.len();
        let want = (concurrency.saturating_sub(before)).min(usize::from(MAX_CLAIM));
        let Ok(want) = u16::try_from(want) else {
            state.first_error = Some(SweepError::Operation("claim limit exceeds u16"));
            break;
        };
        let claim = claim(want);
        tokio::pin!(claim);
        let claimed = loop {
            tokio::select! {
                biased;
                result = &mut claim => break result,
                Some(result) = in_flight.next(), if !in_flight.is_empty() => {
                    state.record_page(result);
                    completed(in_flight.len());
                }
            }
        };
        match claimed {
            Err(error) => {
                if state.first_error.is_none() {
                    state.first_error = Some(error);
                }
            }
            Ok(leases) => {
                state.queue_empty = leases.is_empty() && in_flight.len() == before;
                for lease in leases {
                    in_flight.push(process(lease));
                }
            }
        }
    }
}

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
    let mut state = ScheduleState::default();

    loop {
        try_claim(
            &mut in_flight,
            concurrency,
            &mut state,
            &mut may_claim,
            &mut claim,
            &mut process,
            &mut completed,
        )
        .await;

        if in_flight.is_empty() {
            break;
        }

        if let Some(result) = in_flight.next().await {
            state.record_page(result);
            completed(in_flight.len());
        }
    }

    match state.first_error {
        Some(e) => Err(e),
        None => Ok(state.summary),
    }
}

#[cfg(test)]
#[path = "scheduler/tests.rs"]
mod tests;
#[cfg(test)]
#[path = "scheduler/writer_handoff.rs"]
mod writer_handoff;
