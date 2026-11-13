//! authentication action exposes presence without copying client secret behavior.
use super::*;

#[test]
fn authentication_action_exposes_presence_without_copying_client_secret() -> Result<(), TestError> {
    let action = Action::builder()
        .r#type(ActionTypeEnum::AuthenticateOidc)
        .authenticate_oidc_config(
            AuthenticateOidcActionConfig::builder()
                .issuer("https://issuer.example.test")
                .client_secret("fixture-secret-do-not-retain")
                .build(),
        )
        .build();
    let (actions, targets) = map_actions(&[action])?;
    let encoded = serde_json::to_string(&actions)?;
    assert!(!encoded.contains("fixture-secret-do-not-retain"));
    assert!(targets.is_empty());
    assert_eq!(
        actions
            .first()
            .and_then(|action| action.get("authenticate_oidc_config_present")),
        Some(&json!(true))
    );
    Ok(())
}

#[test]
fn health_membership_identity_includes_group_port_zone_and_quic_id() -> Result<(), TestError> {
    let (ctx, _events) = context()?;
    let mut ids = BTreeSet::new();
    for (group, port, zone, quic) in [
        (GROUP, Some(80), Some("us-east-1a"), None),
        (GROUP, Some(81), Some("us-east-1a"), None),
        (GROUP, Some(80), Some("us-east-1b"), None),
        (SECOND_GROUP, Some(80), Some("us-east-1a"), None),
        (GROUP, None, Some("us-east-1a"), None),
        (
            GROUP,
            Some(80),
            Some("us-east-1a"),
            Some("0x0123456789abcdef"),
        ),
    ] {
        let model = TargetHealthDescription::builder()
            .target(
                TargetDescription::builder()
                    .id("10.0.0.5")
                    .set_port(port)
                    .set_availability_zone(zone.map(str::to_owned))
                    .set_quic_server_id(quic.map(str::to_owned))
                    .build(),
            )
            .build();
        let body = map_target_health(&ctx, group, "ip", &model)?;
        ids.insert(first_resource(&body)?.id.clone());
        assert!(
            body.edges
                .iter()
                .any(|edge| edge.relation == "HAS_TARGET" && edge.from_id == group)
        );
    }
    assert_eq!(ids.len(), 6);
    Ok(())
}

#[test]
fn instance_target_is_an_explicit_account_assumption_and_keeps_unhealthy_state()
-> Result<(), TestError> {
    let (ctx, _events) = context()?;
    let body = map_target_health(
        &ctx,
        GROUP,
        "instance",
        &registered("i-123", Some(8080), None),
    )?;
    let edge = target_edge(&body)?;
    assert_eq!(
        edge.to_id,
        "arn:aws:ec2:us-east-1:111111111111:instance/i-123"
    );
    assert_eq!(
        edge.properties.get("owner_resolution"),
        Some(&json!("assumed_scanning_account"))
    );
    assert_eq!(
        first_resource(&body)?.properties.get("target_health_state"),
        Some(&json!("unhealthy"))
    );
    assert_eq!(
        first_resource(&body)?
            .properties
            .get("target_health_reason"),
        Some(&json!("Target.FailedHealthChecks"))
    );
    Ok(())
}

#[test]
fn ip_target_reference_has_unknown_owner_and_is_scoped_to_target_group() -> Result<(), TestError> {
    let (ctx, _events) = context()?;
    let first = map_target_health(
        &ctx,
        GROUP,
        "ip",
        &registered("10.0.0.5", Some(80), Some("all")),
    )?;
    let second = map_target_health(
        &ctx,
        SECOND_GROUP,
        "ip",
        &registered("10.0.0.5", Some(80), Some("all")),
    )?;
    let edge = target_edge(&first)?;
    assert!(edge.to_id.starts_with("aws-elbv2-ip-target:"));
    assert_ne!(edge.to_id, target_edge(&second)?.to_id);
    assert_eq!(
        edge.properties.get("owner_resolution"),
        Some(&json!("unknown"))
    );
    assert_eq!(edge.properties.get("target_region"), Some(&Value::Null));
    assert_eq!(
        edge.properties.get("target_address"),
        Some(&json!("10.0.0.5"))
    );
    assert_eq!(first.resources.len(), 1);
    assert!(
        first
            .resources
            .iter()
            .all(|resource| resource.kind == "aws_elbv2_target_health")
    );
    Ok(())
}

#[test]
fn returned_lambda_and_alb_arns_are_preserved_including_lambda_qualifier() -> Result<(), TestError>
{
    let (ctx, _events) = context()?;
    for (kind, id) in [
        (
            "lambda",
            "arn:aws:lambda:us-west-2:222222222222:function:processor:production",
        ),
        ("alb", LB),
    ] {
        let body = map_target_health(&ctx, GROUP, kind, &registered(id, None, Some("all")))?;
        let edge = target_edge(&body)?;
        assert_eq!(edge.to_id, id);
        assert_eq!(
            edge.properties.get("owner_resolution"),
            Some(&json!("returned_arn"))
        );
        assert_eq!(
            edge.properties.get("target_region").and_then(Value::as_str),
            id.split(':').nth(3)
        );
    }
    Ok(())
}

#[test]
fn unknown_future_target_type_remains_explicit_unresolved_evidence() -> Result<(), TestError> {
    let (ctx, _events) = context()?;
    let body = map_target_health(
        &ctx,
        GROUP,
        "future-service",
        &registered("opaque:identifier", None, None),
    )?;
    let edge = target_edge(&body)?;
    assert!(edge.to_id.starts_with("aws-elbv2-unknown-target:"));
    assert_eq!(
        edge.properties.get("target_type"),
        Some(&json!("future-service"))
    );
    assert_eq!(
        edge.properties.get("owner_resolution"),
        Some(&json!("unknown"))
    );
    Ok(())
}
