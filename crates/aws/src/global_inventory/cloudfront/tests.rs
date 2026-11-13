use super::super::tests::context;
use super::*;
use aws_sdk_cloudfront::types::{HttpVersion, PriceClass, ViewerProtocolPolicy};

fn distribution(origin: Origin) -> Result<DistributionSummary, Box<dyn std::error::Error>> {
    Ok(DistributionSummary::builder()
        .id("E123")
        .arn("arn:aws:cloudfront::123456789012:distribution/E123")
        .status("Deployed")
        .last_modified_time(aws_smithy_types::DateTime::from_secs(1))
        .domain_name("d123.cloudfront.net")
        .comment("synthetic")
        .price_class(PriceClass::PriceClass100)
        .enabled(true)
        .web_acl_id("")
        .http_version(HttpVersion::Http2)
        .is_ipv6_enabled(true)
        .staging(false)
        .origins(
            aws_sdk_cloudfront::types::Origins::builder()
                .quantity(1)
                .items(origin)
                .build()?,
        )
        .default_cache_behavior(
            DefaultCacheBehavior::builder()
                .target_origin_id("primary")
                .viewer_protocol_policy(ViewerProtocolPolicy::RedirectToHttps)
                .build()?,
        )
        .build()?)
}

#[test]
fn s3_origin_patterns_are_explicit_and_do_not_accept_lookalikes() {
    assert_eq!(
        s3_bucket_from_dns("aws", "a.b.s3.us-east-1.amazonaws.com"),
        Some("a.b")
    );
    assert_eq!(
        s3_bucket_from_dns("aws", "bucket.s3.amazonaws.com"),
        Some("bucket")
    );
    assert_eq!(
        s3_bucket_from_dns("aws", "bucket.s3-website-us-east-1.amazonaws.com"),
        Some("bucket")
    );
    assert_eq!(
        s3_bucket_from_dns("aws", "bucket.s3.dualstack.us-east-1.amazonaws.com"),
        Some("bucket")
    );
    assert_eq!(
        s3_bucket_from_dns("aws", "endpoint.s3-accesspoint.us-east-1.amazonaws.com"),
        None
    );
    assert_eq!(
        s3_bucket_from_dns("aws", "bucket.s3.amazonaws.com.evil.example"),
        None
    );
    assert_eq!(
        s3_bucket_from_dns("aws", "bucket.s3.cn-north-1.amazonaws.com.cn"),
        None
    );
}

#[test]
fn custom_header_values_and_origin_paths_never_enter_inventory()
-> Result<(), Box<dyn std::error::Error>> {
    let (ctx, _rx) = context()?;
    let origin = Origin::builder()
        .id("primary")
        .domain_name("bucket.s3.us-east-1.amazonaws.com")
        .origin_path("/NEVER_PERSIST_PATH")
        .custom_headers(
            aws_sdk_cloudfront::types::CustomHeaders::builder()
                .quantity(1)
                .items(
                    aws_sdk_cloudfront::types::OriginCustomHeader::builder()
                        .header_name("Authorization")
                        .header_value("NEVER_PERSIST_TOKEN")
                        .build()?,
                )
                .build()?,
        )
        .build()?;
    let body = map_distribution(&ctx, &distribution(origin)?, None)?;
    let serialized = serde_json::to_string(&body.resources)?;
    assert!(!serialized.contains("NEVER_PERSIST"));
    assert_eq!(
        body.edges.first().map(|e| e.to_id.as_str()),
        Some("arn:aws:s3:::bucket")
    );
    let resource = body.resources.first().ok_or("missing distribution")?;
    assert_eq!(resource.region, "global");
    assert_eq!(
        resource
            .properties
            .pointer("/origins/0/custom_header_count"),
        Some(&json!(1))
    );
    assert_eq!(
        resource.properties.get("tags_available"),
        Some(&json!(false))
    );
    Ok(())
}

#[test]
fn distribution_uses_actual_certificate_arn_and_preserves_behavior_routing()
-> Result<(), Box<dyn std::error::Error>> {
    let (ctx, _rx) = context()?;
    let origin = Origin::builder()
        .id("primary")
        .domain_name("backend.example.com")
        .build()?;
    let mut value = distribution(origin)?;
    value.viewer_certificate = Some(
        aws_sdk_cloudfront::types::ViewerCertificate::builder()
            .acm_certificate_arn("arn:aws:acm:us-east-1:123456789012:certificate/id")
            .build(),
    );
    let body = map_distribution(&ctx, &value, Some(BTreeMap::new()))?;
    assert!(body.edges.iter().any(|e| e.relation == "USES_CERTIFICATE"));
    assert!(
        body.edges
            .iter()
            .any(|e| e.to_id == "dns:backend.example.com")
    );
    let resource = body.resources.first().ok_or("missing distribution")?;
    assert_eq!(
        resource
            .properties
            .pointer("/default_cache_behavior/target_origin_id"),
        Some(&json!("primary"))
    );
    assert_eq!(
        resource
            .properties
            .pointer("/default_cache_behavior/target_in_configuration"),
        Some(&json!(true))
    );
    Ok(())
}

#[test]
fn duplicate_origins_and_cross_account_source_arns_fail() -> Result<(), Box<dyn std::error::Error>>
{
    let (ctx, _rx) = context()?;
    let origin = Origin::builder()
        .id("primary")
        .domain_name("backend.example.com")
        .build()?;
    let mut value = distribution(origin.clone())?;
    value.origins = Some(
        aws_sdk_cloudfront::types::Origins::builder()
            .quantity(2)
            .items(origin.clone())
            .items(origin)
            .build()?,
    );
    assert!(map_distribution(&ctx, &value, None).is_err());
    value.arn = "arn:aws:cloudfront::999999999999:distribution/E123".into();
    assert!(map_distribution(&ctx, &value, None).is_err());
    Ok(())
}

#[tokio::test]
async fn unreviewed_cloudfront_partition_is_explicit_failure_without_api_calls()
-> Result<(), Box<dyn std::error::Error>> {
    let (mut ctx, mut rx) = context()?;
    ctx.partition = "aws-us-gov".into();
    collect(&ctx).await?;
    assert_eq!(ctx.executor.stats().attempts, 0);
    let event = rx.try_recv()?;
    assert!(
        matches!(event, inventory_core::CollectionEvent::Failure(failure) if failure.category == "unsupported_partition")
    );
    Ok(())
}
