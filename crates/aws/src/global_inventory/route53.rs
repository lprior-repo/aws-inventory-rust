//! Hosted zones and record sets. Record cursors preserve name, type and routing
//! identifier together. TXT/SPF and unknown RDATA are omitted, not interpreted.

use std::collections::BTreeMap;

use aws_sdk_route53::{
    Client,
    operation::{
        get_hosted_zone::GetHostedZoneOutput,
        list_resource_record_sets::ListResourceRecordSetsOutput,
    },
    types::{HostedZone, ResourceRecordSet, RrType, TagResourceType},
};
use inventory_core::{Edge, Resource, Source};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{
    CHILD_CONCURRENCY, SeenIds, checked_text, components, digest_identity, dns_reference, edge_set,
    ensure_active, marker, observed, reference, section, single,
};
use crate::{
    AwsScanError,
    common::{
        Context, PageBody, collect_bounded, error_category, paginate, require_budget, required,
        sdk_error,
    },
};

const ZONE: &str = "aws_route53_zone";
const RECORD: &str = "aws_route53_record";
const MAX_RECORD_VALUES: usize = 512;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RecordCursor {
    name: String,
    record_type: String,
    identifier: Option<String>,
}

impl RecordCursor {
    fn validate(&self) -> Result<(), AwsScanError> {
        checked_text(&self.name, "record cursor name")?;
        checked_text(&self.record_type, "record cursor type")?;
        if self.name.len() > 1_024
            || self.record_type.len() > 32
            || self.identifier.as_ref().is_some_and(|id| {
                id.is_empty()
                    || id.len() > 512
                    || id.chars().take(129).count() > 128
                    || id.contains('\0')
            })
        {
            return Err(AwsScanError::InvalidData(
                "record cursor exceeds field limits".into(),
            ));
        }
        Ok(())
    }
    fn decode(token: Option<&str>) -> Result<Option<Self>, AwsScanError> {
        let Some(token) = token else {
            return Ok(None);
        };
        if token.len() > 2_048 {
            return Err(AwsScanError::InvalidData(
                "record cursor exceeds 2 KiB".into(),
            ));
        }
        let cursor: Self = serde_json::from_str(token)
            .map_err(|_| AwsScanError::InvalidData("invalid record cursor JSON".into()))?;
        cursor.validate()?;
        Ok(Some(cursor))
    }
}

fn next_record_cursor(
    output: &ListResourceRecordSetsOutput,
) -> Result<Option<String>, AwsScanError> {
    if !output.is_truncated() {
        return Ok(None);
    }
    let cursor = RecordCursor {
        name: required(output.next_record_name(), "next record name")?,
        record_type: output
            .next_record_type()
            .map(|v| v.as_str().to_owned())
            .ok_or_else(|| AwsScanError::InvalidData("truncated records lack next type".into()))?,
        identifier: output.next_record_identifier().map(str::to_owned),
    };
    cursor.validate()?;
    serde_json::to_string(&cursor)
        .map(Some)
        .map_err(|_| AwsScanError::InvalidData("record cursor serialization failed".into()))
}

