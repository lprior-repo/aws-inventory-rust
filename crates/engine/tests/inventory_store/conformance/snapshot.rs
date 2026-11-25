use super::*;
use inventory_core::inventory::{Tag, TagCoverage, TagObservation};
use inventory_engine::inventory::{ProjectionEdgePage, ProjectionResource};

pub(super) async fn run<S, F, G>(make: &F) -> TestResult
where
    S: InventoryStore,
    F: AsyncFn() -> TestResult<(G, S, Vec<u8>)>,
{
    let (_guard, store, _) = make().await?;
    given_snapshot_when_read_then_exact_data_is_returned(&store).await?;
    let (_guard, store, _) = make().await?;
    given_tags_when_snapshot_read_then_pagination_is_exact(&store).await
}

async fn given_snapshot_when_read_then_exact_data_is_returned<S: InventoryStore>(
    store: &S,
) -> TestResult {
    references::seed_source(store).await?;
    references::observe_target(store, "target-one", "target-one").await?;
    let mut snapshot = store.begin_projection_snapshot().await?;
    verify_resources(&mut snapshot).await?;
    let edges = verify_edges(&mut snapshot).await?;
    snapshot.finish().await?;
    let resources = store.projection_resources(None, 10).await?;
    assert_eq!(
        resources,
        vec![
            expected("source", "source-name")?,
            expected("target-one", "target-name")?
        ]
    );
    let after_source = store
        .projection_resources(Some(resource("source")?), 1)
        .await?;
    assert_eq!(after_source, vec![expected("target-one", "target-name")?]);
    assert_eq!(store.projection_edges(None, 10).await?, edges);
    Ok(())
}

fn expected(id: &str, value: &str) -> TestResult<ProjectionResource> {
    Ok(ProjectionResource {
        key: resource(id)?,
        name: None,
        content_hash: *observation(id, value)?.content_hash().as_bytes(),
    })
}

async fn verify_resources<P: ProjectionSnapshotRead>(snapshot: &mut P) -> TestResult {
    let first = snapshot.resources(None, 1).await?;
    assert_eq!(first, vec![expected("source", "source-name")?]);
    let source = resource("source")?;
    assert_eq!(snapshot.resource(&source).await?, first.first().cloned());
    let second = snapshot.resources(Some(source), 1).await?;
    assert_eq!(second, vec![expected("target-one", "target-name")?]);
    assert_eq!(snapshot.resource(&resource("missing")?).await?, None);
    let end = snapshot.resources(Some(resource("target-one")?), 1).await?;
    assert!(end.is_empty());
    Ok(())
}

async fn verify_edges<P: ProjectionSnapshotRead>(
    snapshot: &mut P,
) -> TestResult<ProjectionEdgePage> {
    let edges = snapshot.edges(None, 1).await?;
    let [edge] = edges.edges.as_slice() else {
        return Err("expected one snapshot edge".into());
    };
    assert_eq!(edge.from, resource("source")?);
    assert_eq!(edge.to, resource("target-one")?);
    assert_eq!(edge.relation.as_str(), "references");
    let source = resource("source")?;
    assert_eq!(snapshot.edges_for(&source, None, 1).await?, edges);
    let absent = snapshot.edges_for(&resource("missing")?, None, 1).await?;
    assert!(absent.edges.is_empty());
    Ok(edges)
}

async fn seed_tags<S: InventoryStore>(store: &S) -> TestResult {
    let lease = seed_claim(store, "tags", 100).await?;
    let observations = vec![observation("tagged", "alpha")?];
    let mut commit = page(lease, observations, true, None, 101);
    let tags = vec![Tag::new("a", "one")?, Tag::new("b", "two")?];
    commit.tags.push(TagObservation::new(
        resource("tagged")?,
        TagCoverage::Complete,
        tags,
    )?);
    store.commit_page(commit).await?;
    Ok(())
}

async fn given_tags_when_snapshot_read_then_pagination_is_exact<S: InventoryStore>(
    store: &S,
) -> TestResult {
    seed_tags(store).await?;
    let mut snapshot = store.begin_projection_snapshot().await?;
    let key = resource("tagged")?;
    let first = snapshot.tags_for(&key, None, 1).await?;
    let [tag] = first.tags.as_slice() else {
        return Err("expected one first-page tag".into());
    };
    assert_eq!((&*tag.key, &*tag.value), ("a", "one"));
    assert!(first.next.is_some());
    let second = snapshot.tags_for(&key, first.next, 1).await?;
    let [tag] = second.tags.as_slice() else {
        return Err("expected one second-page tag".into());
    };
    assert_eq!((&*tag.key, &*tag.value), ("b", "two"));
    assert_eq!(second.next, None);
    snapshot.finish().await?;
    Ok(())
}
