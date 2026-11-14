use super::*;

#[tokio::test]
pub(super) async fn successful_page_roundtrips_all_rows_across_multiple_sql_batches() -> TestResult
{
    let store = Store::open(":memory:").await?;
    store.begin_scan("many-rows", 1).await?;
    let resources: Vec<_> = (0..257_u32)
        .map(|index| resource(&format!("i-{index:03}"), Source::Direct, 10))
        .collect();
    let edges: Vec<_> = resources
        .iter()
        .map(|resource| Edge {
            from_id: resource.id.clone(),
            ..relationship()
        })
        .collect();
    let mut input = page("ec2", Source::Direct, resources.clone());
    input.edges = edges.clone();
    store.apply_page("many-rows", input.clone()).await?;
    store.apply_page("many-rows", input).await?;
    assert_eq!(store.resources("many-rows").await?, resources);
    assert_eq!(store.edges("many-rows").await?, edges);
    let summary = store
        .finish_scan("many-rows", ScanStatus::Complete, 20)
        .await?;
    assert_eq!(summary.resources, 257);
    assert_eq!(summary.edges, 257);
    assert_eq!(summary.completed_scopes, 1);
    store.close().await;
    Ok(())
}
