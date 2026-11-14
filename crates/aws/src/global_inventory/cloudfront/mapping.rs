use super::*;

pub(super) fn map_distribution(
    ctx: &Context,
    value: &DistributionSummary,
    tags: Option<BTreeMap<String, String>>,
) -> Result<PageBody, AwsScanError> {
    let arn = own_arn(ctx, value.arn(), "cloudfront", "distribution")?;
    if !arn.ends_with(&format!("/{}", value.id())) {
        return Err(AwsScanError::InvalidData(
            "distribution ID and ARN disagree".into(),
        ));
    }
    let origins = value
        .origins()
        .ok_or_else(|| AwsScanError::InvalidData("distribution lacks origins".into()))?;
    let default = value
        .default_cache_behavior()
        .ok_or_else(|| AwsScanError::InvalidData("distribution lacks default behavior".into()))?;
    let behaviors = value.cache_behaviors().map(|v| v.items()).unwrap_or(&[]);
    let aliases = value.aliases().map(|v| v.items()).unwrap_or(&[]);
    let groups = value.origin_groups().map(|v| v.items()).unwrap_or(&[]);
    let mut budget = 0usize;
    for count in [
        origins.items().len(),
        behaviors.len(),
        aliases.len(),
        groups.len(),
    ] {
        consume(&mut budget, count)?;
    }
    let mut origin_ids = BTreeSet::new();
    for origin in origins.items() {
        if !origin_ids.insert(checked_text(origin.id(), "CloudFront origin identifier")?) {
            return Err(AwsScanError::InvalidData(
                "duplicate distribution origin identifier".into(),
            ));
        }
    }
    for group in groups {
        if !origin_ids.insert(checked_text(
            group.id(),
            "CloudFront origin group identifier",
        )?) {
            return Err(AwsScanError::InvalidData(
                "duplicate distribution origin/group identifier".into(),
            ));
        }
        consume(
            &mut budget,
            group.members().map(|v| v.items().len()).unwrap_or(0),
        )?;
        consume(
            &mut budget,
            group
                .failover_criteria()
                .and_then(|v| v.status_codes())
                .map(|v| v.items().len())
                .unwrap_or(0),
        )?;
    }
    let default = Behavior::default(default);
    default.validate(&mut budget)?;
    let behaviors: Vec<Behavior<'_>> = behaviors.iter().map(Behavior::ordered).collect();
    for behavior in &behaviors {
        behavior.validate(&mut budget)?;
    }
    let mapped_origins = origins
        .items()
        .iter()
        .map(map_origin)
        .collect::<Result<Vec<_>, _>>()?;
    let resource = distribution_resource(
        ctx,
        value,
        arn,
        mapped_origins,
        default.properties(&origin_ids),
        behaviors
            .iter()
            .map(|v| v.properties(&origin_ids))
            .collect(),
        tags,
    )?;
    let mut edges = distribution_edges(ctx, &resource, value)?;
    for behavior in std::iter::once(&default).chain(behaviors.iter()) {
        edges.extend(behavior_edges(ctx, &resource, behavior)?);
    }
    resource.validate()?;
    Ok(single(resource, edge_set(edges)?))
}

pub(super) fn distribution_resource(
    ctx: &Context,
    value: &DistributionSummary,
    arn: String,
    origins: Vec<Value>,
    default: Value,
    behaviors: Vec<Value>,
    tags: Option<BTreeMap<String, String>>,
) -> Result<Resource, AwsScanError> {
    let certificate = value.viewer_certificate().map(|v| json!({"acm_certificate_arn":v.acm_certificate_arn(),
        "iam_certificate_id":v.iam_certificate_id(),"default_certificate":v.cloud_front_default_certificate(),
        "ssl_support_method":v.ssl_support_method().map(|v| v.as_str()),"minimum_protocol_version":v.minimum_protocol_version().map(|v| v.as_str())}));
    let groups: Vec<Value> = value.origin_groups().map(|v| v.items()).unwrap_or(&[]).iter().map(|group| json!({
        "id":group.id(),"member_origin_ids":group.members().map(|v| v.items().iter().map(|v| v.origin_id()).collect::<Vec<_>>()).unwrap_or_default(),
        "failover_status_codes":group.failover_criteria().and_then(|v| v.status_codes()).map(|v| v.items()),
        "selection_criteria":group.selection_criteria().map(|v| v.as_str())
    })).collect();
    ctx.resource(KIND, arn, Some(checked_text(value.id(), "distribution identifier")?), json!({
        "distribution_id":value.id(),"domain_name":value.domain_name(),"status":value.status(),"enabled":value.enabled(),
        "staging":value.staging(),"http_version":value.http_version().as_str(),"is_ipv6_enabled":value.is_ipv6_enabled(),
        "price_class":value.price_class().as_str(),"last_modified_time":value.last_modified_time().to_millis().map_err(|_| AwsScanError::InvalidData("distribution timestamp out of range".into()))?,
        "aliases":value.aliases().map(|v| v.items()).unwrap_or(&[]),"origins":origins,"origin_groups":groups,
        "default_cache_behavior":default,"cache_behaviors":behaviors,"viewer_certificate":certificate,
        "web_acl_id":(!value.web_acl_id().is_empty()).then_some(value.web_acl_id()),
        "connection_mode":value.connection_mode().map(|v| v.as_str()),"tags_available":tags.is_some(),
        "origin_header_values_omitted":true,"observation_detail":"list_distributions_summary"
    }), tags.unwrap_or_default())
}

pub(super) fn map_origin(origin: &Origin) -> Result<Value, AwsScanError> {
    checked_text(origin.domain_name(), "CloudFront origin DNS name")?;
    components(
        origin
            .custom_headers()
            .map(|v| v.items().len())
            .unwrap_or(0),
        "origin custom headers",
    )?;
    let custom = origin.custom_origin_config().map(|v| json!({"http_port":v.http_port(),"https_port":v.https_port(),
        "protocol_policy":v.origin_protocol_policy().as_str(),"read_timeout":v.origin_read_timeout(),"keepalive_timeout":v.origin_keepalive_timeout()}));
    Ok(
        json!({"id":origin.id(),"domain_name":origin.domain_name(),"path_configured":origin.origin_path().is_some_and(|v| !v.is_empty()),
        "origin_path_omitted":true,"custom_header_count":origin.custom_headers().map(|v| v.items().len()).unwrap_or(0),
        "custom_header_values_omitted":true,"s3_origin":origin.s3_origin_config().is_some(),
        "origin_access_identity":origin.s3_origin_config().map(|v| v.origin_access_identity()),
        "origin_access_control_id":origin.origin_access_control_id(),"custom_origin":custom,
        "vpc_origin_id":origin.vpc_origin_config().map(|v| v.vpc_origin_id()),
        "vpc_origin_owner":origin.vpc_origin_config().and_then(|v| v.owner_account_id()),
        "connection_attempts":origin.connection_attempts(),"connection_timeout":origin.connection_timeout(),
        "response_completion_timeout":origin.response_completion_timeout(),
        "origin_shield":origin.origin_shield().map(|v| json!({"enabled":v.enabled(),"region":v.origin_shield_region()}))}),
    )
}
