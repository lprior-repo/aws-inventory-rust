use super::*;
pub(in crate::execution::page::child::binding_tests) type Resource =
    (String, String, String, String);
pub(in crate::execution::page::child::binding_tests) type Edge = (Resource, String, Resource);
pub(in crate::execution::page::child::binding_tests) fn key(
    region: &str,
    kind: &str,
    id: &str,
) -> Resource {
    (
        "123456789012".to_owned(),
        region.to_owned(),
        kind.to_owned(),
        id.to_owned(),
    )
}
fn edge_key(value: &ResourceKey) -> Resource {
    (
        value.account.as_str().to_owned(),
        value.region.as_str().to_owned(),
        value.kind.as_str().to_owned(),
        value.id.as_str().to_owned(),
    )
}
pub(in crate::execution::page::child::binding_tests) async fn assert_state(
    path: &Path,
    catalog: &RuntimeCatalog<'_>,
    resources: &BTreeSet<Resource>,
    edges: &BTreeSet<Edge>,
) -> TestResult {
    let reader = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(path)
                .read_only(true),
        )
        .await?;
    let saved: Vec<Resource> = sqlx::query_as(
        "SELECT account,region,kind,id FROM resources ORDER BY account,region,kind,id",
    )
    .fetch_all(&reader)
    .await?;
    assert_eq!(
        saved.into_iter().collect::<BTreeSet<_>>(),
        *resources,
        "canonical ResourceKeys differ"
    );
    reader.close().await;
    let store = BinaryStore::open_with_catalog(path, catalog.raw_bytes()).await?;
    let saved = store
        .projection_edges(None, 1000)
        .await?
        .edges
        .into_iter()
        .map(|edge| {
            (
                edge_key(&edge.from),
                edge.relation.as_str().to_owned(),
                edge_key(&edge.to),
            )
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(saved, *edges, "actual resolved graph edges differ");
    Ok(())
}

pub(in crate::execution::page::child::binding_tests) async fn unbranched_source(
    catalog: &RuntimeCatalog<'_>,
) -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("source-stage.db");
    let store = BinaryStore::create(
        &path,
        SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?,
    )
    .await?;
    let sweep = SweepId::new("unbranched-source-stage".to_owned())?;
    seed(&store, catalog, &sweep, false).await?;
    let lease = store
        .claim(&sweep, 1001, 10000)
        .await?
        .ok_or("producer stage absent")?;
    let producer = catalog
        .lookup("bindings", "ListParents0")
        .ok_or("producer plan absent")?;
    assert_eq!(lease.scope().branch(), Some("producer-stage"));
    let (request, _) = wire(producer, &lease)?;
    apply(
        &store,
        catalog,
        producer,
        lease,
        &request,
        json!({"Items":[{"Id":"owner"}]}),
    )
    .await?;
    drop(store);
    let store = BinaryStore::open_with_catalog(&path, catalog.raw_bytes()).await?;
    let lease = store
        .claim(&sweep, 1001, 10000)
        .await?
        .ok_or("unbranched child absent")?;
    assert_eq!(lease.scope().operation().as_str(), "ListChildren");
    assert!(
        lease.scope().branch().is_none(),
        "producer stage masqueraded as target stage"
    );
    let ancestry = preparation::ancestry(lease.context(), lease.scope().context())?;
    let source_path = producer
        .context_projection()
        .selected
        .first()
        .ok_or("source projection absent")?;
    assert!(matches!(
        ancestry.value_from_branch(
            "bindings",
            "ListParents0",
            "producer-stage",
            inventory_core::catalog::runtime::ContextSource::SelectedItem,
            source_path
        )?,
        Some(ValueRef::Text("owner"))
    ));
    let child = catalog
        .lookup("bindings", "ListChildren")
        .ok_or("child plan absent")?;
    let (request, wire) = wire(child, &lease)?;
    assert_eq!(
        serde_json::from_slice::<Json>(wire.body())?,
        json!({"OrganizationId":"org-0","EntityId":"owner","Regions":["us-west-2"]})
    );
    apply(
        &store,
        catalog,
        child,
        lease,
        &request,
        json!({"Items":[{"Id":"descendant"}]}),
    )
    .await?;
    drop(store);
    let parent = key("us-west-2", "AWS::ApiOnly::BindingParent", "owner");
    let child = key("us-west-2", "AWS::ApiOnly::BindingChild", "descendant");
    assert_state(
        &path,
        catalog,
        &BTreeSet::from([parent.clone(), child.clone()]),
        &BTreeSet::from([(child, "IN_PARENT".to_owned(), parent)]),
    )
    .await?;
    Ok(())
}
