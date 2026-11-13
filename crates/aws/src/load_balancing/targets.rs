//! targets operations.
use super::*;

pub(super) fn map_target_group(
    ctx: &Context,
    model: &TargetGroup,
) -> Result<PageBody, AwsScanError> {
    check_nested(model.load_balancer_arns(), "target group load balancers")?;
    let target_type = required(
        model.target_type().map(|value| value.as_str()),
        "target type",
    )?;
    let balancers: BTreeSet<_> = model
        .load_balancer_arns()
        .iter()
        .map(String::as_str)
        .collect();
    let resource = ctx.resource(
        "aws_elbv2_target_group", required(model.target_group_arn(), "target group ARN")?,
        model.target_group_name().map(str::to_owned),
        json!({
            "target_group_arn": model.target_group_arn(), "target_group_name": model.target_group_name(),
            "target_type": target_type, "protocol": model.protocol().map(|value| value.as_str()),
            "protocol_version": model.protocol_version(), "port": model.port(), "vpc_id": model.vpc_id(),
            "load_balancer_arns": balancers, "health_check_enabled": model.health_check_enabled(),
            "health_check_protocol": model.health_check_protocol().map(|value| value.as_str()),
            "health_check_port": model.health_check_port(), "health_check_path": model.health_check_path(),
            "health_check_interval_seconds": model.health_check_interval_seconds(),
            "health_check_timeout_seconds": model.health_check_timeout_seconds(),
            "healthy_threshold_count": model.healthy_threshold_count(), "unhealthy_threshold_count": model.unhealthy_threshold_count(),
            "matcher": model.matcher().map(|matcher| json!({"http_code": matcher.http_code(), "grpc_code": matcher.grpc_code()})),
            "ip_address_type": model.ip_address_type().map(|value| value.as_str()),
            "target_control_port": model.target_control_port(), "tags_collected": false,
        }), BTreeMap::new(),
    )?;
    let mut edges: Vec<_> = balancers
        .into_iter()
        .map(|arn| ctx.edge(&resource, arn.to_owned(), "TARGET_GROUP_OF"))
        .collect();
    if let Some(id) = model.vpc_id() {
        edges.push(ctx.edge(&resource, ctx.ec2_arn("vpc", id), "IN_VPC"));
    }
    Ok(PageBody {
        resources: vec![resource],
        edges,
        ..PageBody::default()
    })
}

pub(super) fn map_target_health(
    ctx: &Context,
    group: &str,
    target_type: &str,
    model: &TargetHealthDescription,
) -> Result<PageBody, AwsScanError> {
    let target = model
        .target()
        .ok_or_else(|| AwsScanError::InvalidData("target-health record lacks target".into()))?;
    let target_id = required(target.id(), "registered target ID")?;
    // JSON tuple encoding preserves absent values and avoids delimiter collisions.
    let id = format!(
        "aws-elbv2-target-health:{}",
        json!([
            group,
            target_id,
            target.port(),
            target.availability_zone(),
            target.quic_server_id(),
        ])
    );
    let health = model.target_health();
    let resource = ctx.resource("aws_elbv2_target_health", id, None, json!({
        "target_group_arn": group, "target_type": target_type, "target_id": target_id,
        "target_port": target.port(), "availability_zone": target.availability_zone(),
        "quic_server_id": target.quic_server_id(), "health_check_port": model.health_check_port(),
        "target_health_state": health.and_then(|health| health.state()).map(|value| value.as_str()),
        "target_health_reason": health.and_then(|health| health.reason()).map(|value| value.as_str()),
        "target_health_description": health.and_then(|health| health.description()),
        "administrative_override": model.administrative_override().map(|value| json!({
            "state": value.state().map(|state| state.as_str()), "reason": value.reason().map(|reason| reason.as_str()),
            "description": value.description(),
        })),
        "anomaly_detection_requested": false, "tags_collected": false,
    }), BTreeMap::new())?;
    let membership = Edge {
        from_id: group.to_owned(),
        to_id: resource.id.clone(),
        relation: "HAS_TARGET".into(),
        source: resource.source,
        observed_at: resource.observed_at,
        properties: json!({"evidence": "DescribeTargetHealth", "target_type": target_type}),
    };
    let target_edge = target_reference(ctx, &resource, group, target_type, target, &target_id)?;
    Ok(PageBody {
        resources: vec![resource],
        edges: vec![membership, target_edge],
        ..PageBody::default()
    })
}

pub(super) fn target_reference(
    ctx: &Context,
    resource: &Resource,
    group: &str,
    target_type: &str,
    target: &TargetDescription,
    target_id: &str,
) -> Result<Edge, AwsScanError> {
    let (reference, target_kind, ownership) = match target_type {
        "instance" => (
            ctx.ec2_arn("instance", target_id),
            "aws_ec2_instance",
            "assumed_scanning_account",
        ),
        "lambda" => (
            returned_target_arn(target_id, "lambda", "function:")?,
            "aws_lambda_function",
            "returned_arn",
        ),
        "alb" => (
            returned_target_arn(target_id, "elasticloadbalancing", "loadbalancer/app/")?,
            "aws_elbv2_load_balancer",
            "returned_arn",
        ),
        "ip" => {
            target_id.parse::<std::net::IpAddr>().map_err(|error| {
                AwsScanError::InvalidData(format!("invalid registered IP address: {error}"))
            })?;
            (
                format!("aws-elbv2-ip-target:{}", json!([group, target_id])),
                "ip_address",
                "unknown",
            )
        }
        _ => (
            format!(
                "aws-elbv2-unknown-target:{}",
                json!([group, target_type, target_id])
            ),
            "unknown",
            "unknown",
        ),
    };
    let mut edge = ctx.edge(resource, reference, "TARGETS");
    let target_region = match target_type {
        "instance" => Some(ctx.region.as_str()),
        "lambda" | "alb" => target_id.split(':').nth(3),
        _ => None,
    };
    edge.properties = json!({
        "evidence": "DescribeTargetHealth", "target_group_arn": group,
        "target_type": target_type, "target_kind": target_kind, "target_resource_id": target_id,
        "target_region": target_region, "observation_region": ctx.region,
        "target_address": (target_type == "ip").then_some(target_id), "owner_resolution": ownership,
        "target_port": target.port(), "availability_zone": target.availability_zone(),
        "quic_server_id": target.quic_server_id(),
    });
    Ok(edge)
}

pub(super) fn returned_target_arn(
    id: &str,
    service: &str,
    resource_prefix: &str,
) -> Result<String, AwsScanError> {
    let mut parts = id.splitn(6, ':');
    if let (
        Some("arn"),
        Some(partition),
        Some(actual_service),
        Some(region),
        Some(account),
        Some(resource),
    ) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) && !partition.is_empty()
        && actual_service == service
        && !region.is_empty()
        && !account.is_empty()
        && resource
            .strip_prefix(resource_prefix)
            .is_some_and(|name| !name.is_empty())
    {
        return Ok(id.to_owned());
    }
    Err(AwsScanError::InvalidData(format!(
        "registered {service} target has an invalid ARN"
    )))
}
