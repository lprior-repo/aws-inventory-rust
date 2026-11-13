//! config projects recorded elbv2 scalars without native enrichment behavior.
use super::*;

#[test]
fn config_projects_recorded_elbv2_scalars_without_native_enrichment()
-> Result<(), Box<dyn std::error::Error>> {
    let configuration = json!({
        "loadBalancerArn": "arn:aws:elasticloadbalancing:eu-west-1:222222222222:loadbalancer/app/example/lb-id",
        "loadBalancerName": "example", "type": "application", "vpcId": "vpc-example",
        "dNSName": "example.elb.amazonaws.com", "state": {"code": "active"},
        "securityGroups": ["sg-example"], "availabilityZones": [{"subnetId": "subnet-example"}],
    });
    let resource = parse_config_row(
        &config_fixture("AWS::ElasticLoadBalancingV2::LoadBalancer", &configuration),
        100,
    )?;
    assert_eq!(
        resource.properties.get("dns_name"),
        Some(&json!("example.elb.amazonaws.com"))
    );
    assert_eq!(resource.properties.get("state"), Some(&json!("active")));
    assert_eq!(
        resource.properties.get("security_group_ids"),
        Some(&json!(["sg-example"]))
    );
    assert_eq!(
        resource.properties.get("configuration"),
        Some(&configuration)
    );
    for absent in [
        "subnet_ids",
        "availability_zones",
        "state_reason",
        "tags_collected",
        "edges",
    ] {
        assert!(resource.properties.get(absent).is_none());
    }

    let listener_configuration = json!({
        "listenerArn": "listener-example", "loadBalancerArn": "balancer-example",
        "port": 443, "protocol": "HTTPS", "defaultActions": [{"type": "forward", "targetGroupArn": "group-example"}],
    });
    let listener = parse_config_row(
        &config_fixture(
            "AWS::ElasticLoadBalancingV2::Listener",
            &listener_configuration,
        ),
        100,
    )?;
    assert_eq!(
        listener.properties.get("load_balancer_arn"),
        Some(&json!("balancer-example"))
    );
    assert_eq!(listener.properties.get("port"), Some(&json!(443)));
    assert_eq!(
        listener.properties.get("configuration"),
        Some(&listener_configuration)
    );
    for absent in [
        "default_actions",
        "default_target_group_arns",
        "certificate_list_complete",
    ] {
        assert!(listener.properties.get(absent).is_none());
    }
    Ok(())
}

#[test]
fn explorer_maps_network_kinds_without_synthesizing_details()
-> Result<(), Box<dyn std::error::Error>> {
    for (resource_type, path, kind) in [
        (
            "ec2:route-table",
            "route-table/rtb-example",
            "aws_vpc_route_table",
        ),
        (
            "ec2:internet-gateway",
            "internet-gateway/igw-example",
            "aws_vpc_internet_gateway",
        ),
        (
            "ec2:natgateway",
            "natgateway/nat-example",
            "aws_vpc_nat_gateway",
        ),
        (
            "ec2:vpc-endpoint",
            "vpc-endpoint/vpce-example",
            "aws_vpc_endpoint",
        ),
        (
            "ec2:network-acl",
            "network-acl/acl-example",
            "aws_vpc_network_acl",
        ),
        (
            "ec2:vpc-peering-connection",
            "vpc-peering-connection/pcx-example",
            "aws_vpc_peering_connection",
        ),
    ] {
        let arn = format!("arn:aws:ec2:eu-west-1:222222222222:{path}");
        let resource = map_explorer(&explorer_fixture(resource_type, &arn), 100)?;
        assert_eq!(resource.kind, kind);
        assert_eq!(resource.id, arn);
        assert_eq!(resource.source, Source::ResourceExplorer);
        assert_eq!(
            resource.properties.get("resource_type"),
            Some(&json!(resource_type))
        );
        for absent in [
            "routes",
            "attachments",
            "vpc_id",
            "state",
            "subnet_ids",
            "entries",
            "status_code",
            "edges",
        ] {
            assert!(resource.properties.get(absent).is_none());
        }
    }
    Ok(())
}
