use super::*;

#[tokio::test]
pub(super) async fn equal_timestamp_source_ties_ignore_page_arrival_order() -> TestResult {
    let first = Store::open(":memory:").await?;
    let second = Store::open(":memory:").await?;
    first.begin_scan("one", 1).await?;
    second.begin_scan("one", 1).await?;
    let mut a = resource("i-1", Source::Direct, 10);
    a.name = Some("alpha".to_owned());
    let mut b = a.clone();
    b.name = Some("beta".to_owned());
    let alpha = page("a", Source::Direct, vec![a]);
    let beta = page("b", Source::Direct, vec![b]);
    first.apply_page("one", alpha.clone()).await?;
    first.apply_page("one", beta.clone()).await?;
    second.apply_page("one", beta.clone()).await?;
    second.apply_page("one", alpha.clone()).await?;
    assert_eq!(
        first.resources("one").await?,
        second.resources("one").await?
    );
    assert_eq!(
        first.query("SELECT name FROM latest_resources", 10).await?,
        json!([{"name":"alpha"}])
    );
    Ok(())
}
