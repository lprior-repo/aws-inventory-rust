use super::*;
const OTHER_PORTAL: &str = "arn:aws:workspaces-web:us-east-1:123456789012:portal/p-legacy-other";

#[tokio::test]
async fn given_a_context_free_legacy_child_when_recollected_with_context_then_same_binding_releases_baseline_and_two_misses_retire_only_detail() -> TestResult {
    // Given actual pre-context ownership and an archived nonempty child request.
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    persist(&store, &catalog, "ListPortals", json!({"portals":[{"portalArn":PORTAL,"displayName":"old-list"}]}), 1).await?;
    child_run(&store, &catalog, "z-old", PORTAL, None, Some("old-detail"), 2).await?;
    drop(store);
    restore_raw_child_scopes(&directory).await?;
    downgrade(&directory, 15).await?;
    let before = state(&directory, &catalog).await?;
    let reopened = BinaryStore::open_with_catalog(&directory.path().join("inventory.sqlite"), &bytes).await?;
    assert_eq!(reopened.unattributed_authority_count().await?, 1);
    persist(&reopened, &catalog, "ListPortals", json!({"portals":[{"portalArn":PORTAL,"displayName":"fresh-list"}]}), 10).await?;
    assert_eq!(state(&directory, &catalog).await?, before);
    assert_eq!(reopened.unattributed_authority_count().await?, 1);
    drop(reopened);
    // When fresh contextual work has the same real binding, it certifies the historical authority.
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    child_run(&reopened, &catalog, "fresh-context", PORTAL, Some(17), Some("fresh-detail"), 20).await?;
    assert_eq!(reopened.unattributed_authority_count().await?, 0);
    assert_eq!(field(&state(&directory, &catalog).await?.0, 3)?, Some(Value::Text("fresh-detail".to_owned())));
    let db = pool(&directory).await?;
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_scopes WHERE id=?1").bind(PORTAL).fetch_one(&db).await?, 2);
    db.close().await;
    child_run(&reopened, &catalog, "fresh-miss-one", PORTAL, Some(18), None, 30).await?;
    assert_eq!(reopened.projection_resource(&key()?).await?.ok_or("detail vanished on first miss")?.name.as_deref(), Some("fresh-detail"));
    drop(reopened);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    child_run(&reopened, &catalog, "fresh-miss-two", PORTAL, Some(19), None, 40).await?;
    // Then the old context-free child is not immortal and its independent lower list facet survives.
    assert_eq!(reopened.projection_resource(&key()?).await?.ok_or("live root facet vanished")?.name.as_deref(), Some("fresh-list"));
    assert_eq!(field(&state(&directory, &catalog).await?.0, 3)?, Some(Value::Text("fresh-list".to_owned())));
    let db = pool(&directory).await?;
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_parents WHERE child_id=?1").bind(PORTAL).fetch_one(&db).await?, 0);
    db.close().await;
    Ok(())
}

