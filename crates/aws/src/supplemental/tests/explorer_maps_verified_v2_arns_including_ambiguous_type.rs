use super::*;

#[test]
fn explorer_maps_verified_v2_arns_including_ambiguous_type()
-> Result<(), Box<dyn std::error::Error>> {
    for (resource_type, path, kind) in [
        (
            "elasticloadbalancing:loadbalancer",
            "loadbalancer/app/example/lb-id",
            "aws_elbv2_load_balancer",
        ),
        (
            "elasticloadbalancing:loadbalancer",
            "loadbalancer/net/example/lb-id",
            "aws_elbv2_load_balancer",
        ),
        (
            "elasticloadbalancing:loadbalancer",
            "loadbalancer/gwy/example/lb-id",
            "aws_elbv2_load_balancer",
        ),
        (
            "elasticloadbalancing:loadbalancer/app",
            "loadbalancer/app/example/lb-id",
            "aws_elbv2_load_balancer",
        ),
        (
            "elasticloadbalancing:loadbalancer/net",
            "loadbalancer/net/example/lb-id",
            "aws_elbv2_load_balancer",
        ),
        (
            "elasticloadbalancing:loadbalancer/gwy",
            "loadbalancer/gwy/example/lb-id",
            "aws_elbv2_load_balancer",
        ),
        (
            "elasticloadbalancingv2:loadbalancer",
            "loadbalancer/app/example/lb-id",
            "aws_elbv2_load_balancer",
        ),
        (
            "elasticloadbalancing:listener/app",
            "listener/app/example/lb-id/listener-id",
            "aws_elbv2_listener",
        ),
        (
            "elasticloadbalancing:listener/net",
            "listener/net/example/lb-id/listener-id",
            "aws_elbv2_listener",
        ),
        (
            "elasticloadbalancing:listener/gwy",
            "listener/gwy/example/lb-id/listener-id",
            "aws_elbv2_listener",
        ),
        (
            "elasticloadbalancing:listener-rule/app",
            "listener-rule/app/example/lb-id/listener-id/rule-id",
            "aws_elbv2_rule",
        ),
        (
            "elasticloadbalancing:listener-rule/net",
            "listener-rule/net/example/lb-id/listener-id/rule-id",
            "aws_elbv2_rule",
        ),
        (
            "elasticloadbalancing:targetgroup",
            "targetgroup/example/group-id",
            "aws_elbv2_target_group",
        ),
    ] {
        let arn = format!("arn:aws-us-gov:elasticloadbalancing:us-gov-west-1:222222222222:{path}");
        let resource = map_explorer(&explorer_fixture(resource_type, &arn), 100)?;
        assert_eq!(resource.kind, kind, "{resource_type}: {path}");
        assert_eq!(resource.id, arn);
        assert_eq!(resource.source, Source::ResourceExplorer);
        for absent in [
            "type",
            "vpc_id",
            "port",
            "default_actions",
            "target_health_state",
            "edges",
        ] {
            assert!(resource.properties.get(absent).is_none());
        }
    }
    Ok(())
}

#[test]
fn explorer_never_promotes_classic_or_mismatched_arns_to_v2()
-> Result<(), Box<dyn std::error::Error>> {
    for (resource_type, path) in [
        (
            "elasticloadbalancing:loadbalancer",
            "loadbalancer/classic-example",
        ),
        ("elasticloadbalancing:loadbalancer", "loadbalancer/app"),
        (
            "elasticloadbalancing:loadbalancer",
            "loadbalancer/app/example",
        ),
        (
            "elasticloadbalancing:loadbalancer",
            "loadbalancer/app//lb-id",
        ),
        (
            "elasticloadbalancing:loadbalancer",
            "loadbalancer/app/example/lb-id/extra",
        ),
        (
            "elasticloadbalancing:loadbalancer",
            "loadbalancer/unknown/example/lb-id",
        ),
        (
            "elasticloadbalancing:loadbalancer",
            "targetgroup/example/group-id",
        ),
        (
            "elasticloadbalancing:loadbalancer/app",
            "loadbalancer/net/example/lb-id",
        ),
        (
            "elasticloadbalancing:listener/app",
            "loadbalancer/app/example/lb-id",
        ),
        ("elasticloadbalancing:targetgroup", "targetgroup/example"),
        (
            "elasticloadbalancingv2:loadbalancer",
            "loadbalancer/classic-example",
        ),
    ] {
        let arn = format!("arn:aws:elasticloadbalancing:eu-west-1:222222222222:{path}");
        let resource = map_explorer(&explorer_fixture(resource_type, &arn), 100)?;
        assert!(
            !resource.kind.starts_with("aws_elbv2_"),
            "{resource_type}: {path}"
        );
        assert_eq!(resource.id, arn);
        assert_eq!(
            resource.properties.get("resource_type"),
            Some(&json!(resource_type))
        );
    }
    for arn in [
        "arn:aws:ec2:eu-west-1:222222222222:loadbalancer/app/example/lb-id",
        "arn::elasticloadbalancing:eu-west-1:222222222222:loadbalancer/app/example/lb-id",
        "arn:aws:elasticloadbalancing::222222222222:loadbalancer/app/example/lb-id",
        "arn:aws:elasticloadbalancing:eu-west-1::loadbalancer/app/example/lb-id",
    ] {
        let resource = map_explorer(
            &explorer_fixture("elasticloadbalancing:loadbalancer", arn),
            100,
        )?;
        assert_eq!(resource.kind, "aws_elasticloadbalancing_loadbalancer");
    }
    Ok(())
}

#[test]
fn config_uses_resource_owner_not_aggregator_account() -> Result<(), Box<dyn std::error::Error>> {
    let row = r#"{"accountId":"222222222222","awsRegion":"eu-west-1","arn":"arn:aws:ec2:eu-west-1:222222222222:instance/i-abc","resourceType":"AWS::EC2::Instance","resourceId":"i-abc","configuration":{"instanceId":"i-abc","vpcId":"vpc-abc"},"tags":[{"key":"Name","value":"example"}]}"#;
    let resource = parse_config_row(row, 100)?;
    assert_eq!(resource.account_id, "222222222222");
    assert_eq!(resource.region, "eu-west-1");
    assert_eq!(resource.kind, "aws_ec2_instance");
    assert_eq!(resource.source, Source::AwsConfig);
    assert_eq!(resource.name.as_deref(), Some("example"));
    assert_eq!(
        resource
            .properties
            .get("vpc_id")
            .and_then(serde_json::Value::as_str),
        Some("vpc-abc")
    );
    Ok(())
}

#[test]
fn accepts_config_string_configuration_and_rejects_malformed_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let row = r#"{"accountId":"222222222222","awsRegion":"global","resourceType":"AWS::S3::Bucket","resourceId":"example","configuration":"{\"name\":\"example\"}"}"#;
    let resource = parse_config_row(row, 100)?;
    assert!(resource.id.starts_with("aws-config:"));
    assert!(parse_config_row("{}", 100).is_err());
    assert_eq!(
        resource_kind("ec2:security-group"),
        "aws_vpc_security_group"
    );
    assert_eq!(resource_kind("AWS::EKS::Nodegroup"), "aws_eks_node_group");
    Ok(())
}
