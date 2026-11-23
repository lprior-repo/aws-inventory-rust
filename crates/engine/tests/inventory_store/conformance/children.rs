use super::*;

pub(super) async fn run<S, F, G>(make: &F) -> TestResult
where
    S: InventoryStore,
    F: AsyncFn() -> TestResult<(G, S, Vec<u8>)>,
{
    let (_guard, store, _) = make().await?;
    given_parent_when_missing_twice_then_children_are_deleted(&store).await
}

async fn seed_parent_and_child<S: InventoryStore>(store: &S) -> TestResult {
    observe(store, "parent-seed", "parent", "root").await?;
    let work = child_work("child-seed", &resource("parent")?)?;
    store.seed_works(&[work]).await?;
    let child = claim(store, "child-seed", 200).await?;
    let observations = vec![observation("child", "leaf")?];
    let commit = page(child, observations, true, None, 201);
    store.commit_page(commit).await?;
    assert!(has(store, "parent").await?);
    assert!(has(store, "child").await?);
    Ok(())
}

async fn miss_parent<S: InventoryStore>(store: &S, sweep: &str) -> TestResult<u64> {
    let lease = seed_claim(store, sweep, 300).await?;
    let commit = page(lease, Vec::new(), true, None, 301);
    Ok(store.commit_page(commit).await?.stats.resources_deleted)
}

async fn given_parent_when_missing_twice_then_children_are_deleted<S: InventoryStore>(
    store: &S,
) -> TestResult {
    seed_parent_and_child(store).await?;
    assert_eq!(miss_parent(store, "parent-miss-one").await?, 0);
    assert!(has(store, "parent").await?);
    assert!(has(store, "child").await?);
    assert_eq!(miss_parent(store, "parent-miss-two").await?, 2);
    assert!(!has(store, "parent").await?);
    assert!(!has(store, "child").await?);
    Ok(())
}
