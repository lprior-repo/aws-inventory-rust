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

#[tokio::test]
async fn given_manifest_when_recreated_then_original_account_is_preserved() -> TestResult {
    let (_directory, store, _) = fixture().await?;
    assert_eq!(store.discover_active_sweep().await?, None);
    let sweep = SweepId::new("manifest-control".to_owned())?;
    store.create_sweep_manifest(&sweep, "123456789012").await?;
    store.create_sweep_manifest(&sweep, "111111111111").await?;
    store
        .seed_works(&[work(sweep.as_str(), "us-east-1")?])
        .await?;
    assert_eq!(store.discover_active_sweep().await?, Some(sweep.clone()));
    let other = AccountId::parse("111111111111")?;
    let account = AccountId::parse("123456789012")?;
    assert!(matches!(store.resume_sweep(&sweep, &other).await,
        Err(StoreError::AccountMismatch(expected, Some(found)))
            if expected == other && found == account));
    store.resume_sweep(&sweep, &account).await?;
    Ok(())
}
