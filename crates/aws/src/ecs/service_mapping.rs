//! service mapping responsibilities for ecs.
use super::*;

pub(super) fn map_service(
    ctx: &Context,
    body: &mut PageBody,
    service: &Service,
) -> Result<(), AwsScanError> {
    let network = service
        .network_configuration()
        .and_then(|network| network.awsvpc_configuration());
    let components = service
        .load_balancers()
        .len()
        .saturating_add(network.map_or(0, |value| value.subnets().len()))
        .saturating_add(network.map_or(0, |value| value.security_groups().len()));
    if components > MAX_SERVICE_REFERENCES {
        return Err(AwsScanError::InvalidData(
            "ECS service reference capacity exceeded".into(),
        ));
    }
    let target_groups: BTreeSet<_> = service
        .load_balancers()
        .iter()
        .flat_map(|value| {
            [
                value.target_group_arn(),
                value
                    .advanced_configuration()
                    .and_then(|value| value.alternate_target_group_arn()),
            ]
        })
        .flatten()
        .collect();
    let classic_balancers: BTreeSet<_> = service
        .load_balancers()
        .iter()
        .filter_map(|value| value.load_balancer_name())
        .collect();
    let resource = ctx.resource("aws_ecs_service", required(service.service_arn(), "ECS service ARN")?,
        service.service_name().map(str::to_owned), json!({
            "service_arn": service.service_arn(), "service_name": service.service_name(), "cluster_arn": service.cluster_arn(),
            "status": service.status(), "task_definition": service.task_definition(), "desired_count": service.desired_count(),
            "running_count": service.running_count(), "pending_count": service.pending_count(),
            "launch_type": service.launch_type().map(|value|value.as_str()), "platform_version": service.platform_version(),
            "scheduling_strategy": service.scheduling_strategy().map(|value|value.as_str()),
            "subnet_ids": network.map(|network|network.subnets()).unwrap_or_default(),
            "security_group_ids": network.map(|network|network.security_groups()).unwrap_or_default(),
            "target_group_arns": target_groups, "classic_load_balancer_names": classic_balancers,
            "load_balancers": service.load_balancers().iter().map(|value| json!({
                "target_group_arn": value.target_group_arn(), "load_balancer_name": value.load_balancer_name(),
                "container_name": value.container_name(), "container_port": value.container_port(),
                "advanced_configuration": value.advanced_configuration().map(|value| json!({
                    "alternate_target_group_arn": value.alternate_target_group_arn(),
                    "production_listener_rule": value.production_listener_rule(),
                    "test_listener_rule": value.test_listener_rule(), "role_arn": value.role_arn(),
                })),
            })).collect::<Vec<_>>(),
        }), tags(service.tags()))?;
    resource.validate()?;
    let mut links = BTreeMap::new();
    if let Some(arn) = service.cluster_arn() {
        merge_reference_edge(
            &mut links,
            ctx.edge(&resource, arn.to_owned(), "IN_CLUSTER"),
            json!({"field":"cluster_arn"}),
        )?;
    }
    if let Some(arn) = service.task_definition() {
        merge_reference_edge(
            &mut links,
            ctx.edge(&resource, arn.to_owned(), "USES_TASK_DEFINITION"),
            json!({"field":"task_definition"}),
        )?;
    }
    if let Some(network) = network {
        for subnet in network.subnets() {
            merge_reference_edge(
                &mut links,
                ctx.edge(&resource, ctx.ec2_arn("subnet", subnet), "USES_SUBNET"),
                json!({"field":"subnet_ids"}),
            )?;
        }
        for group in network.security_groups() {
            merge_reference_edge(
                &mut links,
                ctx.edge(
                    &resource,
                    ctx.ec2_arn("security-group", group),
                    "USES_SECURITY_GROUP",
                ),
                json!({"field":"security_group_ids"}),
            )?;
        }
    }
    for (index, balancer) in service.load_balancers().iter().enumerate() {
        for (arn, role) in [
            (balancer.target_group_arn(), "primary"),
            (
                balancer
                    .advanced_configuration()
                    .and_then(|value| value.alternate_target_group_arn()),
                "alternate",
            ),
        ] {
            if let Some(arn) = arn {
                let edge = target_group_link(ctx, &resource, arn)?;
                merge_reference_edge(
                    &mut links,
                    edge,
                    json!({
                        "load_balancer_index":index, "configuration_role":role,
                        "container_name":balancer.container_name(), "container_port":balancer.container_port(),
                    }),
                )?;
            }
        }
        if let Some(name) = balancer.load_balancer_name() {
            let name = required(Some(name), "Classic ELB name")?;
            let mut edge = ctx.edge(
                &resource,
                ctx.arn("elasticloadbalancing", &format!("loadbalancer/{name}")),
                "USES_CLASSIC_LOAD_BALANCER",
            );
            edge.properties = json!({"target_resource_id":name,"target_kind":"aws_ec2_classic_load_balancer",
                "target_account_id":ctx.account,"target_region":ctx.region,"owner_resolution":"assumed_scanning_account"});
            merge_reference_edge(
                &mut links,
                edge,
                json!({"load_balancer_index":index,
                "container_name":balancer.container_name(),"container_port":balancer.container_port()}),
            )?;
        }
    }
    for edge in links.into_values() {
        edge.validate()?;
        body.edges.push(edge);
    }
    body.resources.push(resource);
    Ok(())
}

