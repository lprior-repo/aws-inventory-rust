use super::*;

#[tokio::test]
pub(super) async fn duplicate_resource_after_first_sql_batch_rolls_back_entire_page() -> TestResult
{
    let store = Store::open(":memory:").await?;
    store.begin_scan("batch-failure", 1).await?;
    let mut resources: Vec<_> = (0..128_u32)
        .map(|index| resource(&format!("i-{index}"), Source::Direct, 10))
        .collect();
    resources.push(resource("i-0", Source::Direct, 10));
    let input = page("ec2", Source::Direct, resources);
    let scope = input.scope.clone();
    assert!(store.apply_page("batch-failure", input).await.is_err());
    assert_eq!(store.summary("batch-failure").await?.resources, 0);
    assert!(store.checkpoint("batch-failure", &scope).await?.is_none());
    assert_eq!(
        store
            .query("SELECT count(*) AS pages FROM collection_pages", 1)
            .await?,
        json!([{"pages":0}])
    );
    store.close().await;
    Ok(())
}
