use super::*;
use inventory_engine::inventory::WorkLease;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

/// Claims up to `limit` leases; `limit` is at least one.
///
/// With `with_depth` the first lease also carries the queue depth for progress
/// and the rest come from one batch claim. A failed batch claim releases that
/// first lease before the error is returned.
pub(in crate::execution) async fn claim_some(
    store: &impl InventoryStore,
    sweep: &SweepId,
    lease_ms: i64,
    limit: u16,
    with_depth: bool,
) -> Result<Vec<WorkLease>, SweepError> {
    let now = now_ms()?;
    if !with_depth {
        return Ok(store.claim_batch(sweep, now, lease_ms, limit).await?);
    }
    let Some(first) = store.claim_with_depth(sweep, now, lease_ms).await? else {
        return Ok(Vec::new());
    };
    let rest = limit.saturating_sub(1);
    if rest == 0 {
        return Ok(vec![first]);
    }
    match store.claim_batch(sweep, now, lease_ms, rest).await {
        Ok(batch) => Ok(std::iter::once(first).chain(batch).collect()),
        Err(error) => {
            super::settle_owned(store, std::iter::once(first)).await?;
            Err(error.into())
        }
    }
}

/// Queue depth and pages in flight to report after one claim.
///
/// A claim that returned leases reports the depth its first lease carried. A
/// claim that returned nothing reports an empty queue holding only the pages
/// still in flight, so progress keeps moving while the last pages finish.
/// With nothing claimed and nothing in flight there is nothing to report: the
/// drain is over and the finished event says so.
pub(in crate::execution) fn queue_report(
    depth: Option<u64>,
    in_flight: usize,
    claimed: usize,
) -> (Option<u64>, usize) {
    if claimed == 0 {
        let depth = (in_flight != 0).then_some(in_flight);
        return (depth.and_then(|pages| u64::try_from(pages).ok()), in_flight);
    }
    (depth, in_flight.saturating_add(claimed))
}

/// Leases this drain claimed whose page has not yet returned success.
///
/// A cancelled drain releases exactly these, so the set stays bounded by the
/// pages in flight plus the pages that failed.
#[derive(Default)]
pub(in crate::execution) struct Owned {
    leases: RefCell<BTreeMap<u64, WorkLease>>,
    next: Cell<u64>,
    in_flight: Cell<usize>,
}

impl Owned {
    pub(in crate::execution) fn admit(&self, lease: &WorkLease) -> u64 {
        let id = self.next.get();
        self.next.set(id.wrapping_add(1));
        self.leases.borrow_mut().insert(id, lease.clone());
        self.in_flight.set(self.in_flight.get().saturating_add(1));
        id
    }

    /// Runs one admitted page and forgets its lease once the page succeeds.
    pub(in crate::execution) async fn settle(
        &self,
        id: u64,
        page: impl Future<Output = Result<SweepSummary, SweepError>>,
    ) -> Result<SweepSummary, SweepError> {
        let result = page.await;
        if result.is_ok() {
            self.leases.borrow_mut().remove(&id);
        }
        result
    }

    pub(in crate::execution) fn set_in_flight(&self, in_flight: usize) {
        self.in_flight.set(in_flight);
    }

    pub(in crate::execution) fn in_flight(&self) -> usize {
        self.in_flight.get()
    }

    pub(in crate::execution) fn take(&self) -> Vec<WorkLease> {
        std::mem::take(&mut *self.leases.borrow_mut())
            .into_values()
            .collect()
    }
}
