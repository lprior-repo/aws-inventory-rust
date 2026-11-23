use super::*;
#[tokio::test]
async fn given_native_list_detail_and_metadata_when_sparse_list_repeats_then_canonical_state_and_edges_survive()
-> TestResult {
    // Given native WorkSpaces Web list, singleton detail, and metadata responses.
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    full(&store, &catalog, false).await?;
    let before = state(&directory, &catalog).await?;
    // When a later enumeration contains only the stable ARN.
    persist(
        &store,
        &catalog,
        "ListPortals",
        json!({"portals":[{"portalArn":PORTAL}]}),
        10,
    )
    .await?;
    // Then the detail name, typed values, stable hash/timestamp, and settings edge remain.
    assert_eq!(state(&directory, &catalog).await?, before);
    assert_eq!(
        store
            .projection_resource(&key()?)
            .await?
            .ok_or("portal absent")?
            .name
            .as_deref(),
        Some("detail-name")
    );
    assert_eq!(field(&before.0, 2)?, Some(Value::Text(BROWSER.to_owned())));
    assert_eq!(
        field(&before.0, 7)?,
        Some(Value::Text("<metadata/>".to_owned()))
    );
    let edges = store.projection_edges_for(&key()?, None, 100).await?;
    assert_eq!(edges.edges.len(), 1);
    assert_eq!(
        edges
            .edges
            .first()
            .ok_or("settings edge absent")?
            .to
            .id
            .as_str(),
        BROWSER
    );
    Ok(())
}

#[tokio::test]
async fn given_identical_native_facets_when_completion_order_reverses_then_final_bytes_hash_name_and_edges_match()
-> TestResult {
    // Given independent durable stores with the same compiled native shapes and policies.
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (left_dir, left) = store(&catalog).await?;
    let (right_dir, right) = store(&catalog).await?;
    // When list/detail/sibling responses commit in opposite order.
    full(&left, &catalog, false).await?;
    full(&right, &catalog, true).await?;
    // Then canonical state is independent of network completion order.
    let left_state = state(&left_dir, &catalog).await?;
    let right_state = state(&right_dir, &catalog).await?;
    assert_eq!(left_state.0, right_state.0);
    assert_eq!(left_state.2, right_state.2);
    assert_eq!(
        left.projection_resource(&key()?).await?,
        right.projection_resource(&key()?).await?
    );
    assert_eq!(
        left.projection_edges_for(&key()?, None, 100).await?.edges,
        right.projection_edges_for(&key()?, None, 100).await?.edges
    );
    Ok(())
}

#[tokio::test]
async fn given_previous_detail_fields_when_that_source_omits_them_then_only_owned_values_and_edges_are_removed()
-> TestResult {
    // Given complete detail plus a disjoint sibling metadata facet.
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    full(&store, &catalog, false).await?;
    // When the authoritative detail no longer includes its name or settings association.
    persist(
        &store,
        &catalog,
        "GetPortal",
        json!({"portal":{"portalArn":PORTAL}}),
        10,
    )
    .await?;
    // Then stale list values cannot resurrect cleared detail fields; siblings remain.
    let cleared = state(&directory, &catalog).await?;
    assert_eq!(field(&cleared.0, 2)?, None);
    assert_eq!(field(&cleared.0, 3)?, None);
    assert_eq!(
        field(&cleared.0, 7)?,
        Some(Value::Text("<metadata/>".to_owned()))
    );
    assert_eq!(
        store
            .projection_resource(&key()?)
            .await?
            .ok_or("portal absent")?
            .name,
        None
    );
    assert_eq!(
        store.projection_edges_for(&key()?, None, 100).await?.edges,
        Vec::new()
    );
    // When a second sibling also clears its owned metadata, no stale value remains.
    persist(
        &store,
        &catalog,
        "GetPortalServiceProviderMetadata",
        json!({"portalArn":PORTAL}),
        11,
    )
    .await?;
    assert_eq!(field(&state(&directory, &catalog).await?.0, 7)?, None);
    Ok(())
}
