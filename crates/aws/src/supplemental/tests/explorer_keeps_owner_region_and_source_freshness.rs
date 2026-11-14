use super::*;

#[test]
fn explorer_keeps_owner_region_and_source_freshness() -> Result<(), Box<dyn std::error::Error>> {
    let tag = Document::Object(
        [
            ("Key".to_owned(), Document::String("Name".to_owned())),
            ("Value".to_owned(), Document::String("worker".to_owned())),
        ]
        .into_iter()
        .collect(),
    );
    let source = aws_sdk_resourceexplorer2::types::Resource::builder()
        .arn("arn:aws:ec2:eu-west-1:222222222222:instance/i-abc")
        .owning_account_id("222222222222")
        .region("eu-west-1")
        .resource_type("ec2:instance")
        .service("ec2")
        .last_reported_at(DateTime::from_secs(10))
        .properties(
            aws_sdk_resourceexplorer2::types::ResourceProperty::builder()
                .name("tags")
                .data(Document::Array(vec![tag]))
                .build(),
        )
        .build();
    let resource = map_explorer(&source, 20_000)?;
    assert_eq!(resource.account_id, "222222222222");
    assert_eq!(resource.region, "eu-west-1");
    assert_eq!(resource.name.as_deref(), Some("worker"));
    assert_eq!(resource.observed_at, 20_000);
    assert_eq!(
        resource
            .properties
            .get("last_reported_at")
            .and_then(serde_json::Value::as_i64),
        Some(10_000)
    );
    assert_eq!(resource.source, Source::ResourceExplorer);
    Ok(())
}
