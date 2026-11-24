use super::*;
use inventory_engine::inventory::WorkLease;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

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

/// Pages one claim may fetch before its lease returns to the queue, so a long
/// scope cannot hold a worker while other scopes wait.
pub(in crate::execution) const MAX_PAGES_PER_CLAIM: u16 = 64;

/// Leases this drain claimed whose page has not yet returned success.
///
/// A cancelled drain releases exactly these, so the set stays bounded by the
/// pages in flight plus the pages that failed. The scheduler's futures share
/// this state by reference, so it must be `Sync` for the sweep's future to be
/// `Send`.
#[derive(Default)]
pub(in crate::execution) struct Owned {
    leases: Mutex<BTreeMap<u64, WorkLease>>,
    next: AtomicU64,
    in_flight: AtomicUsize,
}

impl Owned {
    /// The lock is never held across an await and the map stays valid if a
    /// holder panics, so a poisoned lock is still usable.
    fn leases(&self) -> MutexGuard<'_, BTreeMap<u64, WorkLease>> {
        self.leases.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(in crate::execution) fn admit(&self, lease: &WorkLease) -> u64 {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        self.leases().insert(id, lease.clone());
        let in_flight = self.in_flight.load(Ordering::Relaxed);
        self.in_flight
            .store(in_flight.saturating_add(1), Ordering::Relaxed);
        id
    }

    /// Runs an admitted scope page by page while each page keeps the lease.
    ///
    /// `page(lease, keep)` processes one page; `keep` is false for the last
    /// page this claim may fetch and whenever `may_keep` refuses. The lease a
    /// page kept replaces the owned one, and a failed page leaves its lease
    /// owned, so a cancelled drain releases the lease that is still valid.
    pub(in crate::execution) async fn settle<F>(
        &self,
        id: u64,
        lease: WorkLease,
        may_keep: impl Fn() -> bool,
        mut page: impl FnMut(WorkLease, bool) -> F,
    ) -> Result<SweepSummary, SweepError>
    where
        F: Future<Output = Result<crate::execution::page::Paged, SweepError>>,
    {
        let mut total = SweepSummary::default();
        let mut lease = lease;
        for fetched in 1..=MAX_PAGES_PER_CLAIM {
            let keep = fetched < MAX_PAGES_PER_CLAIM && may_keep();
            let paged = page(lease, keep).await?;
            self.track(id, paged.next.as_ref());
            add_writes(&mut total, paged.summary)?;
            let Some(next) = paged.next else { break };
            lease = next;
        }
        Ok(total)
    }

    /// Owns the lease a page kept, or forgets the scope when it kept none.
    fn track(&self, id: u64, kept: Option<&WorkLease>) {
        let mut leases = self.leases();
        match kept {
            Some(lease) => leases.insert(id, lease.clone()),
            None => leases.remove(&id),
        };
    }

    pub(in crate::execution) fn set_in_flight(&self, in_flight: usize) {
        self.in_flight.store(in_flight, Ordering::Relaxed);
    }

    pub(in crate::execution) fn in_flight(&self) -> usize {
        self.in_flight.load(Ordering::Relaxed)
    }

    pub(in crate::execution) fn take(&self) -> Vec<WorkLease> {
        std::mem::take(&mut *self.leases()).into_values().collect()
    }
}
