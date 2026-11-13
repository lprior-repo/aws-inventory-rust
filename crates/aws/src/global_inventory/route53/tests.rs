use super::super::tests::context;
use super::*;

#[test]
fn record_cursor_round_trip_keeps_routing_identifier() -> Result<(), Box<dyn std::error::Error>> {
    let output = ListResourceRecordSetsOutput::builder()
        .set_resource_record_sets(Some(Vec::new()))
        .is_truncated(true)
        .max_items(100)
        .next_record_name("weighted.example.")
        .next_record_type(RrType::A)
        .next_record_identifier("blue")
        .build()?;
    let token = next_record_cursor(&output)?.ok_or("missing token")?;
    let cursor = RecordCursor::decode(Some(&token))?.ok_or("missing cursor")?;
    assert_eq!(cursor.identifier.as_deref(), Some("blue"));
    assert_eq!(cursor.name, "weighted.example.");
    assert_eq!(cursor.record_type, "A");
    assert_eq!(RecordCursor::decode(None)?, None);
    assert!(
        RecordCursor::decode(Some(
            r#"{"name":"a","record_type":"A","identifier":null,"injected":true}"#
        ))
        .is_err()
    );
    let unicode = RecordCursor {
        name: "weighted.example.".into(),
        record_type: "A".into(),
        identifier: Some("東".repeat(128)),
    };
    unicode.validate()?;
    assert_eq!(
        RecordCursor::decode(Some(&serde_json::to_string(&unicode)?))?,
        Some(unicode.clone())
    );
    let oversized = RecordCursor {
        identifier: Some("東".repeat(129)),
        ..unicode
    };
    assert!(oversized.validate().is_err());
    Ok(())
}

#[test]
fn truncated_record_page_without_name_or_type_is_failure() -> Result<(), Box<dyn std::error::Error>>
{
    let output = ListResourceRecordSetsOutput::builder()
        .set_resource_record_sets(Some(Vec::new()))
        .is_truncated(true)
        .max_items(1)
        .next_record_name("example.")
        .build()?;
    assert!(next_record_cursor(&output).is_err());
    let done = ListResourceRecordSetsOutput::builder()
        .set_resource_record_sets(Some(Vec::new()))
        .is_truncated(false)
        .max_items(1)
        .next_record_name("stale.")
        .build()?;
    assert_eq!(next_record_cursor(&done)?, None);
    Ok(())
}

#[test]
fn weighted_records_have_distinct_stable_identities() -> Result<(), Box<dyn std::error::Error>> {
    let (ctx, _rx) = context()?;
    let blue = ResourceRecordSet::builder()
        .name("EXAMPLE.com.")
        .r#type(RrType::A)
        .set_identifier("blue")
        .build()?;
    let green = ResourceRecordSet::builder()
        .name("example.com.")
        .r#type(RrType::A)
        .set_identifier("green")
        .build()?;
    assert_ne!(
        record_identity(&ctx, "Z123", &blue)?,
        record_identity(&ctx, "Z123", &green)?
    );
    let same = ResourceRecordSet::builder()
        .name("example.com.")
        .r#type(RrType::A)
        .set_identifier("blue")
        .build()?;
    assert_eq!(
        record_identity(&ctx, "Z123", &blue)?,
        record_identity(&ctx, "Z123", &same)?
    );
    let without_dot = ResourceRecordSet::builder()
        .name("example.com")
        .r#type(RrType::A)
        .set_identifier("blue")
        .build()?;
    assert_eq!(
        record_identity(&ctx, "Z123", &blue)?,
        record_identity(&ctx, "Z123", &without_dot)?
    );
    let simple = ResourceRecordSet::builder()
        .name("example.com.")
        .r#type(RrType::A)
        .build()?;
    let sentinel = ResourceRecordSet::builder()
        .name("example.com.")
        .r#type(RrType::A)
        .set_identifier("<simple>")
        .build()?;
    assert_ne!(
        record_identity(&ctx, "Z123", &simple)?,
        record_identity(&ctx, "Z123", &sentinel)?
    );
    Ok(())
}

