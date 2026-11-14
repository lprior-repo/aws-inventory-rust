use super::*;

#[test]
fn config_maps_all_six_network_kinds() -> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        (
            "AWS::EC2::RouteTable",
            "aws_vpc_route_table",
            "routeTableId",
            "route_table_id",
        ),
        (
            "AWS::EC2::InternetGateway",
            "aws_vpc_internet_gateway",
            "internetGatewayId",
            "internet_gateway_id",
        ),
        (
            "AWS::EC2::NatGateway",
            "aws_vpc_nat_gateway",
            "natGatewayId",
            "nat_gateway_id",
        ),
        (
            "AWS::EC2::VPCEndpoint",
            "aws_vpc_endpoint",
            "vpcEndpointId",
            "vpc_endpoint_id",
        ),
        (
            "AWS::EC2::NetworkAcl",
            "aws_vpc_network_acl",
            "networkAclId",
            "network_acl_id",
        ),
        (
            "AWS::EC2::VPCPeeringConnection",
            "aws_vpc_peering_connection",
            "vpcPeeringConnectionId",
            "vpc_peering_connection_id",
        ),
    ];
    for (resource_type, kind, source, alias) in cases {
        let configuration = json!({(source): "recorded-id"});
        let resource = parse_config_row(&config_fixture(resource_type, &configuration), 100)?;
        assert_eq!(resource.kind, kind);
        assert_eq!(resource.source, Source::AwsConfig);
        assert_eq!(resource.properties.get(alias), Some(&json!("recorded-id")));
        assert_eq!(
            resource.properties.get("configuration"),
            Some(&configuration)
        );
        assert_eq!(
            resource.properties.get("resource_type"),
            Some(&json!(resource_type))
        );
        assert!(resource.properties.get("edges").is_none());
    }
    Ok(())
}

#[test]
fn config_maps_only_real_elbv2_kinds_and_preserves_classic()
-> Result<(), Box<dyn std::error::Error>> {
    for (resource_type, kind) in [
        (
            "AWS::ElasticLoadBalancingV2::LoadBalancer",
            "aws_elbv2_load_balancer",
        ),
        (
            "AWS::ElasticLoadBalancingV2::Listener",
            "aws_elbv2_listener",
        ),
        (
            "AWS::ElasticLoadBalancingV2::TargetGroup",
            "aws_elbv2_target_group",
        ),
        (
            "AWS::ElasticLoadBalancingV2::ListenerRule",
            "aws_elbv2_rule",
        ),
        (
            "AWS::ElasticLoadBalancing::LoadBalancer",
            "aws_elasticloadbalancing_loadbalancer",
        ),
        (
            "AWS::ElasticLoadBalancingV2::TargetHealth",
            "aws_elasticloadbalancingv2_targethealth",
        ),
    ] {
        let resource = parse_config_row(&config_fixture(resource_type, &json!({})), 100)?;
        assert_eq!(resource.kind, kind);
        assert_eq!(resource.source, Source::AwsConfig);
        for absent in [
            "type",
            "load_balancer_arn",
            "vpc_id",
            "subnet_ids",
            "default_actions",
            "target_health_state",
        ] {
            assert!(
                resource.properties.get(absent).is_none(),
                "{resource_type}: {absent}"
            );
        }
    }
    Ok(())
}

#[test]
fn config_keeps_nested_network_evidence_raw_and_projects_present_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let configuration = json!({
        "routeTableId": "rtb-example", "vpcId": "vpc-example",
        "routes": [{"destinationCidrBlock": "0.0.0.0/0", "gatewayId": "igw-example"}],
        "associations": [{"main": true, "routeTableAssociationId": "rtbassoc-example"}],
    });
    let route = parse_config_row(&config_fixture("AWS::EC2::RouteTable", &configuration), 100)?;
    assert_eq!(route.properties.get("vpc_id"), Some(&json!("vpc-example")));
    assert_eq!(route.properties.get("configuration"), Some(&configuration));
    for absent in ["routes", "associations", "is_main", "propagating_vgw_ids"] {
        assert!(route.properties.get(absent).is_none());
    }

    let endpoint_configuration = json!({
        "vpcEndpointId": "vpce-example", "routeTableIds": ["rtb-example"],
        "privateDnsEnabled": false, "groups": [{"groupId": "sg-example"}],
        "networkInterfaceIds": [{"unexpected": "eni-example"}],
    });
    let endpoint = parse_config_row(
        &config_fixture("AWS::EC2::VPCEndpoint", &endpoint_configuration),
        100,
    )?;
    assert_eq!(
        endpoint.properties.get("route_table_ids"),
        Some(&json!(["rtb-example"]))
    );
    assert_eq!(
        endpoint.properties.get("private_dns_enabled"),
        Some(&json!(false))
    );
    assert_eq!(
        endpoint.properties.get("configuration"),
        Some(&endpoint_configuration)
    );
    for absent in [
        "security_group_ids",
        "network_interface_ids",
        "subnet_ids",
        "dns_entries",
    ] {
        assert!(endpoint.properties.get(absent).is_none());
    }

    let peering_configuration = json!({
        "vpcPeeringConnectionId": "pcx-example", "status": {"code": "active"},
        "requesterVpcInfo": {"vpcId": "vpc-requester", "ownerId": "333333333333"},
        "accepterVpcInfo": {"vpcId": "vpc-accepter", "region": "us-east-1"},
    });
    let peering = parse_config_row(
        &config_fixture("AWS::EC2::VPCPeeringConnection", &peering_configuration),
        100,
    )?;
    assert_eq!(
        peering.properties.get("status_code"),
        Some(&json!("active"))
    );
    assert_eq!(
        peering.properties.get("requester_owner_id"),
        Some(&json!("333333333333"))
    );
    assert_eq!(
        peering.properties.get("accepter_region"),
        Some(&json!("us-east-1"))
    );
    assert!(peering.properties.get("requester_region").is_none());
    assert!(peering.properties.get("status_message").is_none());
    Ok(())
}
