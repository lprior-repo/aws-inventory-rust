use inventory_engine::inventory::SweepId;

use super::support::*;

#[tokio::test]
async fn given_root_work_batch_when_seeded_repeatedly_then_each_scope_is_claimed_once() -> TestResult
{
    let (_directory, store, _) = fixture().await?;
    let batch = [
        work("root-batch", "us-east-1")?,
        work("root-batch", "us-west-2")?,
    ];
    store.seed_works(&batch).await?;
    store.seed_works(&batch).await?;
    let sweep = SweepId::new("root-batch".to_owned())?;

    assert!(store.claim(&sweep, 100, 20).await?.is_some());
    assert!(store.claim(&sweep, 100, 20).await?.is_some());
    assert!(store.claim(&sweep, 100, 20).await?.is_none());
    Ok(())
}

#[tokio::test]
async fn given_a_mixed_sweep_batch_when_seeded_then_no_work_is_inserted() -> TestResult {
    let (_directory, store, _) = fixture().await?;
    let batch = [work("mixed-a", "us-east-1")?, work("mixed-b", "us-west-2")?];

    assert!(store.seed_works(&batch).await.is_err());
    assert!(
        store
            .claim(&SweepId::new("mixed-a".to_owned())?, 100, 20)
            .await?
            .is_none()
    );
    Ok(())
}