#[tokio::test]
async fn malformed_record_is_scoped_and_valid_sibling_survives()
-> Result<(), Box<dyn std::error::Error>> {
    let (ctx, mut rx) = context()?;
    let bad = ResourceRecordSet::builder()
        .name("bad.example.")
        .r#type(RrType::A)
        .resource_records(
            aws_sdk_route53::types::ResourceRecord::builder()
                .value("::1")
                .build()?,
        )
        .build()?;
    let good = ResourceRecordSet::builder()
        .name("good.example.")
        .r#type(RrType::A)
        .resource_records(
            aws_sdk_route53::types::ResourceRecord::builder()
                .value("192.0.2.1")
                .build()?,
        )
        .build()?;
    let bad_id = record_identity(&ctx, "Z123", &bad)?;
    assert!(
        map_record_scoped(&ctx, "Z123", &bad_id, &bad)
            .await?
            .is_none()
    );
    let good_id = record_identity(&ctx, "Z123", &good)?;
    assert!(
        map_record_scoped(&ctx, "Z123", &good_id, &good)
            .await?
            .is_some()
    );
    assert!(
        matches!(rx.try_recv()?, inventory_core::CollectionEvent::Failure(failure)
        if failure.scope.parent_id.as_deref() == Some(bad_id.as_str()))
    );
    assert!(rx.try_recv().is_err());
    Ok(())
}

#[test]
fn txt_values_are_omitted_with_explicit_count() -> Result<(), Box<dyn std::error::Error>> {
    let (ctx, _rx) = context()?;
    let record = ResourceRecordSet::builder()
        .name("token.example.")
        .r#type(RrType::Txt)
        .resource_records(
            aws_sdk_route53::types::ResourceRecord::builder()
                .value("secret-token-NEVER_PERSIST")
                .build()?,
        )
        .build()?;
    let id = record_identity(&ctx, "Z123", &record)?;
    let (resource, edges) = map_record(&ctx, "Z123", &id, &record)?;
    assert!(!serde_json::to_string(&resource)?.contains("NEVER_PERSIST"));
    assert_eq!(resource.properties.get("rdata_omitted"), Some(&json!(true)));
    assert_eq!(resource.properties.get("rdata_count"), Some(&json!(1)));
    assert_eq!(edges.len(), 1);
    Ok(())
}

#[test]
fn aliases_stay_dns_references_without_fabricated_load_balancer_identity()
-> Result<(), Box<dyn std::error::Error>> {
    let (ctx, _rx) = context()?;
    let record = ResourceRecordSet::builder()
        .name("service.example.")
        .r#type(RrType::A)
        .alias_target(
            aws_sdk_route53::types::AliasTarget::builder()
                .hosted_zone_id("ZALB")
                .dns_name("dualstack.example.us-east-1.elb.amazonaws.com.")
                .evaluate_target_health(true)
                .build()?,
        )
        .build()?;
    let id = record_identity(&ctx, "Z123", &record)?;
    let (_, edges) = map_record(&ctx, "Z123", &id, &record)?;
    assert_eq!(edges.len(), 2);
    assert!(
        edges
            .iter()
            .any(|edge| edge.to_id == "dns:dualstack.example.us-east-1.elb.amazonaws.com")
    );
    assert!(
        edges
            .iter()
            .all(|edge| !edge.to_id.contains(":loadbalancer/"))
    );
    Ok(())
}

#[test]
fn bad_address_family_is_rejected_and_zone_prefix_normalizes()
-> Result<(), Box<dyn std::error::Error>> {
    let (ctx, _rx) = context()?;
    assert_eq!(zone_id("/hostedzone/Z123")?, "Z123");
    assert!(zone_id("/hostedzone/").is_err());
    assert!(record_target(&ctx, "Z123", "A", "::1").is_err());
    assert!(record_target(&ctx, "Z123", "AAAA", "127.0.0.1").is_err());
    assert!(record_target(&ctx, "Z123", "AAAA", "2001:db8::1")?.is_some());
    Ok(())
}

#[test]
fn null_mx_retains_rdata_without_inventing_a_dns_destination()
-> Result<(), Box<dyn std::error::Error>> {
    let (ctx, _rx) = context()?;
    // RFC 7505 section 3: a sole MX record "0 ." declares no mail service.
    let record = ResourceRecordSet::builder()
        .name("nomail.example.")
        .r#type(RrType::Mx)
        .resource_records(
            aws_sdk_route53::types::ResourceRecord::builder()
                .value("0 .")
                .build()?,
        )
        .build()?;
    let identity = record_identity(&ctx, "Z123", &record)?;
    let (resource, edges) = map_record(&ctx, "Z123", &identity, &record)?;
    assert_eq!(resource.properties.get("records"), Some(&json!(["0 ."])));
    assert_eq!(
        resource.properties.get("rdata_omitted"),
        Some(&json!(false))
    );
    assert_eq!(edges.len(), 1);
    assert_eq!(
        edges.first().map(|edge| edge.relation.as_str()),
        Some("IN_HOSTED_ZONE")
    );
    Ok(())
}
