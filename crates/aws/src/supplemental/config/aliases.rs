//! aliases operations.
use super::*;

pub(in super::super) fn config_alias_value<'a>(
    configuration: &'a Value,
    source: &str,
) -> Option<&'a Value> {
    let value = if source.starts_with('/') {
        configuration.pointer(source)
    } else {
        configuration.get(source)
    }?;
    match value {
        Value::Object(_) => None,
        Value::Array(values) if !values.iter().all(Value::is_string) => None,
        _ => Some(value),
    }
}

pub(in super::super) fn config_aliases(kind: &str) -> &'static [(&'static str, &'static str)] {
    match kind {
        "aws_ec2_instance" => &[
            ("instance_id", "instanceId"),
            ("vpc_id", "vpcId"),
            ("subnet_id", "subnetId"),
            ("instance_type", "instanceType"),
            ("private_ip_address", "privateIpAddress"),
            ("public_ip_address", "publicIpAddress"),
        ],
        "aws_vpc" => &[
            ("vpc_id", "vpcId"),
            ("cidr_block", "cidrBlock"),
            ("is_default", "isDefault"),
            ("state", "state"),
        ],
        "aws_vpc_subnet" => &[
            ("subnet_id", "subnetId"),
            ("vpc_id", "vpcId"),
            ("cidr_block", "cidrBlock"),
            ("availability_zone", "availabilityZone"),
        ],
        "aws_vpc_security_group" => &[
            ("group_id", "groupId"),
            ("group_name", "groupName"),
            ("vpc_id", "vpcId"),
            ("description", "description"),
        ],
        "aws_ec2_network_interface" => &[
            ("network_interface_id", "networkInterfaceId"),
            ("subnet_id", "subnetId"),
            ("vpc_id", "vpcId"),
            ("status", "status"),
        ],
        "aws_vpc_route_table" => &[
            ("route_table_id", "routeTableId"),
            ("vpc_id", "vpcId"),
            ("owner_id", "ownerId"),
        ],
        "aws_vpc_internet_gateway" => &[
            ("internet_gateway_id", "internetGatewayId"),
            ("owner_id", "ownerId"),
        ],
        "aws_vpc_nat_gateway" => &[
            ("nat_gateway_id", "natGatewayId"),
            ("vpc_id", "vpcId"),
            ("subnet_id", "subnetId"),
            ("state", "state"),
            ("connectivity_type", "connectivityType"),
            ("availability_mode", "availabilityMode"),
            ("route_table_id", "routeTableId"),
        ],
        "aws_vpc_endpoint" => &[
            ("vpc_endpoint_id", "vpcEndpointId"),
            ("vpc_endpoint_type", "vpcEndpointType"),
            ("vpc_id", "vpcId"),
            ("owner_id", "ownerId"),
            ("service_name", "serviceName"),
            ("state", "state"),
            ("route_table_ids", "routeTableIds"),
            ("subnet_ids", "subnetIds"),
            ("network_interface_ids", "networkInterfaceIds"),
            ("private_dns_enabled", "privateDnsEnabled"),
        ],
        "aws_vpc_network_acl" => &[
            ("network_acl_id", "networkAclId"),
            ("vpc_id", "vpcId"),
            ("owner_id", "ownerId"),
            ("is_default", "isDefault"),
        ],
        "aws_vpc_peering_connection" => &[
            ("vpc_peering_connection_id", "vpcPeeringConnectionId"),
            ("status_code", "/status/code"),
            ("status_message", "/status/message"),
            ("requester_vpc_id", "/requesterVpcInfo/vpcId"),
            ("requester_owner_id", "/requesterVpcInfo/ownerId"),
            ("requester_region", "/requesterVpcInfo/region"),
            ("accepter_vpc_id", "/accepterVpcInfo/vpcId"),
            ("accepter_owner_id", "/accepterVpcInfo/ownerId"),
            ("accepter_region", "/accepterVpcInfo/region"),
        ],
        "aws_elbv2_load_balancer" => &[
            ("load_balancer_arn", "loadBalancerArn"),
            ("load_balancer_name", "loadBalancerName"),
            ("type", "type"),
            ("scheme", "scheme"),
            ("dns_name", "dNSName"),
            ("state", "/state/code"),
            ("state_reason", "/state/reason"),
            ("ip_address_type", "ipAddressType"),
            ("vpc_id", "vpcId"),
            ("security_group_ids", "securityGroups"),
        ],
        "aws_elbv2_listener" => &[
            ("listener_arn", "listenerArn"),
            ("load_balancer_arn", "loadBalancerArn"),
            ("port", "port"),
            ("protocol", "protocol"),
            ("ssl_policy", "sslPolicy"),
            ("alpn_policy", "alpnPolicy"),
        ],
        "aws_elbv2_target_group" => &[
            ("target_group_arn", "targetGroupArn"),
            ("target_group_name", "targetGroupName"),
            ("target_type", "targetType"),
            ("protocol", "protocol"),
            ("protocol_version", "protocolVersion"),
            ("port", "port"),
            ("vpc_id", "vpcId"),
            ("load_balancer_arns", "loadBalancerArns"),
            ("health_check_enabled", "healthCheckEnabled"),
            ("health_check_protocol", "healthCheckProtocol"),
            ("health_check_port", "healthCheckPort"),
            ("health_check_path", "healthCheckPath"),
            ("ip_address_type", "ipAddressType"),
        ],
        "aws_elbv2_rule" => &[
            ("rule_arn", "ruleArn"),
            ("listener_arn", "listenerArn"),
            ("priority", "priority"),
            ("is_default", "isDefault"),
        ],
        "aws_ebs_volume" => &[
            ("volume_id", "volumeId"),
            ("size", "size"),
            ("state", "state"),
            ("volume_type", "volumeType"),
            ("encrypted", "encrypted"),
        ],
        "aws_dynamodb_table" => &[
            ("table_name", "tableName"),
            ("table_status", "tableStatus"),
            ("item_count", "itemCount"),
            ("table_size_bytes", "tableSizeBytes"),
        ],
        "aws_lambda_function" => &[
            ("function_name", "functionName"),
            ("runtime", "runtime"),
            ("memory_size", "memorySize"),
            ("timeout", "timeout"),
            ("role", "role"),
        ],
        "aws_ecs_cluster" => &[
            ("cluster_name", "clusterName"),
            ("cluster_arn", "clusterArn"),
            ("status", "status"),
        ],
        "aws_ecs_service" => &[
            ("service_name", "serviceName"),
            ("service_arn", "serviceArn"),
            ("cluster_arn", "clusterArn"),
            ("desired_count", "desiredCount"),
            ("running_count", "runningCount"),
        ],
        "aws_eks_cluster" => &[
            ("name", "name"),
            ("arn", "arn"),
            ("status", "status"),
            ("version", "version"),
            ("endpoint", "endpoint"),
        ],
        "aws_s3_bucket" => &[("bucket_name", "name")],
        "aws_iam_role" => &[
            ("role_name", "roleName"),
            ("role_id", "roleId"),
            ("path", "path"),
            ("max_session_duration", "maxSessionDuration"),
        ],
        "aws_iam_instance_profile" => &[
            ("instance_profile_name", "instanceProfileName"),
            ("instance_profile_id", "instanceProfileId"),
            ("path", "path"),
        ],
        "aws_iam_policy" => &[
            ("policy_name", "policyName"),
            ("policy_id", "policyId"),
            ("path", "path"),
            ("default_version_id", "defaultVersionId"),
            ("attachment_count", "attachmentCount"),
        ],
        "aws_route53_zone" => &[
            ("zone_id", "id"),
            ("private_zone", "/config/privateZone"),
            ("resource_record_set_count", "resourceRecordSetCount"),
        ],
        "aws_cloudfront_distribution" => &[
            ("distribution_id", "id"),
            ("domain_name", "domainName"),
            ("status", "status"),
            ("enabled", "/distributionConfig/enabled"),
        ],
        _ => &[],
    }
}
