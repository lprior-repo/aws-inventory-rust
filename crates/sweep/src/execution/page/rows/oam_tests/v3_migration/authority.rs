use super::*;
#[tokio::test]
async fn given_pre_v3_active_lease_when_migration_keeps_its_generation_then_commit_and_complete_are_stale_until_reclaimed()
-> TestResult {
    let bytes = catalog_bytes()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, original) = store(&catalog).await?;
    let old = page(&original, &catalog, "metadata", "old-metadata", 1, true).await?;
    let reference = fixture::historical_evidence(
        &catalog,
        old.unresolved
            .first()
            .ok_or("native OAM reference absent")?,
    )?;
    original.commit_page(old).await?;
    original
        .commit_page(page(&original, &catalog, "discover", "old-discover", 2, false).await?)
        .await?;
    let pending = page(&original, &catalog, "metadata", "pending-old", 3, true).await?;
    let lease = pending.lease.clone();
    drop(original);
    old_image(&directory, &reference).await?;
    let sql = pool(&directory).await?;
    let generation: i64 =
        sqlx::query_scalar("SELECT generation FROM work_queue WHERE sweep_id='pending-old'")
            .fetch_one(&sql)
            .await?;
    sql.close().await;
    let store =
        BinaryStore::open_with_catalog(&directory.path().join("inventory.sqlite"), &bytes).await?;
    assert!(matches!(
        store.commit_page(pending).await,
        Err(inventory_engine::inventory::StoreError::StaleLease)
    ));
    assert!(matches!(
        store.finish_scope(lease, ScopeOutcome::Complete, 4).await,
        Err(inventory_engine::inventory::StoreError::StaleLease)
    ));
    let sql = pool(&directory).await?;
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT generation FROM work_queue WHERE sweep_id='pending-old'"
        )
        .fetch_one(&sql)
        .await?,
        generation
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT authority_generation FROM work_queue WHERE sweep_id='pending-old'"
        )
        .fetch_one(&sql)
        .await?,
        0
    );
    sql.close().await;
    let sweep = store
        .discover_active_sweep()
        .await?
        .ok_or("old work was not resumable")?;
    let lease = store
        .claim(&sweep, 10_010, 10_000)
        .await?
        .ok_or("new claim absent")?;
    store.finish_scope(lease, ScopeOutcome::Complete, 6).await?;
    Ok(())
}

#[tokio::test]
async fn given_unavailable_catalog_when_fresh_generic_completes_match_old_owners_then_no_v3_absence_is_certified()
-> TestResult {
    let bytes = catalog_bytes()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, original) = store(&catalog).await?;
    let old = page(&original, &catalog, "metadata", "old-metadata", 1, true).await?;
    let reference = fixture::historical_evidence(
        &catalog,
        old.unresolved
            .first()
            .ok_or("native OAM reference absent")?,
    )?;
    original.commit_page(old).await?;
    original
        .commit_page(page(&original, &catalog, "discover", "old-discover", 2, false).await?)
        .await?;
    drop(original);
    old_image(&directory, &reference).await?;
    let sql = pool(&directory).await?;
    let before = frozen(&sql, &catalog).await?;
    sqlx::query("DELETE FROM metadata WHERE key='reviewed_catalog'")
        .execute(&sql)
        .await?;
    sql.close().await;
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    for (sweep, at) in [("unchecked-one", 10), ("unchecked-two", 20)] {
        store
            .commit_page(page(&store, &catalog, "discover", sweep, at, false).await?)
            .await?;
    }
    let sql = pool(&directory).await?;
    assert_eq!(frozen(&sql, &catalog).await?, before);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT v3_misses FROM legacy_authorities WHERE id=?1")
            .bind(LINK)
            .fetch_one(&sql)
            .await?,
        0
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM legacy_refreshes WHERE id=?1")
            .bind(LINK)
            .fetch_one(&sql)
            .await?,
        0
    );
    assert_eq!(store.unattributed_authority_count().await?, 1);
    sql.close().await;
    Ok(())
}
