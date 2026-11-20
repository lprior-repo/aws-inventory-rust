//! Within a sweep the stalest scopes are claimed first (ROADMAP M7.1).
//! "Stale" is the time of a scope's latest complete outcome in any earlier sweep;
//! a scope that never completed is the stalest of all. Ties go to the smaller scope key.

use super::*;

const AGES: [(&str, i64); 3] = [("us-east-1", 300), ("us-east-2", 100), ("eu-west-1", 200)];

/// Complete one sweep over `AGES`, each scope observed at its own time.
async fn completed_first_sweep(store: &BinaryStore) -> TestResult {
    for (region, _) in AGES {
        store.seed_work(work("first", region)?).await?;
    }
    let sweep = SweepId::new("first".to_owned())?;
    for _ in AGES {
        let lease = store
            .claim(&sweep, 10, 1_000)
            .await?
            .ok_or("missing lease")?;
        let observed = AGES
            .iter()
            .find(|(region, _)| *region == lease.scope().region().as_str())
            .map(|(_, at)| *at)
            .ok_or("unexpected region")?;
        store
            .finish_scope(lease, ScopeOutcome::Complete, observed)
            .await?;
    }
    assert!(store.claim(&sweep, 20, 1_000).await?.is_none());
    Ok(())
}

async fn claim_regions(
    store: &BinaryStore,
    sweep: &SweepId,
    now: i64,
    lease_ms: i64,
    count: usize,
) -> TestResult<Vec<String>> {
    let mut regions = Vec::new();
    for _ in 0..count {
        let lease = store
            .claim(sweep, now, lease_ms)
            .await?
            .ok_or("missing lease")?;
        regions.push(lease.scope().region().as_str().to_owned());
    }
    Ok(regions)
}

#[tokio::test]
async fn given_earlier_outcomes_when_claiming_then_the_stalest_scope_goes_first() -> TestResult {
    let (_directory, store, _) = fixture().await?;
    completed_first_sweep(&store).await?;

    // ap-south-1 was never swept; the others completed at 100, 200 and 300.
    for region in ["us-east-1", "ap-south-1", "eu-west-1", "us-east-2"] {
        store.seed_work(work("second", region)?).await?;
    }
    let sweep = SweepId::new("second".to_owned())?;
    assert_eq!(
        claim_regions(&store, &sweep, 1_000, 1_000, 4).await?,
        ["ap-south-1", "us-east-2", "eu-west-1", "us-east-1"]
    );
    Ok(())
}

#[tokio::test]
async fn given_no_earlier_outcomes_when_claiming_then_scope_key_order_is_kept() -> TestResult {
    let (_directory, store, _) = fixture().await?;
    for region in ["us-east-2", "eu-west-1", "us-east-1"] {
        store.seed_work(work("only", region)?).await?;
    }
    let sweep = SweepId::new("only".to_owned())?;
    let claimed = claim_regions(&store, &sweep, 10, 1_000, 3).await?;
    let mut sorted = claimed.clone();
    sorted.sort();
    // Scope keys sort by account, then region, so ties resolve alphabetically.
    assert_eq!(claimed, sorted);
    Ok(())
}

#[tokio::test]
async fn given_only_an_incomplete_outcome_when_claiming_then_the_scope_is_still_never_completed()
-> TestResult {
    let (_directory, store, _) = fixture().await?;
    completed_first_sweep(&store).await?;
    // A later partial outcome at 900 does not make us-east-2 fresh: only complete counts.
    store.seed_work(work("partial", "us-east-2")?).await?;
    let partial = SweepId::new("partial".to_owned())?;
    let lease = store
        .claim(&partial, 800, 1_000)
        .await?
        .ok_or("missing lease")?;
    store
        .finish_scope(lease, ScopeOutcome::Inaccessible, 900)
        .await?;

    for region in ["us-east-1", "us-east-2", "eu-west-1"] {
        store.seed_work(work("third", region)?).await?;
    }
    let third = SweepId::new("third".to_owned())?;
    assert_eq!(
        claim_regions(&store, &third, 1_000, 1_000, 3).await?,
        ["us-east-2", "eu-west-1", "us-east-1"]
    );
    Ok(())
}

#[tokio::test]
async fn given_expired_leases_and_ready_work_when_claiming_then_the_stalest_of_both_goes_first()
-> TestResult {
    let (_directory, store, _) = fixture().await?;
    completed_first_sweep(&store).await?;

    for region in ["us-east-1", "us-east-2", "eu-west-1"] {
        store.seed_work(work("second", region)?).await?;
    }
    let sweep = SweepId::new("second".to_owned())?;
    // Three scopes are leased for 10 ms and abandoned, as a killed process would.
    assert_eq!(
        claim_regions(&store, &sweep, 1_000, 10, 3).await?,
        ["us-east-2", "eu-west-1", "us-east-1"]
    );
    // A never-completed scope arrives as new ready work after the lease expired.
    store.seed_work(work("second", "ap-south-1")?).await?;
    assert_eq!(
        claim_regions(&store, &sweep, 2_000, 1_000, 4).await?,
        ["ap-south-1", "us-east-2", "eu-west-1", "us-east-1"]
    );
    assert!(store.claim(&sweep, 2_000, 1_000).await?.is_none());
    Ok(())
}
