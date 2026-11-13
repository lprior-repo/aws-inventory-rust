//! all three load balancer types keep network topology without duplicate edges behavior.
use super::*;

#[test]
fn all_three_load_balancer_types_keep_network_topology_without_duplicate_edges()
-> Result<(), TestError> {
    let (ctx, _events) = context()?;
    for (kind, name, application) in [
        (LoadBalancerTypeEnum::Application, "application", true),
        (LoadBalancerTypeEnum::Network, "network", false),
        (LoadBalancerTypeEnum::Gateway, "gateway", false),
    ] {
        let model = LoadBalancer::builder()
            .load_balancer_arn(LB)
            .load_balancer_name("")
            .r#type(kind)
            .vpc_id("vpc-a")
            .security_groups("sg-a")
            .security_groups("sg-a")
            .availability_zones(
                AvailabilityZone::builder()
                    .zone_name("us-east-1a")
                    .subnet_id("subnet-a")
                    .build(),
            )
            .availability_zones(
                AvailabilityZone::builder()
                    .zone_name("us-east-1a")
                    .subnet_id("subnet-a")
                    .build(),
            )
            .build();
        let body = map_page(std::slice::from_ref(&model), 1, None, |model| {
            map_load_balancer(&ctx, model)
        })?;
        let resource = first_resource(&body)?;
        assert_eq!(resource.properties.get("type"), Some(&json!(name)));
        assert_eq!(resource.name.as_deref(), Some(""));
        assert_eq!(body.edges.len(), 3);
        assert_eq!(application_rules(&model), application);
        assert!(
            body.edges
                .iter()
                .all(|edge| edge.properties.get("owner_resolution")
                    == Some(&json!("assumed_scanning_account")))
        );
    }
    Ok(())
}

#[test]
fn listener_preserves_weighted_actions_and_deduplicates_forward_edges() -> Result<(), TestError> {
    let (ctx, _events) = context()?;
    let listener = Listener::builder()
        .listener_arn(LISTENER)
        .load_balancer_arn(LB)
        .protocol(ProtocolEnum::Https)
        .port(443)
        .default_actions(weighted_forward())
        .build();
    let body = map_listener(&ctx, LB, &listener)?;
    let resource = first_resource(&body)?;
    assert_eq!(
        body.edges
            .iter()
            .filter(|edge| edge.relation == "FORWARDS_TO")
            .count(),
        2
    );
    assert_eq!(
        body.edges
            .iter()
            .filter(|edge| edge.relation == "LISTENER_OF" && edge.to_id == LB)
            .count(),
        1
    );
    assert_eq!(
        resource
            .properties
            .pointer("/default_actions/0/forward_config/target_groups/0/weight"),
        Some(&json!(90))
    );
    assert_eq!(
        resource
            .properties
            .pointer("/default_actions/0/forward_config/target_groups/2/weight"),
        Some(&json!(0))
    );
    assert_eq!(
        resource.properties.get("certificate_list_complete"),
        Some(&json!(false))
    );
    Ok(())
}

#[test]
fn listener_rejects_contradictory_parent_in_response() -> Result<(), TestError> {
    let (ctx, _events) = context()?;
    let listener = Listener::builder()
        .listener_arn(LISTENER)
        .load_balancer_arn("different-parent")
        .build();
    assert!(matches!(
        map_listener(&ctx, LB, &listener),
        Err(AwsScanError::InvalidData(_))
    ));
    Ok(())
}

#[test]
fn unattached_target_group_remains_a_resource_with_health_configuration() -> Result<(), TestError> {
    let (ctx, _events) = context()?;
    let model = TargetGroup::builder()
        .target_group_arn(GROUP)
        .target_group_name("api")
        .target_type(TargetTypeEnum::Instance)
        .protocol(ProtocolEnum::Http)
        .port(8080)
        .vpc_id("vpc-a")
        .health_check_enabled(true)
        .health_check_path("/ready")
        .build();
    let body = map_target_group(&ctx, &model)?;
    assert_eq!(
        first_resource(&body)?.properties.get("health_check_path"),
        Some(&json!("/ready"))
    );
    assert_eq!(
        first_resource(&body)?.properties.get("load_balancer_arns"),
        Some(&json!([]))
    );
    assert_eq!(body.edges.len(), 1);
    assert!(body.edges.iter().all(|edge| edge.relation == "IN_VPC"));
    Ok(())
}

#[test]
fn target_group_relations_are_deduplicated() -> Result<(), TestError> {
    let (ctx, _events) = context()?;
    let model = TargetGroup::builder()
        .target_group_arn(GROUP)
        .target_type(TargetTypeEnum::Lambda)
        .load_balancer_arns(LB)
        .load_balancer_arns(LB)
        .build();
    let body = map_target_group(&ctx, &model)?;
    assert_eq!(body.edges.len(), 1);
    assert!(
        body.edges
            .iter()
            .all(|edge| edge.to_id == LB && edge.relation == "TARGET_GROUP_OF")
    );
    Ok(())
}

#[test]
fn rule_retains_conditions_rewrites_and_forwarding_weights() -> Result<(), TestError> {
    let (ctx, _events) = context()?;
    let rule = Rule::builder().rule_arn("arn:aws:elasticloadbalancing:us-east-1:111111111111:listener-rule/app/front/abc/def/ghi")
        .priority("10").is_default(false).actions(weighted_forward())
        .conditions(RuleCondition::builder().field("host-header")
            .host_header_config(HostHeaderConditionConfig::builder().values("api.example.test").regex_values("^api[.]example[.]test$").build()).build())
        .conditions(RuleCondition::builder().field("query-string")
            .query_string_config(QueryStringConditionConfig::builder().values(QueryStringKeyValuePair::builder().key("stage").value("canary").build()).build()).build())
        .transforms(RuleTransform::builder().r#type(TransformTypeEnum::UrlRewrite)
            .url_rewrite_config(UrlRewriteConfig::builder().rewrites(RewriteConfig::builder().regex("^/v1/(.*)$").replace("/$1").build()).build()).build())
        .build();
    let body = map_rule(&ctx, LISTENER, &rule)?;
    let resource = first_resource(&body)?;
    assert_eq!(
        resource
            .properties
            .pointer("/conditions/0/host_header_config/values/0"),
        Some(&json!("api.example.test"))
    );
    assert_eq!(
        resource
            .properties
            .pointer("/conditions/1/query_string_config/values/0/key"),
        Some(&json!("stage"))
    );
    assert_eq!(
        resource
            .properties
            .pointer("/transforms/0/url_rewrites/0/replace"),
        Some(&json!("/$1"))
    );
    assert_eq!(
        body.edges
            .iter()
            .filter(|edge| edge.relation == "FORWARDS_TO")
            .count(),
        2
    );
    assert!(
        body.edges
            .iter()
            .any(|edge| edge.relation == "RULE_OF" && edge.to_id == LISTENER)
    );
    Ok(())
}
