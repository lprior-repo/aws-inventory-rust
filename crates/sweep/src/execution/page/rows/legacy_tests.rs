use super::*;
#[path = "legacy_tests/fixture.rs"]
mod legacy_fixture;
use legacy_fixture::*;

async fn populated_upgrade(version: u8) -> TestResult {
    let bytes = fixture::compiled_competing()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    baseline(&store, &catalog).await?;
    assert_eq!(
        edge_image(&store).await?,
        vec![(
            BROWSER.to_owned(),
            "USES_BROWSER_SETTINGS".to_owned(),
            "reviewed-rule".to_owned()
        )]
    );
    let (sweep, generation, bindings, token) = staged_work(&store, &catalog).await?;
    drop(store);
    downgrade(&directory, version).await?;
    let before = image(&directory, &catalog).await?;
    let old_work = work_image(&directory).await?;
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(reopened.unattributed_authority_count().await?, 2);
    assert_eq!(image(&directory, &catalog).await?, before);
    assert_eq!(work_image(&directory).await?, old_work);
    drop(reopened);
    let reopened =
        BinaryStore::open_with_catalog(&directory.path().join("inventory.sqlite"), &bytes).await?;
    let lease = reopened
        .claim(&sweep, 20_000, 100)
        .await?
        .ok_or("old leased work did not resume")?;
    assert_eq!(lease.generation(), generation.saturating_add(1));
    assert_eq!(lease.bindings(), bindings);
    assert_eq!(lease.token(), Some(token.as_slice()));
    assert!(lease.context().is_empty());
    resume_first_page(&reopened, &catalog, lease).await?;
    assert_eq!(image(&directory, &catalog).await?, before);
    finish_partial(&reopened, &sweep).await?;
    assert_eq!(image(&directory, &catalog).await?, before);
    drop(reopened);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(reopened.unattributed_authority_count().await?, 2);
    assert_eq!(image(&directory, &catalog).await?, before);
    persist(
        &reopened,
        &catalog,
        "ListBrowserSettings",
        json!({"browserSettings":[{"browserSettingsArn":BROWSER}]}),
        21_000,
    )
    .await?;
    persist(
        &reopened,
        &catalog,
        "ListPortals",
        json!({"portals":[{"portalArn":PORTAL,"displayName":"fresh-list"}]}),
        21_001,
    )
    .await?;
    persist(&reopened, &catalog, "GetPortalServiceProviderMetadata", json!({"portalArn":PORTAL,"serviceProviderSamlMetadata":"<fresh/>","zzMetadataBrowserSettingsArn":SECONDARY}), 21_002).await?;
    assert_eq!(state(&directory, &catalog).await?, before.0);
    assert_eq!(reopened.unattributed_authority_count().await?, 1);
    persist(&reopened, &catalog, "GetPortal", json!({"portal":{"portalArn":PORTAL,"displayName":"fresh-detail","browserSettingsArn":BROWSER}}), 21_003).await?;
    assert_eq!(reopened.unattributed_authority_count().await?, 0);
    let current = state(&directory, &catalog).await?;
    assert_eq!(
        field(&current.0, 3)?,
        Some(Value::Text("fresh-detail".to_owned()))
    );
    assert_eq!(
        field(&current.0, 7)?,
        Some(Value::Text("<fresh/>".to_owned()))
    );
    assert_eq!(current.1, 21_003);
    persist(&reopened, &catalog, "ListBrowserSettings", json!({"browserSettings":[{"browserSettingsArn":BROWSER},{"browserSettingsArn":SECONDARY}]}), 21_004).await?;
    assert_eq!(
        edge_image(&reopened).await?,
        vec![
            (
                BROWSER.to_owned(),
                "USES_BROWSER_SETTINGS".to_owned(),
                "reviewed-rule".to_owned()
            ),
            (
                SECONDARY.to_owned(),
                "USES_METADATA_SETTINGS".to_owned(),
                "reviewed-rule".to_owned()
            )
        ]
    );
    Ok(())
}