#[tokio::test]
async fn given_one_old_scope_with_nonchronological_archives_and_distinct_bindings_when_only_one_binding_misses_then_other_resource_authority_survives() -> TestResult {
    // Given two resources sharing an old execution scope, but each last observed under a different request.
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    persist(&store, &catalog, "ListPortals", json!({"portals":[{"portalArn":PORTAL,"displayName":"old-root"}]}), 1).await?;
    child_run(&store, &catalog, "z-old", PORTAL, None, Some("detail-A"), 2).await?;
    child_run(&store, &catalog, "a-new", OTHER_PORTAL, None, Some("detail-B"), 4).await?;
    let mut other = key()?;
    other.id = ResourceId::parse(OTHER_PORTAL)?;
    let old_other = store.projection_resource(&other).await?;
    drop(store);
    restore_raw_child_scopes(&directory).await?;
    let db = pool(&directory).await?;
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(DISTINCT scope_key) FROM resource_scopes WHERE observed_sweep IN('z-old','a-new')").fetch_one(&db).await?, 1);
    db.close().await;
    downgrade(&directory, 15).await?;
    let reopened = BinaryStore::open_with_catalog(&directory.path().join("inventory.sqlite"), &bytes).await?;
    // When migration uses each row's exact last-positive archive, both semantic owner keys differ.
    let db = pool(&directory).await?;
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(DISTINCT scope_key) FROM resource_scopes WHERE observed_sweep IN('z-old','a-new')").fetch_one(&db).await?, 2);
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(DISTINCT scope_key) FROM resource_parents").fetch_one(&db).await?, 2);
    db.close().await;
    persist(&reopened, &catalog, "ListPortals", json!({"portals":[{"portalArn":PORTAL,"displayName":"fresh-root"}]}), 10).await?;
    child_run(&reopened, &catalog, "A-first-miss", PORTAL, Some(11), None, 20).await?;
    drop(reopened);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    child_run(&reopened, &catalog, "A-second-miss", PORTAL, Some(12), None, 30).await?;
    // Then A falls back to its root facet without incorrectly expiring B or its parent link.
    assert_eq!(reopened.projection_resource(&key()?).await?.ok_or("A lower facet absent")?.name.as_deref(), Some("fresh-root"));
    assert_eq!(reopened.projection_resource(&other).await?, old_other);
    assert_eq!(reopened.unattributed_authority_count().await?, 1);
    let db = pool(&directory).await?;
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT missing_complete FROM resource_scopes WHERE id=?1").bind(OTHER_PORTAL).fetch_one(&db).await?, 0);
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_parents WHERE child_id=?1").bind(OTHER_PORTAL).fetch_one(&db).await?, 1);
    db.close().await;
    Ok(())
}

#[tokio::test]
async fn given_a_legacy_child_without_its_exact_positive_archive_when_fresh_and_unrelated_work_complete_then_opaque_baseline_remains_explicit_and_readable() -> TestResult {
    // Given an old parent-owned record whose only actual last-positive archive was pruned.
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    persist(&store, &catalog, "ListPortals", json!({"portals":[{"portalArn":PORTAL,"displayName":"old-list"}]}), 1).await?;
    child_run(&store, &catalog, "z-old", PORTAL, None, Some("opaque-detail"), 2).await?;
    drop(store);
    restore_raw_child_scopes(&directory).await?;
    let db = pool(&directory).await?;
    sqlx::query("DELETE FROM resource_scopes WHERE observed_sweep<>'z-old'").execute(&db).await?;
    sqlx::query("DELETE FROM work_queue WHERE sweep_id='z-old'").execute(&db).await?;
    db.close().await;
    downgrade(&directory, 15).await?;
    let before = image(&directory, &catalog).await?;
    // When opening even without a reviewed catalog, loss of attribution is reported rather than guessed or rejected.
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(reopened.unattributed_authority_count().await?, 1);
    assert_eq!(image(&directory, &catalog).await?, before);
    drop(reopened);
    let reopened = BinaryStore::open_with_catalog(&directory.path().join("inventory.sqlite"), &bytes).await?;
    child_run(&reopened, &catalog, "fresh-opaque", PORTAL, Some(1), Some("must-remain-buffered"), 10).await?;
    complete_empty(&reopened, "ListBrowserSettings", 20).await?;
    assert_eq!(image(&directory, &catalog).await?, before);
    assert_eq!(reopened.unattributed_authority_count().await?, 1);
    drop(reopened);
    // Then restart keeps the exact baseline and exposes both unknown prior and fresh semantic ownership.
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(image(&directory, &catalog).await?, before);
    assert_eq!(reopened.unattributed_authority_count().await?, 1);
    let db = pool(&directory).await?;
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM legacy_unknown_authorities WHERE id=?1").bind(PORTAL).fetch_one(&db).await?, 1);
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_scopes WHERE id=?1").bind(PORTAL).fetch_one(&db).await?, 2);
    db.close().await;
    Ok(())
}

