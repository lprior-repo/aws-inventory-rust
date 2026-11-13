//! Optional, existing indexes are supplementary evidence, not completeness proofs.

use std::collections::BTreeMap;

use aws_smithy_types::Document;
use inventory_core::{MAX_PROPERTY_BYTES, Resource, Source};
use serde_json::{Value, json};

use crate::{
    AwsScanError,
    common::{Context, PageBody, now_millis, paginate, require_budget, required, sdk_error},
};

const CONFIG_QUERY: &str = "SELECT accountId, awsRegion, arn, resourceId, resourceName, resourceType, configuration, tags, configurationItemCaptureTime";

#[tracing::instrument(skip_all)]
pub(crate) async fn config(ctx: &Context, aggregator: &str) -> Result<(), AwsScanError> {
    let client = aws_sdk_config::Client::new(&ctx.config);
    paginate(
        ctx,
        ctx.scope("aws_config_aggregator", Some(aggregator), Source::AwsConfig),
        |token, budget| {
            let client = &client;
            async move {
                let output = ctx
                    .execute("config", "read", || async {
                        client
                            .select_aggregate_resource_config()
                            .configuration_aggregator_name(aggregator)
                            .expression(CONFIG_QUERY)
                            .max_results(ctx.page_size.clamp(1, 100))
                            .set_next_token(token.clone())
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                require_budget(output.results().len(), budget)?;
                let observed = now_millis()?;
                let resources = output
                    .results()
                    .iter()
                    .map(|row| parse_config_row(row, observed))
                    .collect::<Result<_, _>>()?;
                Ok(PageBody {
                    resources,
                    listed_count: output.results().len(),
                    next_token: output.next_token().map(str::to_owned),
                    ..PageBody::default()
                })
            }
        },
    )
    .await
}

fn parse_config_row(text: &str, observed_at: i64) -> Result<Resource, AwsScanError> {
    if text.len() > MAX_PROPERTY_BYTES {
        return Err(AwsScanError::InvalidData("Config row exceeds 1 MiB".into()));
    }
    let row: Value =
        serde_json::from_str(text).map_err(|error| AwsScanError::InvalidData(error.to_string()))?;
    let account = string(&row, "accountId")?;
    let region = string(&row, "awsRegion")?;
    let resource_type = string(&row, "resourceType")?;
    let resource_id = string(&row, "resourceId")?;
    let kind = resource_kind(&resource_type);
    let id = row
        .get("arn")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("aws-config:{account}:{region}:{resource_type}:{resource_id}"));
    let configuration = match row.get("configuration") {
        Some(Value::String(value)) => serde_json::from_str(value)
            .map_err(|error| AwsScanError::InvalidData(error.to_string()))?,
        Some(value) => value.clone(),
        None => Value::Null,
    };
    let tags = json_tags(row.get("tags"));
    let name = row
        .get("resourceName")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .or_else(|| tags.get("Name").cloned());
    let mut properties = json!({"resource_type":resource_type,"resource_id":resource_id,
        "configuration":configuration,"configuration_capture_time":row.get("configurationItemCaptureTime"),
        "coverage":"recorded resources available to the existing aggregator"});
    if let Some(object) = properties.as_object_mut() {
        for (target, source) in config_aliases(&kind) {
            if let Some(value) = config_alias_value(&configuration, source) {
                object.insert((*target).to_owned(), value.clone());
            }
        }
    }
    let resource = Resource {
        id,
        account_id: account,
        region,
        kind,
        name,
        properties,
        tags,
        source: Source::AwsConfig,
        observed_at,
    };
    resource.validate()?;
    Ok(resource)
}

// Config keeps its original nested representation under `configuration`. Only
// scalar fields and arrays of strings have the same shape as these native
// aliases. In particular, camelCase routes/actions must not become partial
// snake_case native evidence, and missing fields must not become empty lists.
fn config_alias_value<'a>(configuration: &'a Value, source: &str) -> Option<&'a Value> {
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

fn config_aliases(kind: &str) -> &'static [(&'static str, &'static str)] {
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