#[tokio::test]
async fn given_populated_legacy_owners_when_all_are_completely_absent_twice_then_baseline_is_tombstoned_not_published_empty()
-> TestResult {
    let bytes = fixture::compiled_competing()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    baseline(&store, &catalog).await?;
    let before = state(&directory, &catalog).await?;
    drop(store);
    downgrade(&directory, 15).await?;
    let reopened =
        BinaryStore::open_with_catalog(&directory.path().join("inventory.sqlite"), &bytes).await?;
    for (index, operation) in [
        "ListPortals",
        "GetPortal",
        "GetPortalServiceProviderMetadata",
        "ListBrowserSettings",
    ]
    .into_iter()
    .enumerate()
    {
        complete_empty(&reopened, operation, 100 + i64::try_from(index)? * 2).await?;
    }
    assert_eq!(state(&directory, &catalog).await?, before);
    assert_eq!(reopened.unattributed_authority_count().await?, 2);
    drop(reopened);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    for (index, operation) in [
        "ListPortals",
        "GetPortal",
        "GetPortalServiceProviderMetadata",
        "ListBrowserSettings",
    ]
    .into_iter()
    .enumerate()
    {
        complete_empty(&reopened, operation, 200 + i64::try_from(index)? * 2).await?;
    }
    assert_eq!(reopened.unattributed_authority_count().await?, 0);
    assert_eq!(reopened.projection_resource(&key()?).await?, None);
    assert_eq!(reopened.projection_resources(None, 100).await?, Vec::new());
    assert_eq!(
        reopened
            .drain_projection(100)
            .await?
            .iter()
            .filter(|entry| entry.deleted)
            .count(),
        2
    );
    Ok(())
}

#[tokio::test]
async fn given_an_orphan_legacy_baseline_when_only_partial_or_unrelated_complete_work_runs_then_no_unattributed_fields_are_discarded()
-> TestResult {
    let bytes = fixture::compiled_competing()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    baseline(&store, &catalog).await?;
    drop(store);
    let db = pool(&directory).await?;
    sqlx::query("DELETE FROM resource_scopes WHERE id=?1")
        .bind(PORTAL)
        .execute(&db)
        .await?;
    db.close().await;
    downgrade(&directory, 15).await?;
    let before = state(&directory, &catalog).await?;
    let reopened =
        BinaryStore::open_with_catalog(&directory.path().join("inventory.sqlite"), &bytes).await?;
    let sweep = SweepId::new("orphan-partial".to_owned())?;
    reopened
        .seed_work(work(&sweep, "ListPortals", None)?)
        .await?;
    let lease = reopened
        .claim(&sweep, 20_000, 100)
        .await?
        .ok_or("orphan source absent")?;
    resume_first_page(&reopened, &catalog, lease).await?;
    let lease = reopened
        .claim(&sweep, 20_002, 100)
        .await?
        .ok_or("orphan continuation absent")?;
    reopened
        .finish_scope(
            lease,
            inventory_core::inventory::ScopeOutcome::Partial,
            20_003,
            None,
        )
        .await?;
    persist(
        &reopened,
        &catalog,
        "ListBrowserSettings",
        json!({"browserSettings":[{"browserSettingsArn":BROWSER}]}),
        21_000,
    )
    .await?;
    assert_eq!(state(&directory, &catalog).await?, before);
    assert_eq!(reopened.unattributed_authority_count().await?, 1);
    drop(reopened);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(state(&directory, &catalog).await?, before);
    assert_eq!(reopened.unattributed_authority_count().await?, 1);
    persist(
        &reopened,
        &catalog,
        "ListPortals",
        json!({"portals":[{"portalArn":PORTAL,"displayName":"certified"}]}),
        21_001,
    )
    .await?;
    assert_eq!(reopened.unattributed_authority_count().await?, 0);
    let current = state(&directory, &catalog).await?;
    assert_eq!(
        field(&current.0, 3)?,
        Some(Value::Text("certified".to_owned()))
    );
    assert_eq!(field(&current.0, 2)?, None);
    assert_eq!(field(&current.0, 7)?, None);
    assert_eq!(edge_image(&reopened).await?, Vec::new());
    Ok(())
}

#[tokio::test]
async fn given_populated_schema14_and_inflight_pages_when_upgraded_then_exact_baseline_and_resumption_survive_until_attributed_cutover()
-> TestResult {
    populated_upgrade(14).await
}

#[tokio::test]
async fn given_populated_schema15_and_inflight_pages_when_upgraded_then_exact_baseline_and_resumption_survive_until_attributed_cutover()
-> TestResult {
    populated_upgrade(15).await
}

#[path = "legacy_child_tests.rs"]
mod children;
