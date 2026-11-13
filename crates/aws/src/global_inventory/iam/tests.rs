use super::super::tests::context;
use super::*;

fn role() -> Result<Role, Box<dyn std::error::Error>> {
    Ok(Role::builder()
        .path("/application/")
        .role_name("worker")
        .role_id("AROAEXAMPLE")
        .arn("arn:aws:iam::123456789012:role/application/worker")
        .create_date(aws_smithy_types::DateTime::from_secs(1))
        .build()?)
}

#[test]
fn role_maps_identity_and_boundary_without_policy_payload() -> Result<(), Box<dyn std::error::Error>>
{
    let (ctx, _rx) = context()?;
    let mut value = role()?;
    value.permissions_boundary = Some(
        aws_sdk_iam::types::AttachedPermissionsBoundary::builder()
            .permissions_boundary_arn("arn:aws:iam::123456789012:policy/boundary")
            .build(),
    );
    value.assume_role_policy_document = Some(r#"{"Statement":{"Effect":"Allow","Principal":{"Service":"ecs-tasks.amazonaws.com"},"Action":"sts:AssumeRole","Condition":{"StringEquals":{"secret":"NEVER_PERSIST"}}}}"#.into());
    let body = map_role(&ctx, &value, ObservationDetail::Fetched)?;
    let resource = body.resources.first().ok_or("missing role")?;
    assert_eq!(resource.region, "global");
    assert_eq!(
        resource.properties.get("path").and_then(Value::as_str),
        Some("/application/")
    );
    let serialized = serde_json::to_string(&resource)?;
    assert!(!serialized.contains("NEVER_PERSIST"));
    assert_eq!(body.edges.len(), 2);
    assert!(
        body.edges
            .iter()
            .any(|edge| edge.relation == "USES_PERMISSIONS_BOUNDARY")
    );
    assert!(
        body.edges
            .iter()
            .any(|edge| edge.relation == "DECLARES_TRUST")
    );
    Ok(())
}

#[test]
fn list_only_roles_do_not_claim_detail_fields_or_trust_edges()
-> Result<(), Box<dyn std::error::Error>> {
    let (ctx, _rx) = context()?;
    let mut value = role()?;
    value.assume_role_policy_document = Some("malformed list-only document".into());
    let body = map_role(&ctx, &value, ObservationDetail::Listed)?;
    assert!(body.edges.is_empty());
    let resource = body.resources.first().ok_or("missing role")?;
    assert_eq!(
        resource.properties.get("details_available"),
        Some(&json!(false))
    );
    assert!(resource.tags.is_empty());
    Ok(())
}

#[test]
fn trust_parser_decodes_rfc3986_and_does_not_treat_plus_as_space() -> Result<(), AwsScanError> {
    let value = trust_summary(Some(
        "%7B%22Statement%22%3A%7B%22Effect%22%3A%22Allow%22%2C%22Principal%22%3A%7B%22Service%22%3A%22a+b.example%22%7D%2C%22Action%22%3A%22sts%3AAssumeRole%22%7D%7D",
    ))?;
    assert_eq!(
        value
            .first()
            .and_then(|v| v.pointer("/principals/Service/0"))
            .and_then(Value::as_str),
        Some("a+b.example")
    );
    assert!(trust_summary(Some(&"a".repeat(MAX_TRUST_BYTES.saturating_add(1)))).is_err());
    assert!(trust_summary(Some("%FF")).is_err());
    Ok(())
}

#[test]
fn deny_and_not_principal_never_become_allow_edges() -> Result<(), Box<dyn std::error::Error>> {
    let (ctx, _rx) = context()?;
    let mut value = role()?;
    value.assume_role_policy_document = Some(r#"{"Statement":[{"Effect":"Deny","Principal":"*","Action":"sts:AssumeRole"},{"Effect":"Allow","NotPrincipal":{"AWS":"*"},"Action":"sts:AssumeRole"}]}"#.into());
    assert!(
        map_role(&ctx, &value, ObservationDetail::Fetched)?
            .edges
            .is_empty()
    );
    Ok(())
}

#[test]
fn profile_links_exact_role_arn_and_policy_metadata_omits_document()
-> Result<(), Box<dyn std::error::Error>> {
    let (ctx, _rx) = context()?;
    let profile = InstanceProfile::builder()
        .path("/application/")
        .instance_profile_name("worker")
        .instance_profile_id("AIPAEXAMPLE")
        .arn("arn:aws:iam::123456789012:instance-profile/application/worker")
        .create_date(aws_smithy_types::DateTime::from_secs(2))
        .roles(role()?)
        .build()?;
    let body = map_profile(&ctx, &profile, ObservationDetail::Fetched)?;
    assert_eq!(
        body.edges.first().map(|e| e.to_id.as_str()),
        Some("arn:aws:iam::123456789012:role/application/worker")
    );
    let policy = Policy::builder()
        .policy_name("custom")
        .arn("arn:aws:iam::123456789012:policy/application/custom")
        .default_version_id("v3")
        .attachment_count(4)
        .is_attachable(true)
        .build();
    let body = map_policy(&ctx, &policy, ObservationDetail::Fetched)?;
    assert_eq!(
        body.resources
            .first()
            .and_then(|r| r.properties.get("policy_document_collected")),
        Some(&json!(false))
    );
    assert_eq!(
        body.resources
            .first()
            .and_then(|r| r.properties.get("default_version_id")),
        Some(&json!("v3"))
    );
    Ok(())
}

#[test]
fn own_iam_identities_cannot_silently_change_account() -> Result<(), Box<dyn std::error::Error>> {
    let (ctx, _rx) = context()?;
    let mut value = role()?;
    value.arn = "arn:aws:iam::999999999999:role/other".into();
    assert!(map_role(&ctx, &value, ObservationDetail::Fetched).is_err());
    Ok(())
}
