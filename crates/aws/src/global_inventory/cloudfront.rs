//! CloudFront conventional distribution metadata. Origin header values and
//! origin paths are excluded; routing and identity references remain explicit.

use std::collections::{BTreeMap, BTreeSet};

use aws_sdk_cloudfront::{
    Client,
    types::{
        CacheBehavior, DefaultCacheBehavior, DistributionSummary, FunctionAssociation,
        LambdaFunctionAssociation, Method, Origin,
    },
};
use inventory_core::{Edge, Resource, Source};
use serde_json::{Value, json};

use super::{
    CHILD_CONCURRENCY, MAX_COMPONENTS, SeenIds, checked_text, components, dns_reference, edge_set,
    ensure_active, marker, observed, own_arn, reference, section, single,
};
use crate::{
    AwsScanError,
    common::{Context, PageBody, collect_bounded, paginate, sdk_error},
};

const KIND: &str = "aws_cloudfront_distribution";

#[tracing::instrument(skip_all)]
pub(super) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    ensure_active(ctx)?;
    if ctx.partition != "aws" {
        return ctx
            .failure(
                &ctx.scope("aws_cloudfront_distribution_list", None, Source::Direct),
                "unsupported_partition",
                "CloudFront collector has a reviewed endpoint policy only for the aws partition",
            )
            .await;
    }
    let client = Client::new(&ctx.config);
    let seen = SeenIds::default();
    paginate(
        ctx,
        ctx.scope("aws_cloudfront_distribution_list", None, Source::Direct),
        |token, budget| {
            let client = &client;
            let seen = &seen;
            async move {
                ensure_active(ctx)?;
                let output = ctx
                    .execute("cloudfront", "read", || async {
                        client
                            .list_distributions()
                            .set_marker(token.clone())
                            .max_items(ctx.page_size.clamp(1, 100))
                            .send()
                            .await
                            .map_err(sdk_error)
                    })
                    .await?;
                let list = output.distribution_list().ok_or_else(|| {
                    AwsScanError::InvalidData("ListDistributions lacks list".into())
                })?;
                let count =
                    seen.record(list.items().iter().map(DistributionSummary::arn), budget)?;
                let next_token = marker(list.is_truncated(), list.next_marker())?;
                collect_bounded(list.items(), CHILD_CONCURRENCY, |value| {
                    collect_distribution(ctx, client, value)
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
async fn collect_distribution(
    ctx: &Context,
    client: &Client,
    value: &DistributionSummary,
) -> Result<(), AwsScanError> {
    ensure_active(ctx)?;
    let scope = ctx.scope(KIND, Some(value.arn()), Source::Direct);
    let mut mapped = match map_distribution(ctx, value, None) {
        Ok(body) => body,
        Err(error) => return observed(ctx, scope, Err(error), None).await,
    };
    let tags = distribution_tags(ctx, client, value.arn()).await?;
    if let Some(resource) = mapped.resources.first_mut() {
        if let Some(properties) = resource.properties.as_object_mut() {
            properties.insert("tags_available".to_owned(), json!(tags.is_some()));
        }
        resource.tags = tags.unwrap_or_default();
    }
    observed(ctx, scope, Ok(mapped), None).await
}

#[tracing::instrument(skip_all)]
async fn distribution_tags(
    ctx: &Context,
    client: &Client,
    arn: &str,
) -> Result<Option<BTreeMap<String, String>>, AwsScanError> {
    let (_, value) = section(ctx, arn, "aws_cloudfront_distribution_tags", || async {
        let output = ctx
            .execute("cloudfront", "read", || async {
                client
                    .list_tags_for_resource()
                    .resource(arn)
                    .send()
                    .await
                    .map_err(sdk_error)
            })
            .await?;
        let tags = output.tags().ok_or_else(|| {
            AwsScanError::InvalidData("CloudFront tag response lacks tags".into())
        })?;
        components(tags.items().len(), "CloudFront tags")?;
        let mapped = tags
            .items()
            .iter()
            .map(|tag| {
                Ok((
                    checked_text(tag.key(), "CloudFront tag key")?,
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
                .map_err(|_| AwsScanError::InvalidData("invalid mapped CloudFront tags".into()))
        })
        .transpose()
}

struct Behavior<'a> {
    path: &'a str,
    target: &'a str,
    viewer_protocol: &'a str,
    cache_policy: Option<&'a str>,
    origin_policy: Option<&'a str>,
    response_policy: Option<&'a str>,
    realtime_log: Option<&'a str>,
    compress: Option<bool>,
    allowed: &'a [Method],
    lambdas: &'a [LambdaFunctionAssociation],
    functions: &'a [FunctionAssociation],
    minimum_ttl: Option<i64>,
    default_ttl: Option<i64>,
    maximum_ttl: Option<i64>,
}

impl<'a> Behavior<'a> {
    #[expect(
        deprecated,
        reason = "Inventory retains legacy behavior TTL fields for distributions that do not use cache policies"
    )]
    fn default(value: &'a DefaultCacheBehavior) -> Self {
        Self {
            path: "*",
            target: value.target_origin_id(),
            viewer_protocol: value.viewer_protocol_policy().as_str(),
            cache_policy: value.cache_policy_id(),
            origin_policy: value.origin_request_policy_id(),
            response_policy: value.response_headers_policy_id(),
            realtime_log: value.realtime_log_config_arn(),
            compress: value.compress(),
            allowed: value.allowed_methods().map(|v| v.items()).unwrap_or(&[]),
            lambdas: value
                .lambda_function_associations()
                .map(|v| v.items())
                .unwrap_or(&[]),
            functions: value
                .function_associations()
                .map(|v| v.items())
                .unwrap_or(&[]),
            minimum_ttl: value.min_ttl(),
            default_ttl: value.default_ttl(),
            maximum_ttl: value.max_ttl(),
        }
    }
    #[expect(
        deprecated,
        reason = "Inventory retains legacy behavior TTL fields for distributions that do not use cache policies"
    )]
    fn ordered(value: &'a CacheBehavior) -> Self {
        Self {
            path: value.path_pattern(),
            target: value.target_origin_id(),
            viewer_protocol: value.viewer_protocol_policy().as_str(),
            cache_policy: value.cache_policy_id(),
            origin_policy: value.origin_request_policy_id(),
            response_policy: value.response_headers_policy_id(),
            realtime_log: value.realtime_log_config_arn(),
            compress: value.compress(),
            allowed: value.allowed_methods().map(|v| v.items()).unwrap_or(&[]),
            lambdas: value
                .lambda_function_associations()
                .map(|v| v.items())
                .unwrap_or(&[]),
            functions: value
                .function_associations()
                .map(|v| v.items())
                .unwrap_or(&[]),
            minimum_ttl: value.min_ttl(),
            default_ttl: value.default_ttl(),
            maximum_ttl: value.max_ttl(),
        }
    }
    fn validate(&self, budget: &mut usize) -> Result<(), AwsScanError> {
        consume(budget, self.allowed.len())?;
        consume(budget, self.lambdas.len())?;
        consume(budget, self.functions.len())?;
        checked_text(self.target, "CloudFront behavior origin identifier")?;
        checked_text(self.path, "CloudFront behavior path")?;
        Ok(())
    }
    fn properties(&self, origins: &BTreeSet<String>) -> Value {
        json!({"path_pattern":self.path,"target_origin_id":self.target,"target_in_configuration":origins.contains(self.target),
            "viewer_protocol_policy":self.viewer_protocol,"cache_policy_id":self.cache_policy,
            "origin_request_policy_id":self.origin_policy,"response_headers_policy_id":self.response_policy,
            "realtime_log_config_arn":self.realtime_log,"compress":self.compress,
            "allowed_methods":self.allowed.iter().map(|v| v.as_str()).collect::<Vec<_>>(),
            "lambda_associations":self.lambdas.iter().map(|v| json!({"arn":v.lambda_function_arn(),"event_type":v.event_type().as_str(),"include_body":v.include_body()})).collect::<Vec<_>>(),
            "function_associations":self.functions.iter().map(|v| json!({"arn":v.function_arn(),"event_type":v.event_type().as_str()})).collect::<Vec<_>>(),
            "min_ttl":self.minimum_ttl,"default_ttl":self.default_ttl,"max_ttl":self.maximum_ttl,
            "forwarded_values_collected":false})
    }
}

fn consume(used: &mut usize, count: usize) -> Result<(), AwsScanError> {
    *used = used.checked_add(count).ok_or(AwsScanError::ChildLimit)?;
    if *used > MAX_COMPONENTS {
        return Err(AwsScanError::InvalidData(
            "distribution exceeds 4096 mapped components".into(),
        ));
    }
    Ok(())
}

fn map_distribution(
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

fn distribution_resource(
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

fn map_origin(origin: &Origin) -> Result<Value, AwsScanError> {
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

fn distribution_edges(
    ctx: &Context,
    resource: &Resource,
    value: &DistributionSummary,
) -> Result<Vec<Edge>, AwsScanError> {
    let mut edges = Vec::new();
    for origin in value.origins().map(|v| v.items()).unwrap_or(&[]) {
        let target = match s3_bucket_from_dns(&ctx.partition, origin.domain_name()) {
            Some(bucket) => format!("arn:{}:s3:::{bucket}", ctx.partition),
            None => dns_reference(origin.domain_name())?,
        };
        edges.push(reference(ctx, resource, &target, "ORIGINATES_FROM", json!({"origin_id":origin.id(),"field":"Origins.DomainName","identity_evidence":"origin_dns_pattern","target_owner_resolution":"unknown"}))?);
        if let Some(vpc) = origin.vpc_origin_config() {
            let owner = vpc.owner_account_id().unwrap_or(&ctx.account);
            let target = format!(
                "aws-inventory:cloudfront-vpc-origin:{}:{owner}:{}",
                ctx.partition,
                vpc.vpc_origin_id()
            );
            edges.push(reference(ctx, resource, &target, "USES_VPC_ORIGIN", json!({"origin_id":origin.id(),"owner_resolution":if vpc.owner_account_id().is_some() {"reported"} else {"assumed_scanning_account"}}))?);
        }
    }
    if let Some(arn) = value
        .viewer_certificate()
        .and_then(|v| v.acm_certificate_arn())
    {
        edges.push(reference(
            ctx,
            resource,
            arn,
            "USES_CERTIFICATE",
            json!({"field":"ViewerCertificate.ACMCertificateArn"}),
        )?);
    }
    if value.web_acl_id().starts_with("arn:") {
        edges.push(reference(
            ctx,
            resource,
            value.web_acl_id(),
            "USES_WEB_ACL",
            json!({"field":"WebACLId"}),
        )?);
    }
    Ok(edges)
}

fn behavior_edges(
    ctx: &Context,
    resource: &Resource,
    behavior: &Behavior<'_>,
) -> Result<Vec<Edge>, AwsScanError> {
    let mut edges = Vec::new();
    for association in behavior.lambdas {
        edges.push(reference(ctx, resource, association.lambda_function_arn(), "ASSOCIATES_LAMBDA", json!({
            "path_pattern":behavior.path,"event_type":association.event_type().as_str(),"version_arn_preserved":true
        }))?);
    }
    for association in behavior.functions {
        edges.push(reference(
            ctx,
            resource,
            association.function_arn(),
            "ASSOCIATES_FUNCTION",
            json!({"path_pattern":behavior.path,"event_type":association.event_type().as_str()}),
        )?);
    }
    if let Some(log) = behavior.realtime_log {
        edges.push(reference(
            ctx,
            resource,
            log,
            "LOGS_TO",
            json!({"field":"RealtimeLogConfigArn","path_pattern":behavior.path}),
        )?);
    }
    Ok(edges)
}

fn s3_bucket_from_dns<'a>(partition: &str, domain: &'a str) -> Option<&'a str> {
    let suffix = match partition {
        "aws" | "aws-us-gov" => ".amazonaws.com",
        "aws-cn" => ".amazonaws.com.cn",
        _ => return None,
    };
    let base = domain.strip_suffix(suffix)?;
    let (bucket, rest) = base.rsplit_once(".s3")?;
    if bucket.is_empty()
        || bucket.len() > 255
        || !bucket
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return None;
    }
    let valid = rest.is_empty()
        || rest.strip_prefix(".dualstack.").is_some_and(valid_region)
        || rest.strip_prefix(".").is_some_and(valid_region)
        || rest.strip_prefix("-website-").is_some_and(valid_region)
        || rest.strip_prefix("-website.").is_some_and(valid_region)
        || rest.strip_prefix('-').is_some_and(valid_region);
    valid.then_some(bucket)
}

fn valid_region(value: &str) -> bool {
    crate::validate_region(value).is_ok()
        && value.contains('-')
        && value
            .bytes()
            .last()
            .is_some_and(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::super::tests::context;
    use super::*;
    use aws_sdk_cloudfront::types::{HttpVersion, PriceClass, ViewerProtocolPolicy};

    fn distribution(origin: Origin) -> Result<DistributionSummary, Box<dyn std::error::Error>> {
        Ok(DistributionSummary::builder()
            .id("E123")
            .arn("arn:aws:cloudfront::123456789012:distribution/E123")
            .status("Deployed")
            .last_modified_time(aws_smithy_types::DateTime::from_secs(1))
            .domain_name("d123.cloudfront.net")
            .comment("synthetic")
            .price_class(PriceClass::PriceClass100)
            .enabled(true)
            .web_acl_id("")
            .http_version(HttpVersion::Http2)
            .is_ipv6_enabled(true)
            .staging(false)
            .origins(
                aws_sdk_cloudfront::types::Origins::builder()
                    .quantity(1)
                    .items(origin)
                    .build()?,
            )
            .default_cache_behavior(
                DefaultCacheBehavior::builder()
                    .target_origin_id("primary")
                    .viewer_protocol_policy(ViewerProtocolPolicy::RedirectToHttps)
                    .build()?,
            )
            .build()?)
    }

    #[test]
    fn s3_origin_patterns_are_explicit_and_do_not_accept_lookalikes() {
        assert_eq!(
            s3_bucket_from_dns("aws", "a.b.s3.us-east-1.amazonaws.com"),
            Some("a.b")
        );
        assert_eq!(
            s3_bucket_from_dns("aws", "bucket.s3.amazonaws.com"),
            Some("bucket")
        );
        assert_eq!(
            s3_bucket_from_dns("aws", "bucket.s3-website-us-east-1.amazonaws.com"),
            Some("bucket")
        );
        assert_eq!(
            s3_bucket_from_dns("aws", "bucket.s3.dualstack.us-east-1.amazonaws.com"),
            Some("bucket")
        );
        assert_eq!(
            s3_bucket_from_dns("aws", "endpoint.s3-accesspoint.us-east-1.amazonaws.com"),
            None
        );
        assert_eq!(
            s3_bucket_from_dns("aws", "bucket.s3.amazonaws.com.evil.example"),
            None
        );
        assert_eq!(
            s3_bucket_from_dns("aws", "bucket.s3.cn-north-1.amazonaws.com.cn"),
            None
        );
    }

    #[test]
    fn custom_header_values_and_origin_paths_never_enter_inventory()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, _rx) = context()?;
        let origin = Origin::builder()
            .id("primary")
            .domain_name("bucket.s3.us-east-1.amazonaws.com")
            .origin_path("/NEVER_PERSIST_PATH")
            .custom_headers(
                aws_sdk_cloudfront::types::CustomHeaders::builder()
                    .quantity(1)
                    .items(
                        aws_sdk_cloudfront::types::OriginCustomHeader::builder()
                            .header_name("Authorization")
                            .header_value("NEVER_PERSIST_TOKEN")
                            .build()?,
                    )
                    .build()?,
            )
            .build()?;
        let body = map_distribution(&ctx, &distribution(origin)?, None)?;
        let serialized = serde_json::to_string(&body.resources)?;
        assert!(!serialized.contains("NEVER_PERSIST"));
        assert_eq!(
            body.edges.first().map(|e| e.to_id.as_str()),
            Some("arn:aws:s3:::bucket")
        );
        let resource = body.resources.first().ok_or("missing distribution")?;
        assert_eq!(resource.region, "global");
        assert_eq!(
            resource
                .properties
                .pointer("/origins/0/custom_header_count"),
            Some(&json!(1))
        );
        assert_eq!(
            resource.properties.get("tags_available"),
            Some(&json!(false))
        );
        Ok(())
    }

    #[test]
    fn distribution_uses_actual_certificate_arn_and_preserves_behavior_routing()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, _rx) = context()?;
        let origin = Origin::builder()
            .id("primary")
            .domain_name("backend.example.com")
            .build()?;
        let mut value = distribution(origin)?;
        value.viewer_certificate = Some(
            aws_sdk_cloudfront::types::ViewerCertificate::builder()
                .acm_certificate_arn("arn:aws:acm:us-east-1:123456789012:certificate/id")
                .build(),
        );
        let body = map_distribution(&ctx, &value, Some(BTreeMap::new()))?;
        assert!(body.edges.iter().any(|e| e.relation == "USES_CERTIFICATE"));
        assert!(
            body.edges
                .iter()
                .any(|e| e.to_id == "dns:backend.example.com")
        );
        let resource = body.resources.first().ok_or("missing distribution")?;
        assert_eq!(
            resource
                .properties
                .pointer("/default_cache_behavior/target_origin_id"),
            Some(&json!("primary"))
        );
        assert_eq!(
            resource
                .properties
                .pointer("/default_cache_behavior/target_in_configuration"),
            Some(&json!(true))
        );
        Ok(())
    }

    #[test]
    fn duplicate_origins_and_cross_account_source_arns_fail()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, _rx) = context()?;
        let origin = Origin::builder()
            .id("primary")
            .domain_name("backend.example.com")
            .build()?;
        let mut value = distribution(origin.clone())?;
        value.origins = Some(
            aws_sdk_cloudfront::types::Origins::builder()
                .quantity(2)
                .items(origin.clone())
                .items(origin)
                .build()?,
        );
        assert!(map_distribution(&ctx, &value, None).is_err());
        value.arn = "arn:aws:cloudfront::999999999999:distribution/E123".into();
        assert!(map_distribution(&ctx, &value, None).is_err());
        Ok(())
    }

    #[tokio::test]
    async fn unreviewed_cloudfront_partition_is_explicit_failure_without_api_calls()
    -> Result<(), Box<dyn std::error::Error>> {
        let (mut ctx, mut rx) = context()?;
        ctx.partition = "aws-us-gov".into();
        collect(&ctx).await?;
        assert_eq!(ctx.executor.stats().attempts, 0);
        let event = rx.try_recv()?;
        assert!(
            matches!(event, inventory_core::CollectionEvent::Failure(failure) if failure.category == "unsupported_partition")
        );
        Ok(())
    }
}
