use super::*;

pub(super) fn arn_parts(value: &str) -> Option<(&str, &str)> {
    let mut parts = value.splitn(6, ':');
    if parts.next()? != "arn" || parts.next()?.is_empty() || parts.next()?.is_empty() {
        return None;
    }
    let region = parts.next()?;
    let account = parts.next()?;
    if parts.next()?.is_empty()
        || value.len() > inventory_core::MAX_IDENTIFIER_BYTES
        || value.contains('\0')
    {
        return None;
    }
    Some((region, account))
}

pub(super) fn expand_arn(ctx: &Context, template: &str, id: &str) -> Result<String, AwsScanError> {
    if id.is_empty() || id.len() > MAX_IDENTIFIER_BYTES || id.contains(['\0', '|']) {
        return Err(invalid(
            "resource identity template requires one nonempty identifier",
        ));
    }
    let arn = template
        .replace("{partition}", &ctx.partition)
        .replace("{region}", &ctx.region)
        .replace("{account}", &ctx.account)
        .replace("{identifier}", id);
    if arn_parts(&arn).is_none() || arn.contains(['{', '}']) {
        return Err(invalid("invalid identity ARN template"));
    }
    Ok(arn)
}

pub(super) fn resource_identity(
    ctx: &Context,
    descriptor: &ResourceType,
    identifier: &str,
    properties: &Value,
) -> Result<(String, &'static str), AwsScanError> {
    for pointer in &descriptor.arn_pointers {
        if let Some(arn) = properties.pointer(pointer).and_then(Value::as_str) {
            if arn_parts(arn).is_none() {
                return Err(invalid("provider self-ARN property is malformed"));
            }
            return Ok((arn.to_owned(), "returned_resource_arn"));
        }
    }
    if descriptor.identifier_is_arn && arn_parts(identifier).is_some() {
        return Ok((identifier.to_owned(), "listed_identifier_arn"));
    }
    if let Some(template) = &descriptor.arn_template {
        let property_id = descriptor
            .arn_identifier_pointer
            .as_deref()
            .and_then(|pointer| properties.pointer(pointer))
            .and_then(Value::as_str);
        let simple_identifier = (descriptor.primary_identifier.len() == 1
            && !identifier.contains('|'))
        .then_some(identifier);
        if let Some(id) = property_id.or(simple_identifier) {
            return Ok((
                expand_arn(ctx, template, id)?,
                if template.contains("{account}") {
                    "canonical_template_assumed_scanning_account"
                } else {
                    "canonical_template_without_account_component"
                },
            ));
        }
    }
    Ok((
        format!(
            "awscc:{}:{}:{}:{}:{:x}",
            ctx.partition,
            ctx.region,
            ctx.account,
            descriptor.type_name,
            base16ct::HexDisplay(&Sha256::digest(identifier.as_bytes()))
        ),
        "namespaced_provider_identifier",
    ))
}

pub(super) fn tags(properties: &Value) -> Result<BTreeMap<String, String>, AwsScanError> {
    let mut result = BTreeMap::new();
    match properties.get("Tags") {
        Some(Value::Object(values)) => {
            if values.len() > 100 {
                return Err(invalid("provider tag count exceeds 100"));
            }
            for (key, value) in values {
                if let Some(value) = value.as_str() {
                    result.insert(key.clone(), value.to_owned());
                }
            }
        }
        Some(Value::Array(values)) => {
            if values.len() > 100 {
                return Err(invalid("provider tag count exceeds 100"));
            }
            for value in values {
                if let (Some(key), Some(value)) = (
                    value.get("Key").and_then(Value::as_str),
                    value.get("Value").and_then(Value::as_str),
                ) {
                    result.insert(key.to_owned(), value.to_owned());
                }
            }
        }
        _ => {}
    }
    Ok(result)
}

pub(super) fn map_description(
    ctx: &Context,
    descriptor: &ResourceType,
    identifier: &str,
    description: &ResourceDescription,
    complete: bool,
) -> Result<PageBody, AwsScanError> {
    let (provider, redacted) = sanitized_properties(descriptor, description.properties())?;
    let (id, identity_resolution) = resource_identity(ctx, descriptor, identifier, &provider)?;
    let (region, account) = arn_parts(&id).unwrap_or((&ctx.region, &ctx.account));
    let name = descriptor
        .name_pointers
        .iter()
        .find_map(|pointer| provider.pointer(pointer).and_then(Value::as_str))
        .map(str::to_owned);
    let mut resource = Resource {
        account_id: if account.is_empty() {
            ctx.account.clone()
        } else {
            account.to_owned()
        },
        region: if region.is_empty() {
            ctx.region.clone()
        } else {
            region.to_owned()
        },
        id,
        kind: descriptor.kind.clone(),
        name,
        tags: tags(&provider)?,
        source: Source::CloudControl,
        observed_at: now_millis()?,
        properties: json!({"cloudformation_type":descriptor.type_name,"resource_identifier":identifier,
            "provider_properties":provider,"identity_resolution":identity_resolution,"request_region":ctx.region,
            "schema_sha256":descriptor.schema_sha256,"redacted_paths":redacted,"details_available":complete,
            "observation_detail":if complete {"get_resource"} else {"list_only"}}),
    };
    add_native_aliases(descriptor, &mut resource)?;
    resource.validate()?;
    let edges = if complete {
        map_relationships(ctx, descriptor, &resource)?
    } else {
        Vec::new()
    };
    Ok(PageBody {
        resources: vec![resource],
        edges,
        listed_count: 1,
        next_token: None,
    })
}

