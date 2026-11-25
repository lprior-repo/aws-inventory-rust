use super::*;

pub(super) async fn run<S, F, G>(make: &F) -> TestResult
where
    S: InventoryStore,
    F: AsyncFn() -> TestResult<(G, S, Vec<u8>)>,
{
    let (_guard, store, _) = make().await?;
    given_late_unique_target_when_ambiguous_then_edges_are_retracted(&store).await?;
    let (_guard, store, _) = make().await?;
    given_reviewed_edge_when_target_changes_then_obsolete_edge_is_retracted(&store).await
}

async fn given_reviewed_edge_when_target_changes_then_obsolete_edge_is_retracted<
    S: InventoryStore,
>(
    store: &S,
) -> TestResult {
    let source = resource("direct-source")?;
    let target = resource("direct-target")?;
    let first_work = relationship_work("direct-before", "us-east-1", None, Some("beta"))?;
    store.seed_works(&[first_work]).await?;
    let lease = claim(store, "direct-before", 100).await?;
    let mut commit = page(
        lease,
        vec![
            observation("direct-target", "beta")?,
            observation("direct-source", "alpha")?,
        ],
        true,
        None,
        101,
    );
    commit.edges.push(inventory_core::inventory::Edge {
        from: source.clone(),
        to: target.clone(),
        relation: inventory_core::inventory::Relation::parse("depends-on")?,
        source: inventory_core::inventory::EdgeSource::ReviewedRule,
    });
    store.commit_page(commit).await?;
    let initial = store.projection_edges_for(&source, None, 10).await?;
    let [edge] = initial.edges.as_slice() else {
        return Err("expected reviewed edge before target change".into());
    };
    assert_eq!(edge.to, target);
    let next_work = relationship_work("direct-after", "us-east-1", None, Some("beta"))?;
    store.seed_works(&[next_work]).await?;
    let lease = claim(store, "direct-after", 200).await?;
    let result = store
        .commit_page(page(
            lease,
            vec![observation("direct-target", "gamma")?],
            true,
            None,
            201,
        ))
        .await?;
    assert_eq!(result.stats.resources_changed, 1);
    assert!(
        store
            .projection_edges_for(&source, None, 10)
            .await?
            .edges
            .is_empty()
    );
    let updated = store
        .projection_resource(&target)
        .await?
        .ok_or("updated target disappeared")?;
    assert_eq!(
        updated.content_hash.as_slice(),
        observation("direct-target", "gamma")?
            .content_hash()
            .as_bytes()
    );
    Ok(())
}

pub(super) async fn seed_source<S: InventoryStore>(store: &S) -> TestResult {
    let work = relationship_work("source", "us-east-1", Some("target-name"), None)?;
    store.seed_works(&[work]).await?;
    let lease = claim(store, "source", 100).await?;
    let observations = vec![observation("source", "source-name")?];
    let mut commit = page(lease, observations, true, None, 101);
    commit.unresolved.push(unresolved("source", "target-name")?);
    store.commit_page(commit).await?;
    let edges = store
        .projection_edges_for(&resource("source")?, None, 10)
        .await?;
    assert!(edges.edges.is_empty());
    Ok(())
}

pub(super) async fn observe_target<S: InventoryStore>(
    store: &S,
    sweep: &str,
    id: &str,
) -> TestResult {
    let work = relationship_work(sweep, "us-east-1", Some("target-name"), None)?;
    store.seed_works(&[work]).await?;
    let lease = claim(store, sweep, 200).await?;
    let observations = vec![
        observation("source", "source-name")?,
        observation(id, "target-name")?,
    ];
    let mut commit = page(lease, observations, true, None, 201);
    commit.unresolved.push(unresolved("source", "target-name")?);
    store.commit_page(commit).await?;
    Ok(())
}

async fn given_late_unique_target_when_ambiguous_then_edges_are_retracted<S: InventoryStore>(
    store: &S,
) -> TestResult {
    seed_source(store).await?;
    observe_target(store, "target-one", "target-one").await?;
    let key = resource("source")?;
    let unique = store.projection_edges_for(&key, None, 10).await?;
    let [edge] = unique.edges.as_slice() else {
        return Err("expected exactly one resolved reference".into());
    };
    assert_eq!(edge.from, key);
    assert_eq!(edge.to, resource("target-one")?);
    assert_eq!(edge.relation.as_str(), "references");
    observe_target(store, "target-two", "target-two").await?;
    for id in ["source", "target-one", "target-two"] {
        assert!(has(store, id).await?);
    }
    let ambiguous = store.projection_edges_for(&key, None, 10).await?;
    assert!(ambiguous.edges.is_empty());
    Ok(())
}
