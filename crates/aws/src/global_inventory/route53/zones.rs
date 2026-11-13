//! zones responsibilities for route53.
use super::*;

#[tracing::instrument(skip_all)]
pub(super) async fn collect_zone(
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
pub(super) async fn zone_tags(
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

pub(super) fn zone_id(value: &str) -> Result<String, AwsScanError> {
    let id = value.strip_prefix("/hostedzone/").unwrap_or(value);
    if id.is_empty() || id.len() > 32 || !id.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
        return Err(AwsScanError::InvalidData(
            "invalid hosted zone identifier".into(),
        ));
    }
    Ok(id.to_owned())
}

pub(super) fn zone_arn(ctx: &Context, id: &str) -> String {
    format!("arn:{}:route53:::hostedzone/{id}", ctx.partition)
}

pub(super) fn map_zone(
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