pub(super) fn add_native_aliases(
    descriptor: &ResourceType,
    resource: &mut Resource,
) -> Result<(), AwsScanError> {
    let aliases: &[(&str, &str)] = match descriptor.type_name.as_str() {
        "AWS::EC2::Instance" => &[
            ("instance_id", "InstanceId"),
            ("vpc_id", "VpcId"),
            ("subnet_id", "SubnetId"),
            ("instance_type", "InstanceType"),
            ("image_id", "ImageId"),
            ("private_ip_address", "PrivateIp"),
            ("public_ip_address", "PublicIp"),
        ],
        "AWS::EC2::VPC" => &[("vpc_id", "VpcId"), ("cidr_block", "CidrBlock")],
        "AWS::EC2::Subnet" => &[
            ("subnet_id", "SubnetId"),
            ("vpc_id", "VpcId"),
            ("cidr_block", "CidrBlock"),
            ("availability_zone", "AvailabilityZone"),
        ],
        "AWS::EC2::SecurityGroup" => &[
            ("group_id", "GroupId"),
            ("group_name", "GroupName"),
            ("vpc_id", "VpcId"),
            ("description", "GroupDescription"),
        ],
        "AWS::EC2::RouteTable" => &[("route_table_id", "RouteTableId"), ("vpc_id", "VpcId")],
        "AWS::EC2::NetworkAcl" => &[("network_acl_id", "Id"), ("vpc_id", "VpcId")],
        "AWS::EC2::NatGateway" => &[
            ("nat_gateway_id", "NatGatewayId"),
            ("subnet_id", "SubnetId"),
        ],
        "AWS::EC2::VPCEndpoint" => &[
            ("vpc_endpoint_id", "Id"),
            ("vpc_id", "VpcId"),
            ("vpc_endpoint_type", "VpcEndpointType"),
        ],
        "AWS::DynamoDB::Table" => &[("table_name", "TableName")],
        "AWS::Lambda::Function" => &[
            ("function_name", "FunctionName"),
            ("runtime", "Runtime"),
            ("handler", "Handler"),
            ("role", "Role"),
            ("memory_size", "MemorySize"),
            ("timeout", "Timeout"),
        ],
        "AWS::ECS::Cluster" => &[("cluster_name", "ClusterName"), ("cluster_arn", "Arn")],
        "AWS::ECS::Service" => &[
            ("service_name", "Name"),
            ("service_arn", "ServiceArn"),
            ("task_definition", "TaskDefinition"),
            ("desired_count", "DesiredCount"),
        ],
        "AWS::ECS::TaskDefinition" => &[
            ("task_definition_arn", "TaskDefinitionArn"),
            ("family", "Family"),
            ("network_mode", "NetworkMode"),
            ("task_role_arn", "TaskRoleArn"),
            ("execution_role_arn", "ExecutionRoleArn"),
        ],
        "AWS::EKS::Cluster" => &[
            ("name", "Name"),
            ("arn", "Arn"),
            ("version", "Version"),
            ("endpoint", "Endpoint"),
        ],
        "AWS::ElasticLoadBalancingV2::LoadBalancer" => &[
            ("type", "Type"),
            ("load_balancer_arn", "LoadBalancerArn"),
            ("load_balancer_name", "LoadBalancerName"),
            ("scheme", "Scheme"),
            ("dns_name", "DNSName"),
        ],
        "AWS::ElasticLoadBalancingV2::TargetGroup" => &[
            ("target_group_arn", "TargetGroupArn"),
            ("target_group_name", "TargetGroupName"),
            ("vpc_id", "VpcId"),
            ("port", "Port"),
            ("protocol", "Protocol"),
            ("target_type", "TargetType"),
        ],
        _ => &[],
    };
    let properties = resource
        .properties
        .as_object_mut()
        .ok_or_else(|| invalid("resource properties must be an object"))?;
    for (alias, key) in aliases {
        if let Some(value) = properties
            .get("provider_properties")
            .and_then(|provider| provider.get(*key))
            .filter(|value| value.is_string() || value.is_number() || value.is_boolean())
            .cloned()
        {
            properties.insert((*alias).to_owned(), value);
        }
    }
    Ok(())
}