fn string(value: &Value, key: &str) -> Result<String, AwsScanError> {
    required(value.get(key).and_then(Value::as_str), key)
}

fn json_tags(value: Option<&Value>) -> BTreeMap<String, String> {
    match value {
        Some(Value::Object(object)) => object
            .iter()
            .filter_map(|(key, value)| Some((key.clone(), value.as_str()?.to_owned())))
            .collect(),
        Some(Value::Array(array)) => array
            .iter()
            .filter_map(|entry| {
                let key = entry.get("key").or_else(|| entry.get("Key"))?.as_str()?;
                let value = entry
                    .get("value")
                    .or_else(|| entry.get("Value"))?
                    .as_str()?;
                Some((key.to_owned(), value.to_owned()))
            })
            .collect(),
        _ => BTreeMap::new(),
    }
}

#[tracing::instrument(skip_all)]
pub(crate) async fn explorer(ctx: &Context, view: &str) -> Result<(), AwsScanError> {
    let client = aws_sdk_resourceexplorer2::Client::new(&ctx.config);
    paginate(
        ctx,
        ctx.scope(
            "aws_resource_explorer_view",
            Some(view),
            Source::ResourceExplorer,
        ),
        |token, budget| {
            let client = &client;
            async move {
                // AWS explicitly suppresses NextToken when MaxResults is 1000.
                let output = ctx
                    .execute("resource-explorer-2", "read", || async {
                        client
                            .list_resources()
                            .view_arn(view)
                            .max_results(ctx.page_size.clamp(1, 100))
                            .set_next_token(token.clone())
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                require_budget(output.resources().len(), budget)?;
                let observed = now_millis()?;
                let resources = output
                    .resources()
                    .iter()
                    .map(|resource| map_explorer(resource, observed))
                    .collect::<Result<_, _>>()?;
                Ok(PageBody {
                    resources,
                    listed_count: output.resources().len(),
                    next_token: output.next_token().map(str::to_owned),
                    ..PageBody::default()
                })
            }
        },
    )
    .await
}

fn map_explorer(
    resource: &aws_sdk_resourceexplorer2::types::Resource,
    observed_at: i64,
) -> Result<Resource, AwsScanError> {
    let id = required(resource.arn(), "Resource Explorer ARN")?;
    let account = required(
        resource.owning_account_id(),
        "Resource Explorer owning account",
    )?;
    let region = required(resource.region(), "Resource Explorer resource region")?;
    let resource_type = required(resource.resource_type(), "Resource Explorer resource type")?;
    let mut tags = BTreeMap::new();
    for property in resource.properties() {
        if property.name() == Some("tags")
            && let Some(data) = property.data()
        {
            tags.extend(document_tags(data));
        }
    }
    let last_reported = resource
        .last_reported_at()
        .map(|date| date.to_millis())
        .transpose()
        .map_err(|error| AwsScanError::InvalidData(error.to_string()))?;
    let result = Resource {
        kind: explorer_resource_kind(&resource_type, &id),
        id,
        account_id: account,
        region,
        name: tags.get("Name").cloned(),
        tags,
        source: Source::ResourceExplorer,
        observed_at,
        properties: json!({"resource_type":resource_type,"service":resource.service(),"last_reported_at":last_reported,
            "property_names":resource.properties().iter().filter_map(|property|property.name()).collect::<Vec<_>>(),
            "coverage":"resources visible through the existing view; index and view limits apply"}),
    };
    result.validate()?;
    Ok(result)
}

fn document_tags(document: &Document) -> BTreeMap<String, String> {
    if let Some(array) = document.as_array() {
        array
            .iter()
            .filter_map(|entry| {
                let entry = entry.as_object()?;
                let key = entry.get("Key").or_else(|| entry.get("key"))?.as_string()?;
                let value = entry
                    .get("Value")
                    .or_else(|| entry.get("value"))?
                    .as_string()?;
                Some((key.to_owned(), value.to_owned()))
            })
            .collect()
    } else if let Some(object) = document.as_object() {
        object
            .iter()
            .filter_map(|(key, value)| Some((key.clone(), value.as_string()?.to_owned())))
            .collect()
    } else {
        BTreeMap::new()
    }
}

fn resource_kind(resource_type: &str) -> String {
    let normalized = normalize_resource_type(resource_type);
    let known = match normalized.as_str() {
        "ec2:instance" => Some("aws_ec2_instance"),
        "ec2:vpc" => Some("aws_vpc"),
        "ec2:subnet" => Some("aws_vpc_subnet"),
        "ec2:securitygroup" | "ec2:security-group" => Some("aws_vpc_security_group"),
        "ec2:networkinterface" | "ec2:network-interface" => Some("aws_ec2_network_interface"),
        "ec2:routetable" | "ec2:route-table" => Some("aws_vpc_route_table"),
        "ec2:internetgateway" | "ec2:internet-gateway" => Some("aws_vpc_internet_gateway"),
        "ec2:natgateway" | "ec2:nat-gateway" => Some("aws_vpc_nat_gateway"),
        "ec2:vpcendpoint" | "ec2:vpc-endpoint" => Some("aws_vpc_endpoint"),
        "ec2:networkacl" | "ec2:network-acl" => Some("aws_vpc_network_acl"),
        "ec2:vpcpeeringconnection" | "ec2:vpc-peering-connection" => {
            Some("aws_vpc_peering_connection")
        }
        "elasticloadbalancingv2:loadbalancer" => Some("aws_elbv2_load_balancer"),
        "elasticloadbalancingv2:listener" => Some("aws_elbv2_listener"),
        "elasticloadbalancingv2:targetgroup" => Some("aws_elbv2_target_group"),
        "elasticloadbalancingv2:listenerrule" => Some("aws_elbv2_rule"),
        "ec2:volume" => Some("aws_ebs_volume"),
        "dynamodb:table" => Some("aws_dynamodb_table"),
        "lambda:function" => Some("aws_lambda_function"),
        "ecs:cluster" => Some("aws_ecs_cluster"),
        "ecs:service" => Some("aws_ecs_service"),
        "ecs:task" => Some("aws_ecs_task"),
        "eks:cluster" => Some("aws_eks_cluster"),
        "eks:nodegroup" => Some("aws_eks_node_group"),
        "s3:bucket" => Some("aws_s3_bucket"),
        "iam:role" => Some("aws_iam_role"),
        "iam:instanceprofile" | "iam:instance-profile" => Some("aws_iam_instance_profile"),
        "iam:policy" | "iam:managedpolicy" => Some("aws_iam_policy"),
        "route53:hostedzone" | "route53:hosted-zone" => Some("aws_route53_zone"),
        "route53:recordset" | "route53:record-set" => Some("aws_route53_record"),
        "cloudfront:distribution" => Some("aws_cloudfront_distribution"),
        _ => None,
    };
    known
        .map(str::to_owned)
        .unwrap_or_else(|| fallback_kind(&normalized))
}

fn normalize_resource_type(resource_type: &str) -> String {
    resource_type
        .to_ascii_lowercase()
        .replace("aws::", "")
        .replace("::", ":")
}

fn fallback_kind(normalized: &str) -> String {
    format!(
        "aws_{}",
        normalized
            .chars()
            .take(256)
            .map(|character| if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            })
            .collect::<String>()
    )
}

fn explorer_resource_kind(resource_type: &str, arn: &str) -> String {
    let normalized = normalize_resource_type(resource_type);
    let elb_type = normalized
        .strip_prefix("elasticloadbalancing:")
        .or_else(|| normalized.strip_prefix("elasticloadbalancingv2:"));
    match elb_type {
        // Resource Explorer documents that Classic and V2 load balancers can
        // share this namespace/type. Never promote it without a V2 ARN shape.
        Some(resource_type) => explorer_elbv2_kind(resource_type, arn)
            .map(str::to_owned)
            .unwrap_or_else(|| fallback_kind(&normalized)),
        None => resource_kind(resource_type),
    }
}

fn explorer_elbv2_kind(resource_type: &str, arn: &str) -> Option<&'static str> {
    let mut fields = arn.splitn(6, ':');
    if fields.next()? != "arn"
        || fields.next()?.is_empty()
        || fields.next()? != "elasticloadbalancing"
        || fields.next()?.is_empty()
        || fields.next()?.is_empty()
    {
        return None;
    }
    let mut path = fields.next()?.split('/');
    let family = path.next()?;
    let variant = path.next()?;
    let (kind, remaining) = match (family, variant) {
        ("loadbalancer", "app" | "net" | "gwy") => ("aws_elbv2_load_balancer", 2),
        ("listener", "app" | "net" | "gwy") => ("aws_elbv2_listener", 3),
        ("listener-rule", "app" | "net") => ("aws_elbv2_rule", 4),
        ("targetgroup", name) if !name.is_empty() => ("aws_elbv2_target_group", 1),
        _ => return None,
    };
    // The maximum tail is four components. Exact arity rejects Classic's
    // loadbalancer/name and truncated, extra-component or empty-component ARNs.
    if !(0..remaining).all(|_| path.next().is_some_and(|part| !part.is_empty()))
        || path.next().is_some()
    {
        return None;
    }
    let type_matches = if let Some((type_family, type_variant)) = resource_type.split_once('/') {
        type_family == family && type_variant == variant && family != "targetgroup"
    } else {
        resource_type == family || (resource_type == "listenerrule" && family == "listener-rule")
    };
    type_matches.then_some(kind)
}

