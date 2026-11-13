use super::super::tests::context;
use super::*;

#[test]
fn legacy_s3_locations_are_normalized_and_partition_checked() -> Result<(), AwsScanError> {
    assert_eq!(location_region("aws", None)?, "us-east-1");
    assert_eq!(location_region("aws", Some("EU"))?, "eu-west-1");
    assert_eq!(location_region("aws-cn", Some("cn-north-1"))?, "cn-north-1");
    assert!(location_region("aws", Some("cn-north-1")).is_err());
    assert!(location_region("aws-cn", None).is_err());
    assert!(location_region("aws", Some("https://wrong")).is_err());
    assert_eq!(
        location_header(301, Some("eu-west-1")),
        Some("eu-west-1".into())
    );
    assert_eq!(
        location_header(403, Some("us-east-1")),
        Some("us-east-1".into())
    );
    assert_eq!(location_header(503, Some("us-east-1")), None);
    assert_eq!(location_header(429, Some("us-east-1")), None);
    assert_eq!(location_header(400, Some("https://wrong")), None);
    Ok(())
}

#[test]
fn bucket_location_is_real_region_while_arn_stays_global() -> Result<(), Box<dyn std::error::Error>>
{
    let (ctx, _rx) = context()?;
    let bucket = Bucket::builder()
        .name("synthetic-bucket")
        .bucket_region("eu-west-1")
        .build();
    let body = map_bucket(&ctx, &bucket, Some("eu-west-1"), &[], Some(BTreeMap::new()))?;
    let resource = body.resources.first().ok_or("missing bucket")?;
    assert_eq!(resource.id, "arn:aws:s3:::synthetic-bucket");
    assert_eq!(resource.region, "eu-west-1");
    assert_eq!(
        resource.properties.get("details_available"),
        Some(&json!(false))
    );
    assert_eq!(
        resource.properties.get("object_data_collected"),
        Some(&json!(false))
    );
    Ok(())
}

#[test]
fn absent_and_denied_policy_sections_remain_distinct() -> Result<(), Box<dyn std::error::Error>> {
    let (ctx, _rx) = context()?;
    let bucket = Bucket::builder().name("synthetic-bucket").build();
    let denied = map_bucket(
        &ctx,
        &bucket,
        Some("us-east-1"),
        &[("policy_status", None)],
        Some(BTreeMap::new()),
    )?;
    let absent = map_bucket(
        &ctx,
        &bucket,
        Some("us-east-1"),
        &[("policy_status", Some(json!({"configured":false})))],
        Some(BTreeMap::new()),
    )?;
    assert_eq!(
        denied
            .resources
            .first()
            .and_then(|r| r.properties.get("policy_status")),
        Some(&Value::Null)
    );
    assert_eq!(
        absent
            .resources
            .first()
            .and_then(|r| r.properties.pointer("/policy_status/configured")),
        Some(&json!(false))
    );
    Ok(())
}

#[test]
fn bucket_architecture_edges_keep_explicit_arns_and_deduplicate()
-> Result<(), Box<dyn std::error::Error>> {
    let (ctx, _rx) = context()?;
    let bucket = Bucket::builder().name("synthetic-bucket").build();
    let sections = [
        (
            "encryption",
            Some(
                json!({"rules":[{"kms_key_id":"arn:aws:kms:us-east-1:123456789012:key/abc"},{"kms_key_id":"alias/unknown-owner"}]}),
            ),
        ),
        (
            "replication",
            Some(
                json!({"role_arn":"arn:aws:iam::123456789012:role/replication","rules":[{"destination_bucket":"arn:aws:s3:::remote-bucket"}]}),
            ),
        ),
        (
            "notifications",
            Some(
                json!({"topics":[{"arn":"arn:aws:sns:us-east-1:123456789012:topic"},{"arn":"arn:aws:sns:us-east-1:123456789012:topic"}]}),
            ),
        ),
        ("logging", Some(json!({"target_bucket":"log-bucket"}))),
    ];
    let body = map_bucket(
        &ctx,
        &bucket,
        Some("us-east-1"),
        &sections,
        Some(BTreeMap::new()),
    )?;
    assert_eq!(body.edges.len(), 5);
    assert!(
        body.edges
            .iter()
            .all(|e| !e.to_id.contains("alias/unknown-owner"))
    );
    let topic = body
        .edges
        .iter()
        .find(|e| e.relation == "NOTIFIES_TOPIC")
        .ok_or("missing topic edge")?;
    assert_eq!(
        topic
            .properties
            .get("evidence")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(2)
    );
    Ok(())
}

#[test]
fn missing_tags_and_sections_cannot_be_reported_complete() -> Result<(), Box<dyn std::error::Error>>
{
    let (ctx, _rx) = context()?;
    let bucket = Bucket::builder().name("synthetic-bucket").build();
    let sections: Vec<_> = SECTIONS
        .into_iter()
        .map(|key| (key, Some(json!({}))))
        .collect();
    let unavailable = map_bucket(&ctx, &bucket, Some("us-east-1"), &sections, None)?;
    let available = map_bucket(
        &ctx,
        &bucket,
        Some("us-east-1"),
        &sections,
        Some(BTreeMap::new()),
    )?;
    assert_eq!(
        unavailable
            .resources
            .first()
            .and_then(|r| r.properties.get("details_available")),
        Some(&json!(false))
    );
    assert_eq!(
        unavailable
            .resources
            .first()
            .and_then(|r| r.properties.get("unavailable_sections")),
        Some(&json!(["tags"]))
    );
    assert_eq!(
        available
            .resources
            .first()
            .and_then(|r| r.properties.get("details_available")),
        Some(&json!(true))
    );
    let repeated = [
        ("versioning", Some(json!({}))),
        ("versioning", Some(json!({}))),
    ];
    assert!(map_bucket(&ctx, &bucket, Some("us-east-1"), &repeated, None).is_err());
    Ok(())
}

#[test]
fn conflicting_s3_arn_is_not_silently_accepted() -> Result<(), Box<dyn std::error::Error>> {
    let (ctx, _rx) = context()?;
    let bucket = Bucket::builder()
        .name("one")
        .bucket_arn("arn:aws:s3:::two")
        .build();
    assert!(map_bucket(&ctx, &bucket, None, &[], None).is_err());
    Ok(())
}
