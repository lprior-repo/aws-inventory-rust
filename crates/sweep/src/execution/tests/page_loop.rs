use super::support::{Fixture, TestResult};
use super::*;
use draining::admission::{MAX_PAGES_PER_CLAIM, Owned};
use inventory_core::inventory::ScopeOutcome;
use inventory_engine::inventory::{PageCommit, WorkLease};
use std::cell::{Cell, RefCell};

struct Pager<'a> {
    fixture: &'a Fixture,
    pages: u16,
    fail_at: Option<u16>,
    calls: Cell<u16>,
    keeps: RefCell<Vec<bool>>,
}

impl<'a> Pager<'a> {
    fn new(fixture: &'a Fixture, pages: u16, fail_at: Option<u16>) -> Self {
        Self {
            fixture,
            pages,
            fail_at,
            calls: Cell::new(0),
            keeps: RefCell::new(Vec::new()),
        }
    }

    fn commit(&self, lease: WorkLease, call: u16) -> PageCommit {
        let byte = u8::try_from(call).unwrap_or(u8::MAX);
        let next = (call < self.pages).then(|| vec![byte]);
        PageCommit {
            lease,
            observations: Vec::new(),
            edges: Vec::new(),
            unresolved: Vec::new(),
            tags: Vec::new(),
            continuation_digest: next.as_ref().map(|_| [byte; 32]),
            done: next.is_none(),
            next_token: next,
            children: Vec::new(),
            observed_at_ms: self.fixture.now.saturating_add(i64::from(call)),
        }
    }

    async fn fetch(&self, lease: WorkLease, keep: bool) -> Result<page::Paged, SweepError> {
        let call = self.calls.get().saturating_add(1);
        self.calls.set(call);
        self.keeps.borrow_mut().push(keep);
        if self.fail_at == Some(call) {
            return Err(SweepError::Operation("page fault"));
        }
        let commit = self.commit(lease, call);
        let store = &self.fixture.store;
        let next = if keep {
            let (_, next) = store
                .commit_page_keeping(commit, self.fixture.lease_ms)
                .await?;
            next
        } else {
            store.commit_page(commit).await?;
            None
        };
        let summary = SweepSummary {
            resource_writes: 1,
            ..SweepSummary::default()
        };
        Ok(page::Paged { summary, next })
    }
}

async fn settle(
    owned: &Owned,
    pager: &Pager<'_>,
    may_keep: bool,
) -> TestResult<Result<SweepSummary, SweepError>> {
    let lease = pager.fixture.seed().await?;
    let id = owned.admit(&lease);
    Ok(owned
        .settle(
            id,
            lease,
            || may_keep,
            |lease, keep| pager.fetch(lease, keep),
        )
        .await)
}

async fn reclaim(fixture: &Fixture) -> TestResult<WorkLease> {
    Ok(fixture
        .store
        .claim(&fixture.sweep, now_ms()?, fixture.lease_ms)
        .await?
        .ok_or("the scope did not return to the queue")?)
}

#[tokio::test]
async fn given_three_pages_when_one_claim_is_settled_then_all_three_are_fetched_and_the_scope_completes()
-> TestResult {
    let fixture = Fixture::new("page-loop-three").await?;
    let pager = Pager::new(&fixture, 3, None);
    let owned = Owned::default();
    let total = settle(&owned, &pager, true).await??;
    assert_eq!(pager.calls.get(), 3);
    assert_eq!(total.resource_writes, 3);
    assert!(owned.take().is_empty());
    fixture.outcome(ScopeOutcome::Complete).await
}

#[tokio::test]
async fn given_an_endless_scope_when_one_claim_is_settled_then_it_stops_at_the_page_bound_and_requeues()
-> TestResult {
    let fixture = Fixture::new("page-loop-bound").await?;
    let pager = Pager::new(&fixture, u16::MAX, None);
    let owned = Owned::default();
    let total = settle(&owned, &pager, true).await??;
    assert_eq!(pager.calls.get(), MAX_PAGES_PER_CLAIM);
    assert_eq!(total.resource_writes, u64::from(MAX_PAGES_PER_CLAIM));
    let keeps = pager.keeps.take();
    assert_eq!(keeps.last(), Some(&false));
    assert_eq!(keeps.iter().filter(|keep| **keep).count(), 63);
    assert!(owned.take().is_empty());
    let resumed = reclaim(&fixture).await?;
    assert_eq!(resumed.token(), Some([64_u8].as_slice()));
    Ok(())
}

#[tokio::test]
async fn given_the_drain_may_not_claim_when_a_page_has_a_successor_then_the_lease_is_not_kept()
-> TestResult {
    let fixture = Fixture::new("page-loop-refused").await?;
    let pager = Pager::new(&fixture, 3, None);
    let owned = Owned::default();
    settle(&owned, &pager, false).await??;
    assert_eq!(*pager.keeps.borrow(), vec![false]);
    assert!(owned.take().is_empty());
    let resumed = reclaim(&fixture).await?;
    assert_eq!(resumed.token(), Some([1_u8].as_slice()));
    Ok(())
}

#[tokio::test]
async fn given_a_kept_lease_when_the_next_page_fails_then_the_kept_lease_is_the_one_still_owned()
-> TestResult {
    let fixture = Fixture::new("page-loop-fault").await?;
    let pager = Pager::new(&fixture, 3, Some(2));
    let owned = Owned::default();
    let result = settle(&owned, &pager, true).await?;
    assert!(matches!(result, Err(SweepError::Operation("page fault"))));
    let mut left = owned.take();
    assert_eq!(left.len(), 1);
    let kept = left.pop().ok_or("no lease left owned")?;
    assert_eq!(kept.token(), Some([1_u8].as_slice()));
    fixture.store.release_lease(kept).await?;
    let resumed = reclaim(&fixture).await?;
    assert_eq!(resumed.token(), Some([1_u8].as_slice()));
    Ok(())
}
