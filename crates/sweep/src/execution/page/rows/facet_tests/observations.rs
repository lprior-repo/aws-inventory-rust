use super::*;
#[tokio::test]
async fn given_native_list_detail_and_metadata_when_sparse_list_repeats_then_canonical_state_and_edges_survive()
-> TestResult {
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    full(&store, &catalog, false).await?;
    let before = state(&directory, &catalog).await?;
    persist(
        &store,
        &catalog,
        "ListPortals",
        json!({"portals":[{"portalArn":PORTAL}]}),
        10,
    )
    .await?;
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
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (left_dir, left) = store(&catalog).await?;
    let (right_dir, right) = store(&catalog).await?;
    full(&left, &catalog, false).await?;
    full(&right, &catalog, true).await?;
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
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let (directory, store) = store(&catalog).await?;
    full(&store, &catalog, false).await?;
    persist(
        &store,
        &catalog,
        "GetPortal",
        json!({"portal":{"portalArn":PORTAL}}),
        10,
    )
    .await?;
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

#[test]
fn given_named_kind_when_owned_row_is_assembled_then_nonempty_name_matches_legacy() -> TestResult {
    let bytes = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup("workspaces-web", "ListPortals")
        .ok_or("portal plan absent")?;
    let kind = plan.kind();
    let name_path = catalog.name_path(kind).ok_or("portal name path absent")?;
    assert_eq!(name_path.path.len(), 1);
    let name_field = name_path.path.first().ok_or("name field absent")?;
    assert_eq!(catalog.catalog().field_name(*name_field)?, "displayName");
    let rows = named_portal_rows(plan)?;
    assert_eq!(rows.len(), 1);
    let row = rows.first().ok_or("portal row absent")?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let identity = || record::IdentityContext {
        account: &account,
        region: &home,
    };
    let region = Region::Regional(home.clone());
    let request = Record::encode(&Value::Map(BTreeMap::new()))?;
    let context = DiscoveryContext::empty();
    let legacy = observation(
        &catalog,
        identity(),
        plan,
        &region,
        row.clone(),
        &request,
        &context,
    )?;
    assert_eq!(legacy.name(), Some("owned portal"));
    let prepared = identities(
        &catalog,
        identity(),
        plan,
        &region,
        &rows,
        &request,
        &[context],
    )?;
    let moved = observations(
        rows,
        prepared,
        plan.hash_exclusions(),
        plan.facet_precedence(),
    )?;
    let actual = moved.first().ok_or("owned observation absent")?;
    assert_eq!(actual.name(), Some("owned portal"));
    assert_eq!(actual.name(), legacy.name());
    Ok(())
}

fn named_portal_rows(plan: &CallablePlan<'_>) -> TestResult<Vec<Record>> {
    let body = serde_json::to_vec(&json!({"portals":[{
        "portalArn":PORTAL, "displayName":"owned portal"
    }]}))?;
    let response = aws_wire::response::decode(aws_wire::response::ResponseContext {
        service: plan.service(),
        operation: plan.operation(),
        protocol: aws_wire::request::Protocol::RestJson1,
        body: &body,
        headers: &http::HeaderMap::new(),
        status: http::StatusCode::OK,
        limits: aws_wire::response::DecodeLimits::default(),
    })?;
    Ok(record::item_records(&response, plan.items())?)
}
