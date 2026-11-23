use super::*;
#[tokio::test]
async fn given_live_list_when_detail_is_absent_twice_after_reopen_then_only_detail_authority_retires()
-> TestResult {
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    full(&store, &catalog, false).await?;
    let before = state(&directory, &catalog).await?;
    persist(&store, &catalog, "GetPortal", json!({}), 5).await?;
    assert_eq!(state(&directory, &catalog).await?, before);
    persist(
        &store,
        &catalog,
        "ListPortals",
        json!({"portals":[{"portalArn":PORTAL,"displayName":"list-name"}]}),
        6,
    )
    .await?;
    drop(store);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    persist(&reopened, &catalog, "GetPortal", json!({}), 7).await?;
    let current = state(&directory, &catalog).await?;
    assert_eq!(field(&current.0, 2)?, None);
    assert_eq!(
        field(&current.0, 3)?,
        Some(Value::Text("list-name".to_owned()))
    );
    assert_eq!(
        field(&current.0, 7)?,
        Some(Value::Text("<metadata/>".to_owned()))
    );
    assert_eq!(current.1, 7);
    assert_ne!(current.2, before.2);
    assert_eq!(
        reopened
            .projection_resource(&key()?)
            .await?
            .ok_or("surviving portal absent")?
            .name
            .as_deref(),
        Some("list-name")
    );
    assert_eq!(
        reopened
            .projection_edges_for(&key()?, None, 100)
            .await?
            .edges,
        Vec::new()
    );
    let pool = sqlx::SqlitePool::connect(&format!(
        "sqlite://{}",
        directory.path().join("inventory.sqlite").display()
    ))
    .await?;
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM unresolved_refs WHERE from_id=?1")
            .bind(PORTAL)
            .fetch_one(&pool)
            .await?,
        0
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resource_facets WHERE id=?1")
            .bind(PORTAL)
            .fetch_one(&pool)
            .await?,
        2
    );
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_competing_reference_owners_when_detail_wins_then_exact_edges_follow_canonical_leaf_and_sibling_survives()
-> TestResult {
    const DETAIL: &str = "arn:aws:workspaces-web:us-east-1:123456789012:browserSettings/b-2";
    const SIBLING: &str = "arn:aws:workspaces-web:us-east-1:123456789012:browserSettings/b-3";
    let bytes = fixture::compiled_competing()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    for reverse in [false, true] {
        let (directory, store) = store(&catalog).await?;
        persist(&store, &catalog, "ListBrowserSettings", json!({"browserSettings":[{"browserSettingsArn":BROWSER},{"browserSettingsArn":DETAIL},{"browserSettingsArn":SIBLING}]}), 1).await?;
        let list = (
            "ListPortals",
            json!({"portals":[{"portalArn":PORTAL,"browserSettingsArn":BROWSER}]}),
        );
        let detail = (
            "GetPortal",
            json!({"portal":{"portalArn":PORTAL,"browserSettingsArn":DETAIL}}),
        );
        let sibling = (
            "GetPortalServiceProviderMetadata",
            json!({"portalArn":PORTAL,"zzMetadataBrowserSettingsArn":SIBLING}),
        );
        let operations = if reverse {
            [sibling, detail, list]
        } else {
            [list, detail, sibling]
        };
        for (index, (operation, body)) in operations.into_iter().enumerate() {
            persist(
                &store,
                &catalog,
                operation,
                body,
                i64::try_from(index)?
                    .checked_add(2)
                    .ok_or("time overflow")?,
            )
            .await?;
        }
        let current = state(&directory, &catalog).await?.0;
        assert_eq!(field(&current, 2)?, Some(Value::Text(DETAIL.to_owned())));
        assert_eq!(field(&current, 10)?, Some(Value::Text(SIBLING.to_owned())));
        assert_eq!(
            edge_image(&store).await?,
            vec![
                (
                    DETAIL.to_owned(),
                    "USES_BROWSER_SETTINGS".to_owned(),
                    inventory_core::inventory::EdgeSource::ReviewedRule
                        .as_str()
                        .to_owned()
                ),
                (
                    SIBLING.to_owned(),
                    "USES_METADATA_SETTINGS".to_owned(),
                    inventory_core::inventory::EdgeSource::ReviewedRule
                        .as_str()
                        .to_owned()
                ),
            ]
        );
        drop(store);
        let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
        persist(
            &reopened,
            &catalog,
            "GetPortal",
            json!({"portal":{"portalArn":PORTAL}}),
            8,
        )
        .await?;
        let current = state(&directory, &catalog).await?.0;
        assert_eq!(field(&current, 2)?, None);
        assert_eq!(field(&current, 10)?, Some(Value::Text(SIBLING.to_owned())));
        assert_eq!(
            edge_image(&reopened).await?,
            vec![(
                SIBLING.to_owned(),
                "USES_METADATA_SETTINGS".to_owned(),
                inventory_core::inventory::EdgeSource::ReviewedRule
                    .as_str()
                    .to_owned()
            ),]
        );
    }
    Ok(())
}
