//! ELBv2 topology and target-registration evidence. Every child has its own scope.

use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;

use aws_sdk_elasticloadbalancingv2::{
    Client,
    operation::describe_target_health::DescribeTargetHealthOutput,
    types::{
        Action, AvailabilityZone, Listener, LoadBalancer, Rule, RuleCondition, RuleTransform,
        TargetDescription, TargetGroup, TargetHealthDescription,
    },
};
use inventory_core::{Edge, Resource, Source};
use inventory_engine::limiter::ApiError;
use serde_json::{Value, json};

use crate::{
    AwsScanError,
    common::{Context, PageBody, collect_bounded, paginate, require_budget, required, sdk_error},
};

const CHILD_CONCURRENCY: usize = 4;
const MAX_NESTED_ITEMS: usize = 1_024;

#[tracing::instrument(skip_all)]
pub(crate) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    let client = Client::new(&ctx.config);
    // These roots are independent: unattached target groups must also be inventoried.
    collect_bounded([false, true], 2, |groups| {
        let client = &client;
        async move {
            if groups {
                target_groups(ctx, client).await
            } else {
                load_balancers(ctx, client).await
            }
        }
    })
    .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn load_balancers(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope("aws_elbv2_load_balancer", None, Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("elasticloadbalancing", "read", || async {
                    client
                        .describe_load_balancers()
                        .page_size(ctx.page_size.clamp(1, 400))
                        .set_marker(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            let body = map_page(
                output.load_balancers(),
                budget,
                output.next_marker(),
                |model| map_load_balancer(ctx, model),
            )?;
            collect_bounded(
                output.load_balancers().iter(),
                CHILD_CONCURRENCY,
                |model| async move {
                    let arn = required(model.load_balancer_arn(), "load balancer ARN")?;
                    listeners(ctx, client, &arn, application_rules(model)).await
                },
            )
            .await?;
            Ok(body)
        },
    )
    .await
}

#[tracing::instrument(skip_all, fields(load_balancer_arn))]
async fn listeners(
    ctx: &Context,
    client: &Client,
    load_balancer_arn: &str,
    collect_rules: bool,
) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope(
            "aws_elbv2_listener",
            Some(load_balancer_arn),
            Source::Direct,
        ),
        |token, budget| async move {
            let output = ctx
                .execute("elasticloadbalancing", "read", || async {
                    client
                        .describe_listeners()
                        .load_balancer_arn(load_balancer_arn)
                        .page_size(ctx.page_size.clamp(1, 400))
                        .set_marker(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            let body = map_page(output.listeners(), budget, output.next_marker(), |model| {
                map_listener(ctx, load_balancer_arn, model)
            })?;
            if collect_rules {
                collect_bounded(
                    output.listeners().iter(),
                    CHILD_CONCURRENCY,
                    |model| async move {
                        let arn = required(model.listener_arn(), "listener ARN")?;
                        rules(ctx, client, &arn).await
                    },
                )
                .await?;
            }
            Ok(body)
        },
    )
    .await
}

#[tracing::instrument(skip_all, fields(listener_arn))]
async fn rules(ctx: &Context, client: &Client, listener_arn: &str) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope("aws_elbv2_rule", Some(listener_arn), Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("elasticloadbalancing", "read", || async {
                    client
                        .describe_rules()
                        .listener_arn(listener_arn)
                        .page_size(ctx.page_size.clamp(1, 400))
                        .set_marker(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            map_page(output.rules(), budget, output.next_marker(), |model| {
                map_rule(ctx, listener_arn, model)
            })
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
async fn target_groups(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope("aws_elbv2_target_group", None, Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("elasticloadbalancing", "read", || async {
                    client
                        .describe_target_groups()
                        .page_size(ctx.page_size.clamp(1, 400))
                        .set_marker(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            let body = map_page(
                output.target_groups(),
                budget,
                output.next_marker(),
                |model| map_target_group(ctx, model),
            )?;
            collect_bounded(
                output.target_groups().iter(),
                CHILD_CONCURRENCY,
                |model| async move {
                    let arn = required(model.target_group_arn(), "target group ARN")?;
                    let target_type =
                        required(model.target_type().map(|kind| kind.as_str()), "target type")?;
                    target_health_with(ctx, &arn, &target_type, || {
                        ctx.execute("elasticloadbalancing", "read", || async {
                            client
                                .describe_target_health()
                                .target_group_arn(&arn)
                                .send()
                                .await
                                .map_err(sdk_error)
                        })
                    })
                    .await
                },
            )
            .await?;
            Ok(body)
        },
    )
    .await
}

/// DescribeTargetHealth has no pagination. Reuse the scope driver so a failed
/// request never becomes an empty successful result, including in injected tests.
#[tracing::instrument(skip_all, fields(target_group_arn, target_type))]
async fn target_health_with<F, Fut>(
    ctx: &Context,
    target_group_arn: &str,
    target_type: &str,
    fetch: F,
) -> Result<(), AwsScanError>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = Result<DescribeTargetHealthOutput, ApiError>> + Send,
{
    paginate(
        ctx,
        ctx.scope(
            "aws_elbv2_target_health",
            Some(target_group_arn),
            Source::Direct,
        ),
        |token, budget| {
            let fetch = &fetch;
            async move {
                if token.is_some() {
                    return Err(AwsScanError::InvalidData(
                        "DescribeTargetHealth does not support continuation tokens".into(),
                    ));
                }
                let output = fetch().await?;
                map_page(output.target_health_descriptions(), budget, None, |model| {
                    map_target_health(ctx, target_group_arn, target_type, model)
                })
            }
        },
    )
    .await
}

fn application_rules(model: &LoadBalancer) -> bool {
    model
        .r#type()
        .is_some_and(|kind| kind.as_str() == "application")
}

/// Bound enrichment before allocating mapped rows; each mapper emits one row.
/// Payload accounting caps retained mapped JSON while the SDK response is alive.
fn map_page<T, F>(
    models: &[T],
    budget: usize,
    next_marker: Option<&str>,
    mut map: F,
) -> Result<PageBody, AwsScanError>
where
    F: FnMut(&T) -> Result<PageBody, AwsScanError>,
{
    require_budget(models.len(), budget)?;
    let mut body = PageBody {
        listed_count: models.len(),
        next_token: next_marker.map(str::to_owned),
        ..PageBody::default()
    };
    let mut payload_bytes = 0usize;
    let mut resource_ids = BTreeSet::new();
    for model in models {
        let mut mapped = map(model)?;
        for resource in &mapped.resources {
            resource.validate()?;
            if !resource_ids.insert(resource.id.clone()) {
                return Err(AwsScanError::InvalidData(
                    "ELBv2 page repeats a resource identity".into(),
                ));
            }
        }
        for edge in &mapped.edges {
            edge.validate()?;
        }
        if body.resources.len().saturating_add(mapped.resources.len())
            > inventory_core::MAX_PAGE_RESOURCES
            || body.edges.len().saturating_add(mapped.edges.len()) > inventory_core::MAX_PAGE_EDGES
        {
            return Err(AwsScanError::InvalidData(
                "ELBv2 mapped page exceeds row or edge limit".into(),
            ));
        }
        let encoded = serde_json::to_vec(&(&mapped.resources, &mapped.edges))
            .map_err(|error| AwsScanError::InvalidData(error.to_string()))?;
        payload_bytes = payload_bytes.saturating_add(encoded.len());
        if payload_bytes > inventory_core::MAX_PAGE_BYTES {
            return Err(AwsScanError::InvalidData(
                "ELBv2 mapped page exceeds byte limit".into(),
            ));
        }
        body.resources.append(&mut mapped.resources);
        body.edges.append(&mut mapped.edges);
    }
    Ok(body)
}

fn check_nested<T>(items: &[T], field: &str) -> Result<(), AwsScanError> {
    if items.len() > MAX_NESTED_ITEMS {
        return Err(AwsScanError::InvalidData(format!(
            "ELBv2 {field} exceeds nested item limit {MAX_NESTED_ITEMS}"
        )));
    }
    Ok(())
}

fn map_load_balancer(ctx: &Context, model: &LoadBalancer) -> Result<PageBody, AwsScanError> {
    check_nested(model.availability_zones(), "availability zones")?;
    check_nested(model.security_groups(), "security groups")?;
    let kind = required(
        model.r#type().map(|value| value.as_str()),
        "load balancer type",
    )?;
    let zones = model
        .availability_zones()
        .iter()
        .map(map_zone)
        .collect::<Result<Vec<_>, _>>()?;
    let subnets: BTreeSet<_> = model
        .availability_zones()
        .iter()
        .filter_map(|zone| zone.subnet_id())
        .collect();
    let groups: BTreeSet<_> = model.security_groups().iter().map(String::as_str).collect();
    let resource = ctx.resource(
        "aws_elbv2_load_balancer",
        required(model.load_balancer_arn(), "load balancer ARN")?,
        model.load_balancer_name().map(str::to_owned),
        json!({
            "load_balancer_arn": model.load_balancer_arn(), "load_balancer_name": model.load_balancer_name(),
            "type": kind, "scheme": model.scheme().map(|value| value.as_str()),
            "dns_name": model.dns_name(), "state": model.state().and_then(|state| state.code()).map(|value| value.as_str()),
            "state_reason": model.state().and_then(|state| state.reason()),
            "ip_address_type": model.ip_address_type().map(|value| value.as_str()),
            "vpc_id": model.vpc_id(), "subnet_ids": subnets, "security_group_ids": groups,
            "availability_zones": zones, "tags_collected": false,
        }),
        BTreeMap::new(),
    )?;
    let mut edges = Vec::new();
    if let Some(id) = model.vpc_id() {
        edges.push(ctx.edge(&resource, ctx.ec2_arn("vpc", id), "IN_VPC"));
    }
    edges.extend(
        subnets
            .into_iter()
            .map(|id| ctx.edge(&resource, ctx.ec2_arn("subnet", id), "USES_SUBNET")),
    );
    edges.extend(groups.into_iter().map(|id| {
        ctx.edge(
            &resource,
            ctx.ec2_arn("security-group", id),
            "USES_SECURITY_GROUP",
        )
    }));
    Ok(PageBody {
        resources: vec![resource],
        edges,
        ..PageBody::default()
    })
}

fn map_zone(zone: &AvailabilityZone) -> Result<Value, AwsScanError> {
    check_nested(zone.load_balancer_addresses(), "load balancer addresses")?;
    check_nested(zone.source_nat_ipv6_prefixes(), "source NAT IPv6 prefixes")?;
    Ok(json!({
        "zone_name": zone.zone_name(), "subnet_id": zone.subnet_id(), "outpost_id": zone.outpost_id(),
        "source_nat_ipv6_prefixes": zone.source_nat_ipv6_prefixes(),
        "load_balancer_addresses": zone.load_balancer_addresses().iter().map(|address| json!({
            "ip_address": address.ip_address(), "allocation_id": address.allocation_id(),
            "private_ipv4_address": address.private_ipv4_address(), "ipv6_address": address.ipv6_address(),
        })).collect::<Vec<_>>(),
    }))
}

fn map_listener(ctx: &Context, parent: &str, model: &Listener) -> Result<PageBody, AwsScanError> {
    check_nested(model.certificates(), "listener certificates")?;
    check_nested(model.alpn_policy(), "ALPN policies")?;
    if model.load_balancer_arn().is_some_and(|arn| arn != parent) {
        return Err(AwsScanError::InvalidData(
            "listener belongs to a different load balancer than its scope".into(),
        ));
    }
    let (actions, targets) = map_actions(model.default_actions())?;
    let resource = ctx.resource(
        "aws_elbv2_listener", required(model.listener_arn(), "listener ARN")?, None,
        json!({
            "listener_arn": model.listener_arn(), "load_balancer_arn": parent,
            "port": model.port(), "protocol": model.protocol().map(|value| value.as_str()),
            "ssl_policy": model.ssl_policy(), "alpn_policy": model.alpn_policy(),
            "certificates": model.certificates().iter().map(|certificate| json!({
                "certificate_arn": certificate.certificate_arn(), "is_default": certificate.is_default(),
            })).collect::<Vec<_>>(),
            "certificate_list_complete": false,
            "default_actions": actions, "default_target_group_arns": targets, "tags_collected": false,
        }), BTreeMap::new(),
    )?;
    let mut edges = vec![ctx.edge(&resource, parent.to_owned(), "LISTENER_OF")];
    edges.extend(
        targets
            .into_iter()
            .map(|arn| ctx.edge(&resource, arn, "FORWARDS_TO")),
    );
    Ok(PageBody {
        resources: vec![resource],
        edges,
        ..PageBody::default()
    })
}

fn map_target_group(ctx: &Context, model: &TargetGroup) -> Result<PageBody, AwsScanError> {
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

fn map_target_health(
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

fn target_reference(
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

fn returned_target_arn(
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

fn map_rule(ctx: &Context, listener: &str, model: &Rule) -> Result<PageBody, AwsScanError> {
    check_nested(model.conditions(), "rule conditions")?;
    check_nested(model.transforms(), "rule transforms")?;
    let conditions = model
        .conditions()
        .iter()
        .map(map_condition)
        .collect::<Result<Vec<_>, _>>()?;
    let transforms = model
        .transforms()
        .iter()
        .map(map_transform)
        .collect::<Result<Vec<_>, _>>()?;
    let (actions, targets) = map_actions(model.actions())?;
    let resource = ctx.resource(
        "aws_elbv2_rule",
        required(model.rule_arn(), "rule ARN")?,
        None,
        json!({
            "rule_arn": model.rule_arn(), "listener_arn": listener, "priority": model.priority(),
            "is_default": model.is_default(), "conditions": conditions, "actions": actions,
            "transforms": transforms, "target_group_arns": targets, "tags_collected": false,
        }),
        BTreeMap::new(),
    )?;
    let mut edges = vec![ctx.edge(&resource, listener.to_owned(), "RULE_OF")];
    edges.extend(
        targets
            .into_iter()
            .map(|arn| ctx.edge(&resource, arn, "FORWARDS_TO")),
    );
    Ok(PageBody {
        resources: vec![resource],
        edges,
        ..PageBody::default()
    })
}

fn map_actions(actions: &[Action]) -> Result<(Vec<Value>, BTreeSet<String>), AwsScanError> {
    check_nested(actions, "listener actions")?;
    let mut targets = BTreeSet::new();
    let values = actions.iter().map(|action| {
        let action_type = required(action.r#type().map(|kind| kind.as_str()), "listener action type")?;
        let forward = action.forward_config();
        let groups = forward.map(|config| config.target_groups()).unwrap_or_default();
        check_nested(groups, "forward target groups")?;
        // Only forward actions establish routing edges; authentication/redirect
        // settings remain their own action descriptions.
        if action_type == "forward" {
            if action.target_group_arn().is_none() && groups.is_empty() {
                return Err(AwsScanError::InvalidData("forward action lacks target groups".into()));
            }
            if let Some(arn) = action.target_group_arn() {
                targets.insert(required(Some(arn), "forward target group ARN")?);
            }
            for group in groups {
                targets.insert(required(group.target_group_arn(), "weighted target group ARN")?);
            }
        }
        Ok(json!({
            "type": action_type, "order": action.order(),
            "target_group_arn": action.target_group_arn(),
            "forward_config": forward.map(|config| json!({
                "target_groups": config.target_groups().iter().map(|group| json!({
                    "target_group_arn": group.target_group_arn(), "weight": group.weight(),
                })).collect::<Vec<_>>(),
                "target_group_stickiness_config": config.target_group_stickiness_config().map(|value| json!({
                    "enabled": value.enabled(), "duration_seconds": value.duration_seconds(),
                })),
            })),
            "redirect_config": action.redirect_config().map(|value| json!({
                "protocol": value.protocol(), "port": value.port(), "host": value.host(),
                "path": value.path(), "query": value.query(), "status_code": value.status_code().map(|status| status.as_str()),
            })),
            "fixed_response_config": action.fixed_response_config().map(|value| json!({
                "status_code": value.status_code(), "content_type": value.content_type(), "message_body": value.message_body(),
            })),
            "authenticate_oidc_config_present": action.authenticate_oidc_config().is_some(),
            "authenticate_cognito_config_present": action.authenticate_cognito_config().is_some(),
            "jwt_validation_config_present": action.jwt_validation_config().is_some(),
            "authentication_details_collected": false,
        }))
    }).collect::<Result<Vec<_>, AwsScanError>>()?;
    Ok((values, targets))
}

fn map_condition(condition: &RuleCondition) -> Result<Value, AwsScanError> {
    check_nested(condition.values(), "condition values")?;
    check_nested(condition.regex_values(), "condition regex values")?;
    let host = condition.host_header_config();
    let path = condition.path_pattern_config();
    let header = condition.http_header_config();
    let methods = condition
        .http_request_method_config()
        .map(|value| value.values())
        .unwrap_or_default();
    let ips = condition
        .source_ip_config()
        .map(|value| value.values())
        .unwrap_or_default();
    let query = condition
        .query_string_config()
        .map(|value| value.values())
        .unwrap_or_default();
    for values in [
        host.map(|value| value.values()).unwrap_or_default(),
        host.map(|value| value.regex_values()).unwrap_or_default(),
        path.map(|value| value.values()).unwrap_or_default(),
        path.map(|value| value.regex_values()).unwrap_or_default(),
        header.map(|value| value.values()).unwrap_or_default(),
        header.map(|value| value.regex_values()).unwrap_or_default(),
        methods,
        ips,
    ] {
        check_nested(values, "condition configuration values")?;
    }
    check_nested(query, "query string condition values")?;
    Ok(json!({
        "field": condition.field(), "values": condition.values(), "regex_values": condition.regex_values(),
        "host_header_config": host.map(|value| json!({"values": value.values(), "regex_values": value.regex_values()})),
        "path_pattern_config": path.map(|value| json!({"values": value.values(), "regex_values": value.regex_values()})),
        "http_header_config": header.map(|value| json!({"http_header_name": value.http_header_name(), "values": value.values(), "regex_values": value.regex_values()})),
        "http_request_method_config": condition.http_request_method_config().map(|value| json!({"values": value.values()})),
        "source_ip_config": condition.source_ip_config().map(|value| json!({"values": value.values(), "ip_address_type": value.ip_address_type().map(|kind| kind.as_str())})),
        "query_string_config": condition.query_string_config().map(|value| json!({"values": value.values().iter().map(|pair| json!({"key": pair.key(), "value": pair.value()})).collect::<Vec<_>>()})),
    }))
}

fn map_transform(transform: &RuleTransform) -> Result<Value, AwsScanError> {
    let host = transform
        .host_header_rewrite_config()
        .map(|value| value.rewrites())
        .unwrap_or_default();
    let url = transform
        .url_rewrite_config()
        .map(|value| value.rewrites())
        .unwrap_or_default();
    check_nested(host, "host header rewrites")?;
    check_nested(url, "URL rewrites")?;
    Ok(json!({
        "type": transform.r#type().map(|value| value.as_str()),
        "host_header_rewrites": host.iter().map(|rewrite| json!({"regex": rewrite.regex(), "replace": rewrite.replace()})).collect::<Vec<_>>(),
        "url_rewrites": url.iter().map(|rewrite| json!({"regex": rewrite.regex(), "replace": rewrite.replace()})).collect::<Vec<_>>(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_elasticloadbalancingv2::types::{
        ActionTypeEnum, AuthenticateOidcActionConfig, ForwardActionConfig,
        HostHeaderConditionConfig, LoadBalancerTypeEnum, ProtocolEnum, QueryStringConditionConfig,
        QueryStringKeyValuePair, RewriteConfig, TargetGroupTuple, TargetHealth,
        TargetHealthReasonEnum, TargetHealthStateEnum, TargetTypeEnum, TransformTypeEnum,
        UrlRewriteConfig,
    };
    use inventory_core::CollectionEvent;
    use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
    use std::sync::Arc;
    use tokio::sync::mpsc;

    type TestError = Box<dyn std::error::Error>;
    const LB: &str =
        "arn:aws:elasticloadbalancing:us-east-1:111111111111:loadbalancer/app/front/abc";
    const LISTENER: &str =
        "arn:aws:elasticloadbalancing:us-east-1:111111111111:listener/app/front/abc/def";
    const GROUP: &str = "arn:aws:elasticloadbalancing:us-east-1:111111111111:targetgroup/api/abc";
    const SECOND_GROUP: &str =
        "arn:aws:elasticloadbalancing:us-east-1:111111111111:targetgroup/canary/def";

    fn context() -> Result<(Context, mpsc::Receiver<CollectionEvent>), ApiError> {
        let (sender, receiver) = mpsc::channel(8);
        Ok((
            Context {
                config: aws_types::SdkConfig::builder().build(),
                executor: Arc::new(ApiExecutor::new(ExecutorConfig::default())?),
                sender,
                account: "111111111111".into(),
                partition: "aws".into(),
                region: "us-east-1".into(),
                max_pages: 10,
                page_size: 100,
            },
            receiver,
        ))
    }

    fn first_resource(body: &PageBody) -> Result<&Resource, TestError> {
        body.resources
            .first()
            .ok_or_else(|| "missing mapped resource".into())
    }

    fn target_edge(body: &PageBody) -> Result<&Edge, TestError> {
        body.edges
            .iter()
            .find(|edge| edge.relation == "TARGETS")
            .ok_or_else(|| "missing target relation".into())
    }

    fn registered(id: &str, port: Option<i32>, zone: Option<&str>) -> TargetHealthDescription {
        TargetHealthDescription::builder()
            .target(
                TargetDescription::builder()
                    .id(id)
                    .set_port(port)
                    .set_availability_zone(zone.map(str::to_owned))
                    .build(),
            )
            .health_check_port("8080")
            .target_health(
                TargetHealth::builder()
                    .state(TargetHealthStateEnum::Unhealthy)
                    .reason(TargetHealthReasonEnum::FailedHealthChecks)
                    .description("Health checks failed")
                    .build(),
            )
            .build()
    }

    fn weighted_forward() -> Action {
        Action::builder()
            .r#type(ActionTypeEnum::Forward)
            .order(2)
            .target_group_arn(GROUP)
            .forward_config(
                ForwardActionConfig::builder()
                    .target_groups(
                        TargetGroupTuple::builder()
                            .target_group_arn(GROUP)
                            .weight(90)
                            .build(),
                    )
                    .target_groups(
                        TargetGroupTuple::builder()
                            .target_group_arn(SECOND_GROUP)
                            .weight(10)
                            .build(),
                    )
                    .target_groups(
                        TargetGroupTuple::builder()
                            .target_group_arn(GROUP)
                            .weight(0)
                            .build(),
                    )
                    .build(),
            )
            .build()
    }

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
    fn listener_preserves_weighted_actions_and_deduplicates_forward_edges() -> Result<(), TestError>
    {
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
    fn unattached_target_group_remains_a_resource_with_health_configuration()
    -> Result<(), TestError> {
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

    #[test]
    fn authentication_action_exposes_presence_without_copying_client_secret()
    -> Result<(), TestError> {
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
    fn ip_target_reference_has_unknown_owner_and_is_scoped_to_target_group() -> Result<(), TestError>
    {
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
    fn returned_lambda_and_alb_arns_are_preserved_including_lambda_qualifier()
    -> Result<(), TestError> {
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

    #[test]
    fn malformed_identity_and_nested_overflow_are_explicit_errors() -> Result<(), TestError> {
        let (ctx, _events) = context()?;
        assert!(
            map_load_balancer(
                &ctx,
                &LoadBalancer::builder()
                    .r#type(LoadBalancerTypeEnum::Application)
                    .build()
            )
            .is_err()
        );
        assert!(
            map_target_health(&ctx, GROUP, "ip", &registered("not-an-IP", None, None)).is_err()
        );
        assert!(
            map_target_health(&ctx, GROUP, "lambda", &registered("not-an-ARN", None, None))
                .is_err()
        );
        assert!(
            returned_target_arn(
                "arn:aws:lambda:us-east-1::function:processor",
                "lambda",
                "function:"
            )
            .is_err()
        );
        assert!(map_actions(&[Action::builder().r#type(ActionTypeEnum::Forward).build()]).is_err());
        let actions = vec![weighted_forward(); MAX_NESTED_ITEMS.saturating_add(1)];
        assert!(matches!(
            map_actions(&actions),
            Err(AwsScanError::InvalidData(_))
        ));
        Ok(())
    }

    #[test]
    fn repeated_health_identity_fails_before_storage() -> Result<(), TestError> {
        let (ctx, _events) = context()?;
        let models = vec![registered("i-123", Some(8080), None); 2];
        let result = map_page(&models, 2, None, |model| {
            map_target_health(&ctx, GROUP, "instance", model)
        });
        assert!(
            matches!(result, Err(AwsScanError::InvalidData(message)) if message.contains("repeats a resource identity"))
        );
        Ok(())
    }

    #[test]
    fn oversized_unpaginated_health_response_is_rejected_before_mapping() {
        let models = vec![(); inventory_core::MAX_PAGE_RESOURCES.saturating_add(1)];
        let calls = std::cell::Cell::new(0usize);
        let result = map_page(&models, crate::common::MAX_CHILDREN, None, |_| {
            calls.set(calls.get().saturating_add(1));
            Ok(PageBody::default())
        });
        assert!(matches!(result, Err(AwsScanError::InvalidData(_))));
        assert_eq!(calls.get(), 0);
    }

    #[tokio::test]
    async fn denied_health_child_does_not_discard_parent_or_claim_empty_success()
    -> Result<(), TestError> {
        let (ctx, mut events) = context()?;
        let model = TargetGroup::builder()
            .target_group_arn(GROUP)
            .target_type(TargetTypeEnum::Instance)
            .build();
        paginate(
            &ctx,
            ctx.scope("aws_elbv2_target_group", None, Source::Direct),
            |_token, budget| {
                let model = &model;
                let ctx = &ctx;
                async move {
                    let body = map_page(std::slice::from_ref(model), budget, None, |model| {
                        map_target_group(ctx, model)
                    })?;
                    target_health_with(ctx, GROUP, "instance", || {
                        std::future::ready(Err(ApiError::Denied("fixture AccessDenied".into())))
                    })
                    .await?;
                    Ok(body)
                }
            },
        )
        .await?;
        assert!(
            matches!(events.try_recv()?, CollectionEvent::Failure(failure)
            if failure.scope.collector == "aws_elbv2_target_health" && failure.scope.parent_id.as_deref() == Some(GROUP) && failure.category == "access_denied")
        );
        assert!(matches!(events.try_recv()?, CollectionEvent::Page(page)
            if page.scope.collector == "aws_elbv2_target_group" && page.resources.len() == 1 && page.next_token.is_none()));
        assert!(matches!(
            events.try_recv(),
            Err(mpsc::error::TryRecvError::Empty)
        ));
        Ok(())
    }

    #[tokio::test]
    async fn missing_child_target_is_invalid_response_and_sibling_still_collects()
    -> Result<(), TestError> {
        let (ctx, mut events) = context()?;
        let malformed = DescribeTargetHealthOutput::builder()
            .target_health_descriptions(TargetHealthDescription::builder().build())
            .build();
        target_health_with(&ctx, GROUP, "instance", || {
            std::future::ready(Ok(malformed.clone()))
        })
        .await?;
        let valid = DescribeTargetHealthOutput::builder()
            .target_health_descriptions(registered("i-456", Some(80), None))
            .build();
        target_health_with(&ctx, SECOND_GROUP, "instance", || {
            std::future::ready(Ok(valid.clone()))
        })
        .await?;
        assert!(
            matches!(events.try_recv()?, CollectionEvent::Failure(failure)
            if failure.scope.parent_id.as_deref() == Some(GROUP) && failure.category == "invalid_response")
        );
        assert!(matches!(events.try_recv()?, CollectionEvent::Page(page)
            if page.scope.parent_id.as_deref() == Some(SECOND_GROUP) && page.resources.len() == 1));
        Ok(())
    }

    #[tokio::test]
    async fn deleted_target_group_is_failure_while_successful_empty_group_has_terminal_page()
    -> Result<(), TestError> {
        let (ctx, mut events) = context()?;
        target_health_with(&ctx, GROUP, "instance", || {
            std::future::ready(Err(ApiError::Permanent("TargetGroupNotFound".into())))
        })
        .await?;
        target_health_with(&ctx, SECOND_GROUP, "instance", || {
            std::future::ready(Ok(DescribeTargetHealthOutput::builder().build()))
        })
        .await?;
        assert!(
            matches!(events.try_recv()?, CollectionEvent::Failure(failure)
            if failure.scope.parent_id.as_deref() == Some(GROUP) && failure.category == "api_error")
        );
        assert!(matches!(events.try_recv()?, CollectionEvent::Page(page)
            if page.scope.parent_id.as_deref() == Some(SECOND_GROUP) && page.resources.is_empty() && page.page_number == 1 && page.next_token.is_none()));
        Ok(())
    }
}
