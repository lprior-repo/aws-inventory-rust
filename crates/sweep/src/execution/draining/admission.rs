use super::*;
use inventory_engine::inventory::WorkLease;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

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

pub(in crate::execution) const MAX_PAGES_PER_CLAIM: u16 = 64;

#[derive(Default)]
pub(in crate::execution) struct Owned {
    leases: Mutex<BTreeMap<u64, WorkLease>>,
    next: AtomicU64,
    in_flight: AtomicUsize,
}

impl Owned {
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
