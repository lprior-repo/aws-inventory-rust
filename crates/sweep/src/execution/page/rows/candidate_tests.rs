use super::*;
mod retirement;
const USER: &str = "arn:aws:workspaces-web:us-east-1:123456789012:userSettings/u-1";

async fn status(directory: &tempfile::TempDir) -> TestResult<(i64, String)> {
    let pool = sqlx::SqlitePool::connect(&format!(
        "sqlite://{}",
        directory.path().join("inventory.sqlite").display()
    ))
    .await?;
    let row = sqlx::query("SELECT count(*) AS count,min(resolution) AS resolution FROM unresolved_refs WHERE from_id=?1").bind(PORTAL).fetch_one(&pool).await?;
    let result = (row.try_get("count")?, row.try_get("resolution")?);
    pool.close().await;
    Ok(result)
}

async fn source(store: &BinaryStore, catalog: &RuntimeCatalog<'_>) -> TestResult {
    persist(
        store,
        catalog,
        "ListPortals",
        json!({"portals":[{"portalArn":PORTAL,"browserSettingsArn":BROWSER}]}),
        1,
    )
    .await
}

#[tokio::test]
async fn given_candidate_group_before_targets_when_unique_then_ambiguous_then_unique_again_edges_follow_observed_evidence()
-> TestResult {
    // Given one group with no observed candidate, not two asserted placeholder references.
    let bytes = fixture::compiled_candidates()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    source(&store, &catalog).await?;
    assert_eq!(status(&directory).await?, (1, "unresolved".to_owned()));
    assert_eq!(
        store.projection_edges_for(&key()?, None, 100).await?.edges,
        Vec::new()
    );
    // When exactly one allowed kind is observed, the evidence resolves to it.
    persist(
        &store,
        &catalog,
        "ListBrowserSettings",
        json!({"browserSettings":[{"browserSettingsArn":BROWSER}]}),
        3,
    )
    .await?;
    assert_eq!(status(&directory).await?, (1, "resolved".to_owned()));
    assert_eq!(
        store
            .projection_edges_for(&key()?, None, 100)
            .await?
            .edges
            .first()
            .ok_or("unique edge absent")?
            .to
            .kind
            .as_str(),
        "AWS::WorkSpacesWeb::BrowserSettings"
    );
    // When a second allowed kind matches the same path/value, no edge may claim certainty.
    persist(
        &store,
        &catalog,
        "ListUserSettings",
        json!({"userSettings":[{"userSettingsArn":USER,"browserSettingsArn":BROWSER}]}),
        4,
    )
    .await?;
    assert_eq!(status(&directory).await?, (1, "ambiguous".to_owned()));
    assert_eq!(
        store.projection_edges_for(&key()?, None, 100).await?.edges,
        Vec::new()
    );
    // Then removing that candidate's matching field restores the unique remaining match.
    persist(
        &store,
        &catalog,
        "ListUserSettings",
        json!({"userSettings":[{"userSettingsArn":USER}]}),
        5,
    )
    .await?;
    assert_eq!(status(&directory).await?, (1, "resolved".to_owned()));
    assert_eq!(
        store
            .projection_edges_for(&key()?, None, 100)
            .await?
            .edges
            .first()
            .ok_or("restored edge absent")?
            .to
            .id
            .as_str(),
        BROWSER
    );
    Ok(())
}

#[tokio::test]
async fn given_observed_candidate_before_group_when_reference_arrives_then_only_actual_kind_resolves()
-> TestResult {
    // Given the target exists before its referring resource and group.
    let bytes = fixture::compiled_candidates()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    persist(
        &store,
        &catalog,
        "ListUserSettings",
        json!({"userSettings":[{"userSettingsArn":USER,"browserSettingsArn":BROWSER}]}),
        0,
    )
    .await?;
    // Native decoding/extraction must preserve the reviewed nonprimary alternative.
    source(&store, &catalog).await?;
    // Then exactly one group and exactly one observed target exist.
    assert_eq!(status(&directory).await?, (1, "resolved".to_owned()));
    let edges = store.projection_edges_for(&key()?, None, 100).await?.edges;
    assert_eq!(edges.len(), 1);
    assert_eq!(
        edges.first().ok_or("target edge absent")?.to.kind.as_str(),
        "AWS::WorkSpacesWeb::UserSettings"
    );
    assert_eq!(store.projection_resources(None, 100).await?.len(), 2);
    assert_eq!(
        edges.first().ok_or("target edge absent")?.to.id.as_str(),
        USER
    );
    persist(
        &store,
        &catalog,
        "ListBrowserSettings",
        json!({"browserSettings":[{"browserSettingsArn":BROWSER}]}),
        2,
    )
    .await?;
    assert_eq!(status(&directory).await?, (1, "ambiguous".to_owned()));
    assert_eq!(
        store.projection_edges_for(&key()?, None, 100).await?.edges,
        Vec::new()
    );
    Ok(())
}


async fn reject_decoy(account: &str, region: &str) -> TestResult {
    let bytes = fixture::compiled_candidates()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    source(&store, &catalog).await?;
    persist(
        &store,
        &catalog,
        "ListBrowserSettings",
        json!({"browserSettings":[{"browserSettingsArn":BROWSER}]}),
        2,
    )
    .await?;
    let expected = edge_image(&store).await?;
    assert_eq!(
        expected,
        vec![(
            BROWSER.to_owned(),
            "USES_BROWSER_SETTINGS".to_owned(),
            inventory_core::inventory::EdgeSource::ReviewedRule
                .as_str()
                .to_owned()
        )]
    );
    let decoy = format!("arn:aws:workspaces-web:{region}:{account}:userSettings/decoy");
    persist_scoped(
        &store,
        &catalog,
        "ListUserSettings",
        json!({"userSettings":[{"userSettingsArn":decoy,"browserSettingsArn":BROWSER}]}),
        account,
        region,
        3,
    )
    .await?;
    assert_eq!(status(&directory).await?, (1, "resolved".to_owned()));
    assert_eq!(edge_image(&store).await?, expected);
    assert_eq!(store.projection_resources(None, 100).await?.len(), 3);
    persist_scoped(
        &store,
        &catalog,
        "ListUserSettings",
        json!({"userSettings":[{"userSettingsArn":decoy}]}),
        account,
        region,
        4,
    )
    .await?;
    assert_eq!(status(&directory).await?, (1, "resolved".to_owned()));
    assert_eq!(edge_image(&store).await?, expected);
    persist_scoped(
        &store,
        &catalog,
        "ListUserSettings",
        json!({"userSettings":[]}),
        account,
        region,
        5,
    )
    .await?;
    persist_scoped(
        &store,
        &catalog,
        "ListUserSettings",
        json!({"userSettings":[]}),
        account,
        region,
        6,
    )
    .await?;
    assert_eq!(status(&directory).await?, (1, "resolved".to_owned()));
    assert_eq!(edge_image(&store).await?, expected);
    assert_eq!(store.projection_resources(None, 100).await?.len(), 2);
    Ok(())
}

#[tokio::test]
async fn given_same_candidate_value_in_another_account_when_observed_updated_and_deleted_then_reference_stays_in_its_owner_account()
-> TestResult {
    reject_decoy("999999999999", "us-east-1").await
}

#[tokio::test]
async fn given_same_candidate_value_in_another_region_when_observed_updated_and_deleted_then_reference_stays_in_its_owner_region()
-> TestResult {
    reject_decoy("123456789012", "us-west-2").await
}
