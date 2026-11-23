use super::*;

#[tokio::test]
async fn given_active_sweep_for_wrong_account_when_discovered_then_identity_is_unchanged()
-> TestResult {
    let (_directory, store, _) = fixture().await?;
    let sweep = SweepId::new("work-1".to_owned())?;
    store.seed_works(&[work("work-1", "us-east-1")?]).await?;
    store.create_sweep_manifest(&sweep, "111111111111").await?;
    assert_eq!(store.discover_active_sweep().await?.as_ref(), Some(&sweep));
    let wrong = AccountId::parse("222222222222")?;
    assert!(matches!(
        store.resume_sweep(&sweep, &wrong).await,
        Err(StoreError::AccountMismatch(_, _))
    ));
    assert!(store.claim(&sweep, 100, 20).await?.is_some());
    Ok(())
}

#[tokio::test]
async fn given_duplicate_manifest_when_created_then_original_is_kept() -> TestResult {
    let (_directory, store, _) = fixture().await?;
    let sweep = SweepId::new("duplicate-manifest".to_owned())?;
    let queued = work("duplicate-manifest", "us-east-1")?;
    store.seed_works(&[queued]).await?;
    store.create_sweep_manifest(&sweep, "111111111111").await?;
    store.create_sweep_manifest(&sweep, "222222222222").await?;
    let manifest = store.sweep_manifest_account(&sweep).await?;
    assert_eq!(manifest, Some("111111111111".to_owned()));
    Ok(())
}
