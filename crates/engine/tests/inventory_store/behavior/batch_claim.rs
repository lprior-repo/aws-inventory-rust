//! `claim_batch` on a store that keeps the trait's default: one `claim` per lease.

use super::*;
use inventory_engine::inventory::InventoryStore;

async fn batch_regions(
    store: &BinaryStore,
    sweep: &SweepId,
    limit: u16,
) -> TestResult<Vec<String>> {
    let leases = store.claim_batch(sweep, 100, 10_000, limit).await?;
    Ok(leases
        .iter()
        .map(|lease| lease.scope().region().as_str().to_owned())
        .collect())
}

#[tokio::test]
async fn given_three_ready_scopes_when_claiming_two_then_the_smaller_keys_come_first_and_one_remains()
-> TestResult {
    let (_directory, store, _) = fixture().await?;
    for region in ["us-west-2", "eu-west-1", "us-east-1"] {
        store.seed_work(work("batch", region)?).await?;
    }
    let sweep = SweepId::new("batch".to_owned())?;

    assert_eq!(
        batch_regions(&store, &sweep, 2).await?,
        ["eu-west-1", "us-east-1"]
    );
    assert_eq!(batch_regions(&store, &sweep, 2).await?, ["us-west-2"]);
    assert!(batch_regions(&store, &sweep, 2).await?.is_empty());
    Ok(())
}

#[tokio::test]
async fn given_a_zero_limit_when_claiming_a_batch_then_it_is_refused_and_the_work_stays_ready()
-> TestResult {
    let (_directory, store, _) = fixture().await?;
    store.seed_work(work("zero", "us-east-1")?).await?;
    let sweep = SweepId::new("zero".to_owned())?;

    let refused = store.claim_batch(&sweep, 100, 10_000, 0).await;
    assert!(matches!(refused, Err(StoreError::Invalid(_))));
    assert_eq!(batch_regions(&store, &sweep, 1).await?, ["us-east-1"]);
    Ok(())
}