#[tokio::test]
async fn given_an_opaque_parent_owned_legacy_detail_when_that_exact_parent_retires_then_independent_root_authority_survives_and_unknown_origin_retires() -> TestResult {
    // Given a child with independent root ownership and an opaque but explicitly linked parent origin.
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    persist(&store, &catalog, "ListPortals", json!({"portals":[{"portalArn":PORTAL,"displayName":"parent"},{"portalArn":OTHER_PORTAL,"displayName":"old-root"}]}), 1).await?;
    child_run(&store, &catalog, "a-new", OTHER_PORTAL, None, Some("opaque-detail"), 2).await?;
    drop(store);
    restore_raw_child_scopes(&directory).await?;
    let db = pool(&directory).await?;
    sqlx::query("DELETE FROM work_queue WHERE sweep_id='a-new'").execute(&db).await?;
    db.close().await;
    downgrade(&directory, 15).await?;
    let reopened = BinaryStore::open_with_catalog(&directory.path().join("inventory.sqlite"), &bytes).await?;
    let mut other = key()?;
    other.id = ResourceId::parse(OTHER_PORTAL)?;
    assert_eq!(reopened.unattributed_authority_count().await?, 2);
    // When the parent is absent twice, the actual parent link (not a guessed binding) retires its source.
    persist(&reopened, &catalog, "ListPortals", json!({"portals":[{"portalArn":OTHER_PORTAL,"displayName":"fresh-root"}]}), 10).await?;
    assert_eq!(reopened.projection_resource(&other).await?.ok_or("opaque detail absent before retirement")?.name.as_deref(), Some("opaque-detail"));
    drop(reopened);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    persist(&reopened, &catalog, "ListPortals", json!({"portals":[{"portalArn":OTHER_PORTAL,"displayName":"fresh-root"}]}), 20).await?;
    // Then the independently owned child cuts over while the deleted parent and opaque origin disappear.
    assert_eq!(reopened.projection_resource(&key()?).await?, None);
    assert_eq!(reopened.projection_resource(&other).await?.ok_or("independent child was cascaded")?.name.as_deref(), Some("fresh-root"));
    assert_eq!(reopened.projection_resources(None, 100).await?.len(), 1);
    assert_eq!(reopened.unattributed_authority_count().await?, 0);
    let db = pool(&directory).await?;
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM legacy_unknown_authorities").fetch_one(&db).await?, 0);
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_parents WHERE child_id=?1").bind(OTHER_PORTAL).fetch_one(&db).await?, 0);
    db.close().await;
    Ok(())
}

#[tokio::test]
async fn given_an_already_expired_legacy_child_without_archive_when_upgraded_then_it_is_not_resurrected_as_opaque_live_authority() -> TestResult {
    // Given a live root and an already exhausted old child whose request archive was pruned.
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    persist(&store, &catalog, "ListPortals", json!({"portals":[{"portalArn":PORTAL,"displayName":"old-list"}]}), 1).await?;
    child_run(&store, &catalog, "z-old", PORTAL, None, Some("expired-detail"), 2).await?;
    drop(store);
    restore_raw_child_scopes(&directory).await?;
    let db = pool(&directory).await?;
    sqlx::query("UPDATE resource_scopes SET missing_complete=2 WHERE observed_sweep='z-old'").execute(&db).await?;
    sqlx::query("DELETE FROM work_queue WHERE sweep_id='z-old'").execute(&db).await?;
    db.close().await;
    downgrade(&directory, 15).await?;
    // When the old schema is opened, only live prior owners may become unknown authority.
    let reopened = BinaryStore::open_with_catalog(&directory.path().join("inventory.sqlite"), &bytes).await?;
    assert_eq!(reopened.unattributed_authority_count().await?, 1);
    let db = pool(&directory).await?;
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM legacy_unknown_authorities").fetch_one(&db).await?, 0);
    db.close().await;
    persist(&reopened, &catalog, "ListPortals", json!({"portals":[{"portalArn":PORTAL,"displayName":"certified-root"}]}), 10).await?;
    // Then fresh completion releases the baseline instead of retaining an immortal exhausted child.
    assert_eq!(reopened.unattributed_authority_count().await?, 0);
    assert_eq!(reopened.projection_resource(&key()?).await?.ok_or("root was lost")?.name.as_deref(), Some("certified-root"));
    drop(reopened);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(reopened.unattributed_authority_count().await?, 0);
    assert_eq!(reopened.projection_resource(&key()?).await?.ok_or("root was lost after restart")?.name.as_deref(), Some("certified-root"));
    Ok(())
}