#[cfg(test)]
mod property_tests;

#[cfg(test)]
mod tests {
    use super::{map_explorer, parse_config_row, resource_kind};
    use aws_smithy_types::{DateTime, Document};
    use inventory_core::Source;
    use serde_json::{Value, json};

    fn config_fixture(resource_type: &str, configuration: &Value) -> String {
        json!({
            "accountId": "222222222222", "awsRegion": "eu-west-1",
            "resourceType": resource_type, "resourceId": "recorded-id",
            "configuration": configuration, "configurationItemCaptureTime": "2026-11-12T00:00:00Z",
        })
        .to_string()
    }

    fn explorer_fixture(
        resource_type: &str,
        arn: &str,
    ) -> aws_sdk_resourceexplorer2::types::Resource {
        aws_sdk_resourceexplorer2::types::Resource::builder()
            .arn(arn)
            .owning_account_id("222222222222")
            .region("eu-west-1")
            .resource_type(resource_type)
            .build()
    }

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
            let arn =
                format!("arn:aws-us-gov:elasticloadbalancing:us-gov-west-1:222222222222:{path}");
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
    fn config_uses_resource_owner_not_aggregator_account() -> Result<(), Box<dyn std::error::Error>>
    {
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

    #[test]
    fn explorer_keeps_owner_region_and_source_freshness() -> Result<(), Box<dyn std::error::Error>>
    {
        let tag = Document::Object(
            [
                ("Key".to_owned(), Document::String("Name".to_owned())),
                ("Value".to_owned(), Document::String("worker".to_owned())),
            ]
            .into_iter()
            .collect(),
        );
        let source = aws_sdk_resourceexplorer2::types::Resource::builder()
            .arn("arn:aws:ec2:eu-west-1:222222222222:instance/i-abc")
            .owning_account_id("222222222222")
            .region("eu-west-1")
            .resource_type("ec2:instance")
            .service("ec2")
            .last_reported_at(DateTime::from_secs(10))
            .properties(
                aws_sdk_resourceexplorer2::types::ResourceProperty::builder()
                    .name("tags")
                    .data(Document::Array(vec![tag]))
                    .build(),
            )
            .build();
        let resource = map_explorer(&source, 20_000)?;
        assert_eq!(resource.account_id, "222222222222");
        assert_eq!(resource.region, "eu-west-1");
        assert_eq!(resource.name.as_deref(), Some("worker"));
        assert_eq!(resource.observed_at, 20_000);
        assert_eq!(
            resource
                .properties
                .get("last_reported_at")
                .and_then(serde_json::Value::as_i64),
            Some(10_000)
        );
        assert_eq!(resource.source, Source::ResourceExplorer);
        Ok(())
    }
}
