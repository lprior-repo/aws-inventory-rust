use super::*;

#[tokio::test]
pub(super) async fn quoted_identifiers_and_values_are_bound_in_ingestion_paths() -> TestResult {
    let store = Store::open(":memory:").await?;
    let scan = "scan'); DROP TABLE scans; --";
    store.begin_scan(scan, 1).await?;
    let input = page(
        "ec2'",
        Source::Direct,
        vec![resource("i-'quoted", Source::Direct, 10)],
    );
    store.apply_page(scan, input.clone()).await?;
    assert_eq!(store.resources(scan).await?, input.resources);
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM scans", 10)
            .await?,
        json!([{"total":1}])
    );
    Ok(())
}