pub(super) const MAX_SERVICE_REFERENCES: usize = 1_024;

/// One relationship can carry several container/port registrations. Retain the
/// observations while matching the store's (from,to,relation) identity.
pub(crate) fn merge_reference_edge(
    links: &mut BTreeMap<(String, String, String), Edge>,
    mut edge: Edge,
    evidence: Value,
) -> Result<(), AwsScanError> {
    let key = (
        edge.from_id.clone(),
        edge.to_id.clone(),
        edge.relation.clone(),
    );
    if let Some(existing) = links.get_mut(&key) {
        let observations = existing
            .properties
            .get_mut("evidence")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| {
                AwsScanError::InvalidData("resource edge evidence is not an array".into())
            })?;
        if observations.len() >= MAX_SERVICE_REFERENCES {
            return Err(AwsScanError::InvalidData(
                "resource edge evidence capacity exceeded".into(),
            ));
        }
        observations.push(evidence);
    } else {
        let properties = edge.properties.as_object_mut().ok_or_else(|| {
            AwsScanError::InvalidData("resource edge properties are not an object".into())
        })?;
        properties.insert("evidence".into(), json!([evidence]));
        links.insert(key, edge);
    }
    Ok(())
}

pub(super) fn target_group_link(
    ctx: &Context,
    resource: &Resource,
    arn: &str,
) -> Result<Edge, AwsScanError> {
    let mut parts = arn.splitn(6, ':');
    let (
        Some("arn"),
        Some(partition),
        Some("elasticloadbalancing"),
        Some(region),
        Some(account),
        Some(path),
    ) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    )
    else {
        return Err(AwsScanError::InvalidData(
            "ECS load balancer target group has an invalid ARN".into(),
        ));
    };
    if partition.is_empty()
        || region.is_empty()
        || account.is_empty()
        || !path
            .strip_prefix("targetgroup/")
            .and_then(|value| value.split_once('/'))
            .is_some_and(|(name, id)| !name.is_empty() && !id.is_empty())
    {
        return Err(AwsScanError::InvalidData(
            "ECS load balancer target group has an invalid ARN".into(),
        ));
    }
    let mut edge = ctx.edge(resource, arn.to_owned(), "USES_TARGET_GROUP");
    edge.properties = json!({"target_resource_id":arn,"target_kind":"aws_elbv2_target_group",
        "target_account_id":account,"target_region":region,"owner_resolution":"aws_returned_arn"});
    Ok(edge)
}
