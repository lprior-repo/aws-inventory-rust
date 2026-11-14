use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed};

use super::*;

fn configuration() -> Config {
    Config {
        cases: 128,
        max_shrink_iters: 4_096,
        rng_seed: RngSeed::Fixed(0x5355_5050_4c45_4d54),
        ..Config::default()
    }
}

fn varied_case(value: &str, mask: u64) -> String {
    value
        .bytes()
        .enumerate()
        .map(|(index, byte)| {
            char::from(if mask & (1_u64 << (index % 64)) == 0 {
                byte.to_ascii_lowercase()
            } else {
                byte.to_ascii_uppercase()
            })
        })
        .collect()
}

proptest! {
    #![proptest_config(configuration())]

    #[test]
    fn all_known_native_type_names_keep_their_kind_under_ascii_case_changes(mask in any::<u64>()) {
        for (source, expected) in [
            ("AWS::EC2::Instance", "aws_ec2_instance"),
            ("AWS::EC2::VPC", "aws_vpc"),
            ("AWS::EC2::Subnet", "aws_vpc_subnet"),
            ("AWS::EC2::SecurityGroup", "aws_vpc_security_group"),
            ("AWS::EC2::NetworkInterface", "aws_ec2_network_interface"),
            ("AWS::EC2::RouteTable", "aws_vpc_route_table"),
            ("AWS::EC2::InternetGateway", "aws_vpc_internet_gateway"),
            ("AWS::EC2::NatGateway", "aws_vpc_nat_gateway"),
            ("AWS::EC2::VPCEndpoint", "aws_vpc_endpoint"),
            ("AWS::EC2::NetworkAcl", "aws_vpc_network_acl"),
            ("AWS::EC2::VPCPeeringConnection", "aws_vpc_peering_connection"),
            ("AWS::EC2::Volume", "aws_ebs_volume"),
            ("AWS::ElasticLoadBalancingV2::LoadBalancer", "aws_elbv2_load_balancer"),
            ("AWS::ElasticLoadBalancingV2::Listener", "aws_elbv2_listener"),
            ("AWS::ElasticLoadBalancingV2::ListenerRule", "aws_elbv2_rule"),
            ("AWS::ElasticLoadBalancingV2::TargetGroup", "aws_elbv2_target_group"),
            ("AWS::DynamoDB::Table", "aws_dynamodb_table"),
            ("AWS::Lambda::Function", "aws_lambda_function"),
            ("AWS::ECS::Cluster", "aws_ecs_cluster"),
            ("AWS::ECS::Service", "aws_ecs_service"),
            ("AWS::ECS::Task", "aws_ecs_task"),
            ("AWS::EKS::Cluster", "aws_eks_cluster"),
            ("AWS::EKS::Nodegroup", "aws_eks_node_group"),
            ("AWS::S3::Bucket", "aws_s3_bucket"),
            ("AWS::IAM::Role", "aws_iam_role"),
            ("AWS::IAM::InstanceProfile", "aws_iam_instance_profile"),
            ("AWS::IAM::Policy", "aws_iam_policy"),
            ("AWS::Route53::HostedZone", "aws_route53_zone"),
            ("AWS::Route53::RecordSet", "aws_route53_record"),
            ("AWS::CloudFront::Distribution", "aws_cloudfront_distribution"),
        ] {
            prop_assert_eq!(resource_kind(&varied_case(source, mask)), expected);
        }
    }

    #[test]
    fn well_formed_config_types_and_explorer_types_normalize_to_the_same_identity(
        service in prop::sample::select(vec!["EC2", "Lambda", "S3", "CodeBuild", "ServiceCatalog", "CloudWatch"]),
        kind in "[A-Za-z][A-Za-z0-9]{0,24}",
        mask in any::<u64>(),
    ) {
        let expected = format!("{}:{}", service.to_ascii_lowercase(), kind.to_ascii_lowercase());
        let config = varied_case(&format!("AWS::{service}::{kind}"), mask);
        let explorer = varied_case(&format!("{service}:{kind}"), mask);
        prop_assert_eq!(normalize_resource_type(&config), expected.clone());
        prop_assert_eq!(normalize_resource_type(&explorer), expected.clone());
        prop_assert_eq!(normalize_resource_type(&expected), expected);
        prop_assert_eq!(resource_kind(&config), resource_kind(&explorer));
    }

    #[test]
    fn fallback_identifiers_are_bounded_ascii_and_do_not_split_unicode(input in prop::collection::vec(any::<char>(), 0..520)) {
        let value = input.iter().collect::<String>();
        let kind = fallback_kind(&value);
        prop_assert!(kind.starts_with("aws_"));
        prop_assert_eq!(kind.len(), 4_usize.saturating_add(input.len().min(256)));
        prop_assert!(kind.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'));
        for (original, output) in input.iter().take(256).zip(kind.chars().skip(4)) {
            if original.is_ascii_alphanumeric() {
                prop_assert_eq!(output, *original);
            } else {
                prop_assert_eq!(output, '_');
            }
        }
    }

    #[test]
    fn complete_v2_arn_shapes_are_recognized_in_both_resource_explorer_namespaces(
        name in "[A-Za-z][A-Za-z0-9-]{0,20}",
        id in "[a-f0-9]{8,16}",
        variant in prop::sample::select(vec!["app", "net", "gwy"]),
        namespace in prop::sample::select(vec!["elasticloadbalancing", "elasticloadbalancingv2"]),
    ) {
        for (family, tail, expected) in [
            ("loadbalancer", format!("{variant}/{name}/{id}"), "aws_elbv2_load_balancer"),
            ("listener", format!("{variant}/{name}/{id}/{id}"), "aws_elbv2_listener"),
            ("targetgroup", format!("{name}/{id}"), "aws_elbv2_target_group"),
        ] {
            let arn = format!("arn:aws:elasticloadbalancing:us-east-1:000000000000:{family}/{tail}");
            prop_assert_eq!(explorer_resource_kind(&format!("{namespace}:{family}"), &arn), expected);
            if family != "targetgroup" {
                prop_assert_eq!(explorer_resource_kind(&format!("{namespace}:{family}/{variant}"), &arn), expected);
            }
        }
        for rule_variant in ["app", "net"] {
            let arn = format!("arn:aws:elasticloadbalancing:us-east-1:000000000000:listener-rule/{rule_variant}/{name}/{id}/{id}/{id}");
            prop_assert_eq!(explorer_resource_kind(&format!("{namespace}:listener-rule"), &arn), "aws_elbv2_rule");
            prop_assert_eq!(explorer_resource_kind(&format!("{namespace}:listenerrule"), &arn), "aws_elbv2_rule");
        }
    }

    #[test]
    fn v2_arn_inference_requires_every_component_and_exact_arity(
        name in "[A-Za-z][A-Za-z0-9-]{0,20}",
        id in "[a-f0-9]{8,16}",
    ) {
        for (family, mut parts) in [
            ("loadbalancer", vec!["app".to_owned(), name.clone(), id.clone()]),
            ("listener", vec!["net".to_owned(), name.clone(), id.clone(), id.clone()]),
            ("listener-rule", vec!["app".to_owned(), name.clone(), id.clone(), id.clone(), id.clone()]),
            ("targetgroup", vec![name.clone(), id.clone()]),
        ] {
            let prefix = format!("arn:aws:elasticloadbalancing:us-east-1:000000000000:{family}/");
            let valid = format!("{prefix}{}", parts.join("/"));
            prop_assert!(explorer_elbv2_kind(family, &valid).is_some());
            prop_assert!(explorer_elbv2_kind(family, &format!("{valid}/extra")).is_none(), "extra ARN path component accepted");
            for index in 0..parts.len() {
                let previous = parts
                    .get_mut(index)
                    .map(std::mem::take)
                    .ok_or_else(|| TestCaseError::fail("missing ARN component"))?;
                prop_assert!(explorer_elbv2_kind(family, &format!("{prefix}{}", parts.join("/"))).is_none(), "empty ARN path component accepted");
                let slot = parts
                    .get_mut(index)
                    .ok_or_else(|| TestCaseError::fail("missing ARN component"))?;
                *slot = previous;
            }
            parts.pop();
            prop_assert!(explorer_elbv2_kind(family, &format!("{prefix}{}", parts.join("/"))).is_none(), "truncated ARN path accepted");
        }
    }

    #[test]
    fn classic_load_balancer_arns_never_acquire_v2_kinds(
        name in "[A-Za-z][A-Za-z0-9-]{0,30}",
        namespace in prop::sample::select(vec!["elasticloadbalancing", "elasticloadbalancingv2"]),
        resource_type in prop::sample::select(vec!["loadbalancer", "loadbalancer/app", "loadbalancer/net", "listener", "targetgroup", "listenerrule"]),
    ) {
        let arn = format!("arn:aws:elasticloadbalancing:us-east-1:000000000000:loadbalancer/{name}");
        let kind = explorer_resource_kind(&format!("{namespace}:{resource_type}"), &arn);
        prop_assert!(!kind.starts_with("aws_elbv2_"));
        prop_assert_eq!(kind, fallback_kind(&format!("{namespace}:{resource_type}")));
    }

    #[test]
    fn explorer_type_cannot_contradict_its_arn_family_or_variant(name in "[A-Za-z][A-Za-z0-9-]{0,20}") {
        let arn = format!("arn:aws:elasticloadbalancing:us-east-1:000000000000:loadbalancer/app/{name}/12345678");
        for wrong in ["listener", "targetgroup", "listener-rule", "listenerrule", "loadbalancer/net", "loadbalancer/gwy", "loadbalancer/app/extra"] {
            prop_assert!(explorer_elbv2_kind(wrong, &arn).is_none());
        }
        for prefix in ["arn::elasticloadbalancing:us-east-1:000000000000", "arn:aws:ec2:us-east-1:000000000000", "arn:aws:elasticloadbalancing::000000000000", "arn:aws:elasticloadbalancing:us-east-1:", "url:aws:elasticloadbalancing:us-east-1:000000000000"] {
            prop_assert!(explorer_elbv2_kind("loadbalancer", &format!("{prefix}:loadbalancer/app/{name}/12345678")).is_none(), "invalid ELB ARN prefix accepted");
        }
    }
}
