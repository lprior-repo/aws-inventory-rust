use super::*;

pub(super) fn map_page<T, F>(
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

pub(super) fn check_nested<T>(items: &[T], field: &str) -> Result<(), AwsScanError> {
    if items.len() > MAX_NESTED_ITEMS {
        return Err(AwsScanError::InvalidData(format!(
            "ELBv2 {field} exceeds nested item limit {MAX_NESTED_ITEMS}"
        )));
    }
    Ok(())
}

pub(super) fn map_load_balancer(
    ctx: &Context,
    model: &LoadBalancer,
) -> Result<PageBody, AwsScanError> {
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

pub(super) fn map_zone(zone: &AvailabilityZone) -> Result<Value, AwsScanError> {
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

pub(super) fn map_listener(
    ctx: &Context,
    parent: &str,
    model: &Listener,
) -> Result<PageBody, AwsScanError> {
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