#[tracing::instrument(skip_all)]
pub(super) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    ensure_active(ctx)?;
    let client = Client::new(&ctx.config);
    let seen = SeenIds::default();
    paginate(
        ctx,
        ctx.scope("aws_route53_zone_list", None, Source::Direct),
        |token, budget| {
            let seen = &seen;
            let client = &client;
            async move {
                ensure_active(ctx)?;
                let output = ctx
                    .execute("route53", "read", || async {
                        client
                            .list_hosted_zones()
                            .set_marker(token.clone())
                            .max_items(ctx.page_size.clamp(1, 100))
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                let count =
                    seen.record(output.hosted_zones().iter().map(HostedZone::id), budget)?;
                let next_token = marker(output.is_truncated(), output.next_marker())?;
                collect_bounded(output.hosted_zones(), CHILD_CONCURRENCY, |zone| {
                    collect_zone(ctx, client, zone)
                })
                .await?;
                Ok(PageBody {
                    next_token,
                    listed_count: count,
                    ..PageBody::default()
                })
            }
        },
    )
    .await?;
    ensure_active(ctx)
}

#[tracing::instrument(skip_all)]
async fn collect_zone(
    ctx: &Context,
    client: &Client,
    listed: &HostedZone,
) -> Result<(), AwsScanError> {
    ensure_active(ctx)?;
    let id = match zone_id(listed.id()) {
        Ok(id) => id,
        Err(error) => {
            return observed(
                ctx,
                ctx.scope(ZONE, Some(listed.id()), Source::Direct),
                Err(error),
                None,
            )
            .await;
        }
    };
    let scope = ctx.scope(ZONE, Some(&id), Source::Direct);
    let fallback = match map_zone(ctx, listed, None, None) {
        Ok(body) => body,
        Err(error) => return observed(ctx, scope, Err(error), None).await,
    };
    let mapped = async {
        let output = ctx
            .execute("route53", "read", || async {
                client
                    .get_hosted_zone()
                    .id(&id)
                    .send()
                    .await
                    .map_err(sdk_error)
            })
            .await?;
        let zone = output
            .hosted_zone()
            .ok_or_else(|| AwsScanError::InvalidData("GetHostedZone lacks hosted zone".into()))?;
        if zone_id(zone.id())? != id {
            return Err(AwsScanError::InvalidData(
                "GetHostedZone identity changed".into(),
            ));
        }
        let tags = zone_tags(ctx, client, &id).await?;
        map_zone(ctx, zone, Some(&output), tags)
    }
    .await;
    observed(ctx, scope, mapped, Some(fallback)).await?;
    records(ctx, client, &id).await
}

#[tracing::instrument(skip_all)]
async fn zone_tags(
    ctx: &Context,
    client: &Client,
    id: &str,
) -> Result<Option<BTreeMap<String, String>>, AwsScanError> {
    let (_, value) = section(ctx, id, "aws_route53_zone_tags", || async {
        let output = ctx
            .execute("route53", "read", || async {
                client
                    .list_tags_for_resource()
                    .resource_type(TagResourceType::Hostedzone)
                    .resource_id(id)
                    .send()
                    .await
                    .map_err(sdk_error)
            })
            .await?;
        let tags = output
            .resource_tag_set()
            .ok_or_else(|| AwsScanError::InvalidData("ListTagsForResource lacks tag set".into()))?;
        if tags
            .resource_id()
            .is_some_and(|value| value != id && value != format!("/hostedzone/{id}"))
        {
            return Err(AwsScanError::InvalidData(
                "hosted zone tag response changed identity".into(),
            ));
        }
        components(tags.tags().len(), "hosted zone tags")?;
        let mapped = tags
            .tags()
            .iter()
            .map(|tag| {
                Ok((
                    required(tag.key(), "zone tag key")?,
                    tag.value().unwrap_or("").to_owned(),
                ))
            })
            .collect::<Result<BTreeMap<_, _>, AwsScanError>>()?;
        Ok(json!(mapped))
    })
    .await?;
    value
        .map(|value| {
            serde_json::from_value(value)
                .map_err(|_| AwsScanError::InvalidData("invalid mapped zone tags".into()))
        })
        .transpose()
}

#[tracing::instrument(skip_all)]
async fn records(ctx: &Context, client: &Client, id: &str) -> Result<(), AwsScanError> {
    let seen = SeenIds::default();
    paginate(
        ctx,
        ctx.scope("aws_route53_record_list", Some(id), Source::Direct),
        |token, budget| {
            let seen = &seen;
            async move {
                ensure_active(ctx)?;
                let cursor = RecordCursor::decode(token.as_deref())?;
                let output = ctx
                    .execute("route53", "read", || async {
                        client
                            .list_resource_record_sets()
                            .hosted_zone_id(id)
                            .max_items(ctx.page_size.clamp(1, 300))
                            .set_start_record_name(cursor.as_ref().map(|v| v.name.clone()))
                            .set_start_record_type(
                                cursor
                                    .as_ref()
                                    .map(|v| RrType::from(v.record_type.as_str())),
                            )
                            .set_start_record_identifier(
                                cursor.as_ref().and_then(|v| v.identifier.clone()),
                            )
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                require_budget(output.resource_record_sets().len(), budget)?;
                let identities = output
                    .resource_record_sets()
                    .iter()
                    .map(|record| record_identity(ctx, id, record))
                    .collect::<Result<Vec<_>, _>>()?;
                let count = seen.record(identities.iter().map(String::as_str), budget)?;
                let mapped = collect_bounded(
                    output.resource_record_sets().iter().zip(identities.iter()),
                    CHILD_CONCURRENCY,
                    |(record, identity)| map_record_scoped(ctx, id, identity, record),
                )
                .await?;
                let mut body = PageBody {
                    next_token: next_record_cursor(&output)?,
                    listed_count: count,
                    ..PageBody::default()
                };
                for (resource, edges) in mapped.into_iter().flatten() {
                    body.resources.push(resource);
                    body.edges.extend(edges);
                }
                Ok(body)
            }
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
async fn map_record_scoped(
    ctx: &Context,
    zone: &str,
    identity: &str,
    record: &ResourceRecordSet,
) -> Result<Option<(Resource, Vec<Edge>)>, AwsScanError> {
    ensure_active(ctx)?;
    match map_record(ctx, zone, identity, record) {
        Ok(mapped) => Ok(Some(mapped)),
        Err(error) => {
            ctx.failure(
                &ctx.scope(RECORD, Some(identity), Source::Direct),
                error_category(&error),
                error.to_string(),
            )
            .await?;
            Ok(None)
        }
    }
}

fn zone_id(value: &str) -> Result<String, AwsScanError> {
    let id = value.strip_prefix("/hostedzone/").unwrap_or(value);
    if id.is_empty() || id.len() > 32 || !id.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
        return Err(AwsScanError::InvalidData(
            "invalid hosted zone identifier".into(),
        ));
    }
    Ok(id.to_owned())
}

fn zone_arn(ctx: &Context, id: &str) -> String {
    format!("arn:{}:route53:::hostedzone/{id}", ctx.partition)
}

fn map_zone(
    ctx: &Context,
    zone: &HostedZone,
    details: Option<&GetHostedZoneOutput>,
    tags: Option<BTreeMap<String, String>>,
) -> Result<PageBody, AwsScanError> {
    let id = zone_id(zone.id())?;
    let vpcs = details.map(GetHostedZoneOutput::vpcs).unwrap_or(&[]);
    components(vpcs.len(), "hosted zone VPCs")?;
    let nameservers = details
        .and_then(|v| v.delegation_set())
        .map(|v| v.name_servers())
        .unwrap_or(&[]);
    components(nameservers.len(), "hosted zone nameservers")?;
    let resource = ctx.resource(ZONE, zone_arn(ctx, &id), Some(checked_text(zone.name(), "hosted zone name")?), json!({
        "zone_id":id,"name":zone.name(),"private_zone":zone.config().map(|v| v.private_zone()),
        "comment":zone.config().and_then(|v| v.comment()),"resource_record_set_count":zone.resource_record_set_count(),
        "name_servers":nameservers,"linked_service_principal":zone.linked_service().and_then(|v| v.service_principal()),
        "vpcs":vpcs.iter().map(|v| json!({"vpc_id":v.vpc_id(),"vpc_region":v.vpc_region().map(|r| r.as_str()),"owner_resolution":"assumed_scanning_account"})).collect::<Vec<_>>(),
        "details_available":details.is_some(),"tags_available":tags.is_some(),
        "observation_detail":if details.is_some() {"get_hosted_zone"} else {"list_only"}
    }), tags.unwrap_or_default())?;
    let mut edges = Vec::new();
    for vpc in vpcs {
        let id = required(vpc.vpc_id(), "zone VPC identifier")?;
        let region = vpc
            .vpc_region()
            .map(|r| r.as_str())
            .ok_or_else(|| AwsScanError::InvalidData("zone VPC lacks region".into()))?;
        crate::validate_region(region)?;
        if !id.starts_with("vpc-") {
            return Err(AwsScanError::InvalidData(
                "invalid hosted zone VPC reference".into(),
            ));
        }
        let target = format!(
            "arn:{}:ec2:{region}:{}:vpc/{id}",
            ctx.partition, ctx.account
        );
        edges.push(reference(ctx, &resource, &target, "ASSOCIATED_WITH_VPC", json!({"field":"VPCs","target_region":region,"target_resource_id":id,"owner_resolution":"assumed_scanning_account"}))?);
    }
    resource.validate()?;
    Ok(single(resource, edge_set(edges)?))
}

fn record_identity(
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

fn safe_record_values(record: &ResourceRecordSet) -> Result<(Vec<String>, bool), AwsScanError> {
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

fn map_record(
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

fn record_target(
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

#[cfg(test)]
mod tests {
    use super::super::tests::context;
    use super::*;

    #[test]
    fn record_cursor_round_trip_keeps_routing_identifier() -> Result<(), Box<dyn std::error::Error>>
    {
        let output = ListResourceRecordSetsOutput::builder()
            .set_resource_record_sets(Some(Vec::new()))
            .is_truncated(true)
            .max_items(100)
            .next_record_name("weighted.example.")
            .next_record_type(RrType::A)
            .next_record_identifier("blue")
            .build()?;
        let token = next_record_cursor(&output)?.ok_or("missing token")?;
        let cursor = RecordCursor::decode(Some(&token))?.ok_or("missing cursor")?;
        assert_eq!(cursor.identifier.as_deref(), Some("blue"));
        assert_eq!(cursor.name, "weighted.example.");
        assert_eq!(cursor.record_type, "A");
        assert_eq!(RecordCursor::decode(None)?, None);
        assert!(
            RecordCursor::decode(Some(
                r#"{"name":"a","record_type":"A","identifier":null,"injected":true}"#
            ))
            .is_err()
        );
        let unicode = RecordCursor {
            name: "weighted.example.".into(),
            record_type: "A".into(),
            identifier: Some("東".repeat(128)),
        };
        unicode.validate()?;
        assert_eq!(
            RecordCursor::decode(Some(&serde_json::to_string(&unicode)?))?,
            Some(unicode.clone())
        );
        let oversized = RecordCursor {
            identifier: Some("東".repeat(129)),
            ..unicode
        };
        assert!(oversized.validate().is_err());
        Ok(())
    }

    #[test]
    fn truncated_record_page_without_name_or_type_is_failure()
    -> Result<(), Box<dyn std::error::Error>> {
        let output = ListResourceRecordSetsOutput::builder()
            .set_resource_record_sets(Some(Vec::new()))
            .is_truncated(true)
            .max_items(1)
            .next_record_name("example.")
            .build()?;
        assert!(next_record_cursor(&output).is_err());
        let done = ListResourceRecordSetsOutput::builder()
            .set_resource_record_sets(Some(Vec::new()))
            .is_truncated(false)
            .max_items(1)
            .next_record_name("stale.")
            .build()?;
        assert_eq!(next_record_cursor(&done)?, None);
        Ok(())
    }

    #[test]
    fn weighted_records_have_distinct_stable_identities() -> Result<(), Box<dyn std::error::Error>>
    {
        let (ctx, _rx) = context()?;
        let blue = ResourceRecordSet::builder()
            .name("EXAMPLE.com.")
            .r#type(RrType::A)
            .set_identifier("blue")
            .build()?;
        let green = ResourceRecordSet::builder()
            .name("example.com.")
            .r#type(RrType::A)
            .set_identifier("green")
            .build()?;
        assert_ne!(
            record_identity(&ctx, "Z123", &blue)?,
            record_identity(&ctx, "Z123", &green)?
        );
        let same = ResourceRecordSet::builder()
            .name("example.com.")
            .r#type(RrType::A)
            .set_identifier("blue")
            .build()?;
        assert_eq!(
            record_identity(&ctx, "Z123", &blue)?,
            record_identity(&ctx, "Z123", &same)?
        );
        let without_dot = ResourceRecordSet::builder()
            .name("example.com")
            .r#type(RrType::A)
            .set_identifier("blue")
            .build()?;
        assert_eq!(
            record_identity(&ctx, "Z123", &blue)?,
            record_identity(&ctx, "Z123", &without_dot)?
        );
        let simple = ResourceRecordSet::builder()
            .name("example.com.")
            .r#type(RrType::A)
            .build()?;
        let sentinel = ResourceRecordSet::builder()
            .name("example.com.")
            .r#type(RrType::A)
            .set_identifier("<simple>")
            .build()?;
        assert_ne!(
            record_identity(&ctx, "Z123", &simple)?,
            record_identity(&ctx, "Z123", &sentinel)?
        );
        Ok(())
    }

    #[tokio::test]
    async fn malformed_record_is_scoped_and_valid_sibling_survives()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, mut rx) = context()?;
        let bad = ResourceRecordSet::builder()
            .name("bad.example.")
            .r#type(RrType::A)
            .resource_records(
                aws_sdk_route53::types::ResourceRecord::builder()
                    .value("::1")
                    .build()?,
            )
            .build()?;
        let good = ResourceRecordSet::builder()
            .name("good.example.")
            .r#type(RrType::A)
            .resource_records(
                aws_sdk_route53::types::ResourceRecord::builder()
                    .value("192.0.2.1")
                    .build()?,
            )
            .build()?;
        let bad_id = record_identity(&ctx, "Z123", &bad)?;
        assert!(
            map_record_scoped(&ctx, "Z123", &bad_id, &bad)
                .await?
                .is_none()
        );
        let good_id = record_identity(&ctx, "Z123", &good)?;
        assert!(
            map_record_scoped(&ctx, "Z123", &good_id, &good)
                .await?
                .is_some()
        );
        assert!(
            matches!(rx.try_recv()?, inventory_core::CollectionEvent::Failure(failure)
            if failure.scope.parent_id.as_deref() == Some(bad_id.as_str()))
        );
        assert!(rx.try_recv().is_err());
        Ok(())
    }

    #[test]
    fn txt_values_are_omitted_with_explicit_count() -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, _rx) = context()?;
        let record = ResourceRecordSet::builder()
            .name("token.example.")
            .r#type(RrType::Txt)
            .resource_records(
                aws_sdk_route53::types::ResourceRecord::builder()
                    .value("secret-token-NEVER_PERSIST")
                    .build()?,
            )
            .build()?;
        let id = record_identity(&ctx, "Z123", &record)?;
        let (resource, edges) = map_record(&ctx, "Z123", &id, &record)?;
        assert!(!serde_json::to_string(&resource)?.contains("NEVER_PERSIST"));
        assert_eq!(resource.properties.get("rdata_omitted"), Some(&json!(true)));
        assert_eq!(resource.properties.get("rdata_count"), Some(&json!(1)));
        assert_eq!(edges.len(), 1);
        Ok(())
    }

    #[test]
    fn aliases_stay_dns_references_without_fabricated_load_balancer_identity()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, _rx) = context()?;
        let record = ResourceRecordSet::builder()
            .name("service.example.")
            .r#type(RrType::A)
            .alias_target(
                aws_sdk_route53::types::AliasTarget::builder()
                    .hosted_zone_id("ZALB")
                    .dns_name("dualstack.example.us-east-1.elb.amazonaws.com.")
                    .evaluate_target_health(true)
                    .build()?,
            )
            .build()?;
        let id = record_identity(&ctx, "Z123", &record)?;
        let (_, edges) = map_record(&ctx, "Z123", &id, &record)?;
        assert_eq!(edges.len(), 2);
        assert!(
            edges
                .iter()
                .any(|edge| edge.to_id == "dns:dualstack.example.us-east-1.elb.amazonaws.com")
        );
        assert!(
            edges
                .iter()
                .all(|edge| !edge.to_id.contains(":loadbalancer/"))
        );
        Ok(())
    }

    #[test]
    fn bad_address_family_is_rejected_and_zone_prefix_normalizes()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, _rx) = context()?;
        assert_eq!(zone_id("/hostedzone/Z123")?, "Z123");
        assert!(zone_id("/hostedzone/").is_err());
        assert!(record_target(&ctx, "Z123", "A", "::1").is_err());
        assert!(record_target(&ctx, "Z123", "AAAA", "127.0.0.1").is_err());
        assert!(record_target(&ctx, "Z123", "AAAA", "2001:db8::1")?.is_some());
        Ok(())
    }

    #[test]
    fn null_mx_retains_rdata_without_inventing_a_dns_destination()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, _rx) = context()?;
        // RFC 7505 section 3: a sole MX record "0 ." declares no mail service.
        let record = ResourceRecordSet::builder()
            .name("nomail.example.")
            .r#type(RrType::Mx)
            .resource_records(
                aws_sdk_route53::types::ResourceRecord::builder()
                    .value("0 .")
                    .build()?,
            )
            .build()?;
        let identity = record_identity(&ctx, "Z123", &record)?;
        let (resource, edges) = map_record(&ctx, "Z123", &identity, &record)?;
        assert_eq!(resource.properties.get("records"), Some(&json!(["0 ."])));
        assert_eq!(
            resource.properties.get("rdata_omitted"),
            Some(&json!(false))
        );
        assert_eq!(edges.len(), 1);
        assert_eq!(
            edges.first().map(|edge| edge.relation.as_str()),
            Some("IN_HOSTED_ZONE")
        );
        Ok(())
    }
}
