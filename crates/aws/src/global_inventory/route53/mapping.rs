//! mapping responsibilities for route53.
use super::*;

pub(super) fn record_identity(
    ctx: &Context,
    zone: &str,
    record: &ResourceRecordSet,
) -> Result<String, AwsScanError> {
    let name = record
        .name()
        .strip_suffix('.')
        .filter(|value| !value.is_empty())
        .unwrap_or(record.name())
        .to_ascii_lowercase();
    let identifier = record.set_identifier().unwrap_or("<simple>");
    let identified = if record.set_identifier().is_some() {
        "identified"
    } else {
        "simple"
    };
    digest_identity(
        "route53-record",
        &[
            &ctx.partition,
            &ctx.account,
            zone,
            &name,
            record.r#type().as_str(),
            identified,
            identifier,
        ],
    )
}

pub(super) fn safe_record_values(
    record: &ResourceRecordSet,
) -> Result<(Vec<String>, bool), AwsScanError> {
    if record.resource_records().len() > MAX_RECORD_VALUES {
        return Err(AwsScanError::InvalidData(
            "DNS record exceeds 512 RDATA values".into(),
        ));
    }
    let allowed = matches!(
        record.r#type().as_str(),
        "A" | "AAAA" | "CNAME" | "NS" | "PTR" | "MX" | "SRV" | "SOA" | "CAA"
    );
    if !allowed {
        return Ok((Vec::new(), !record.resource_records().is_empty()));
    }
    let values = record
        .resource_records()
        .iter()
        .map(|value| checked_text(value.value(), "DNS RDATA value"))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((values, false))
}

pub(super) fn map_record(
    ctx: &Context,
    zone: &str,
    identity: &str,
    record: &ResourceRecordSet,
) -> Result<(Resource, Vec<Edge>), AwsScanError> {
    let (values, omitted) = safe_record_values(record)?;
    let alias = record.alias_target().map(|alias| json!({"dns_name":alias.dns_name(),"hosted_zone_id":alias.hosted_zone_id(),"evaluate_target_health":alias.evaluate_target_health()}));
    let resource = ctx.resource(RECORD, identity.to_owned(), Some(checked_text(record.name(), "record name")?), json!({
        "zone_id":zone,"name":record.name(),"type":record.r#type().as_str(),"set_identifier":record.set_identifier(),
        "ttl":record.ttl(),"records":values,"rdata_omitted":omitted,"rdata_count":record.resource_records().len(),
        "alias_target":alias,"weight":record.weight(),"region":record.region().map(|v| v.as_str()),
        "failover":record.failover().map(|v| v.as_str()),"multi_value_answer":record.multi_value_answer(),
        "health_check_id":record.health_check_id(),"traffic_policy_instance_id":record.traffic_policy_instance_id(),
        "geo_location":record.geo_location().map(|v| json!({"continent":v.continent_code(),"country":v.country_code(),"subdivision":v.subdivision_code()})),
        "cidr_routing":record.cidr_routing_config().map(|v| json!({"collection_id":v.collection_id(),"location_name":v.location_name()})),
        "identity_model":"zone_name_type_set_identifier","live_dns_resolution_performed":false
    }), BTreeMap::new())?;
    let mut edges = vec![reference(
        ctx,
        &resource,
        &zone_arn(ctx, zone),
        "IN_HOSTED_ZONE",
        json!({"field":"HostedZoneId"}),
    )?];
    if let Some(alias) = record.alias_target() {
        edges.push(reference(ctx, &resource, &dns_reference(alias.dns_name())?, "ALIASES_DNS_NAME", json!({"field":"AliasTarget.DNSName","alias_hosted_zone_id":alias.hosted_zone_id(),"evaluate_target_health":alias.evaluate_target_health()}))?);
    }
    if let Some(health) = record.health_check_id() {
        checked_text(health, "health check identifier")?;
        edges.push(reference(
            ctx,
            &resource,
            &format!("arn:{}:route53:::healthcheck/{health}", ctx.partition),
            "USES_HEALTH_CHECK",
            json!({"field":"HealthCheckId"}),
        )?);
    }
    for value in &values {
        if let Some(target) = record_target(ctx, zone, record.r#type().as_str(), value)? {
            edges.push(reference(ctx, &resource, &target, "DNS_TARGET", json!({"field":"ResourceRecords","record_type":record.r#type().as_str(),"native_resource_resolution":"unresolved"}))?);
        }
    }
    resource.validate()?;
    Ok((resource, edge_set(edges)?))
}

pub(super) fn record_target(
    ctx: &Context,
    zone: &str,
    kind: &str,
    value: &str,
) -> Result<Option<String>, AwsScanError> {
    match kind {
        "A" | "AAAA" => {
            let address: std::net::IpAddr = value
                .parse()
                .map_err(|_| AwsScanError::InvalidData("invalid DNS IP address".into()))?;
            if (kind == "A") != address.is_ipv4() {
                return Err(AwsScanError::InvalidData(
                    "DNS address family does not match record type".into(),
                ));
            }
            Ok(Some(format!(
                "dns-ip:{}:{}:{zone}:{address}",
                ctx.partition, ctx.account
            )))
        }
        "CNAME" | "NS" | "PTR" => dns_reference(value).map(Some),
        "MX" => value
            .split_whitespace()
            .nth(1)
            .filter(|value| *value != ".")
            .map(dns_reference)
            .transpose(),
        "SRV" => value
            .split_whitespace()
            .nth(3)
            .filter(|value| *value != ".")
            .map(dns_reference)
            .transpose(),
        _ => Ok(None),
    }
}
