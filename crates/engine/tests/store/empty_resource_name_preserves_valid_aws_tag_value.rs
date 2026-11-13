//! Behavior: empty_resource_name_preserves_valid_aws_tag_value.
use super::*;

#[tokio::test]
pub(super) async fn empty_resource_name_preserves_valid_aws_tag_value() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("empty-name", 1).await?;
    let mut instance = resource("i-empty-name", Source::Direct, 10);
    instance.name = Some(String::new());
    instance.tags.insert("Name".to_owned(), String::new());
    store
        .apply_page(
            "empty-name",
            page("ec2", Source::Direct, vec![instance.clone()]),
        )
        .await?;
    assert_eq!(store.resources("empty-name").await?, vec![instance]);
    store.close().await;
    Ok(())
}
