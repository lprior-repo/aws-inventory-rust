use super::*;
#[tokio::test]
async fn given_ambiguous_group_when_candidates_are_tombstoned_then_resolution_recovers_unique_and_then_unresolved()
-> TestResult {
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
    persist(
        &store,
        &catalog,
        "ListUserSettings",
        json!({"userSettings":[{"userSettingsArn":USER,"browserSettingsArn":BROWSER}]}),
        4,
    )
    .await?;
    assert_eq!(status(&directory).await?, (1, "ambiguous".to_owned()));
    persist(
        &store,
        &catalog,
        "ListUserSettings",
        json!({"userSettings":[]}),
        5,
    )
    .await?;
    assert_eq!(status(&directory).await?, (1, "ambiguous".to_owned()));
    persist(
        &store,
        &catalog,
        "ListUserSettings",
        json!({"userSettings":[]}),
        6,
    )
    .await?;
    assert_eq!(status(&directory).await?, (1, "resolved".to_owned()));
    assert_eq!(
        store
            .projection_edges_for(&key()?, None, 100)
            .await?
            .edges
            .first()
            .ok_or("surviving target edge absent")?
            .to
            .id
            .as_str(),
        BROWSER
    );
    persist(
        &store,
        &catalog,
        "ListBrowserSettings",
        json!({"browserSettings":[]}),
        7,
    )
    .await?;
    persist(
        &store,
        &catalog,
        "ListBrowserSettings",
        json!({"browserSettings":[]}),
        8,
    )
    .await?;
    assert_eq!(status(&directory).await?, (1, "unresolved".to_owned()));
    assert_eq!(
        store.projection_edges_for(&key()?, None, 100).await?.edges,
        Vec::new()
    );
    assert_eq!(store.projection_resources(None, 100).await?.len(), 1);
    drop(store);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    persist(
        &reopened,
        &catalog,
        "ListBrowserSettings",
        json!({"browserSettings":[{"browserSettingsArn":BROWSER}]}),
        9,
    )
    .await?;
    assert_eq!(status(&directory).await?, (1, "resolved".to_owned()));
    assert_eq!(reopened.projection_resources(None, 100).await?.len(), 2);
    assert_eq!(
        reopened
            .projection_edges_for(&key()?, None, 100)
            .await?
            .edges
            .len(),
        1
    );
    assert_eq!(
        reopened
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
    persist(
        &reopened,
        &catalog,
        "ListUserSettings",
        json!({"userSettings":[{"userSettingsArn":USER,"browserSettingsArn":BROWSER}]}),
        10,
    )
    .await?;
    assert_eq!(status(&directory).await?, (1, "ambiguous".to_owned()));
    assert_eq!(
        reopened
            .projection_edges_for(&key()?, None, 100)
            .await?
            .edges,
        Vec::new()
    );
    Ok(())
}
