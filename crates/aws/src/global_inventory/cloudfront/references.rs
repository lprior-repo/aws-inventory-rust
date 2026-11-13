//! references responsibilities for cloudfront.
use super::*;

pub(super) fn distribution_edges(
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

pub(super) fn behavior_edges(
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

pub(super) fn s3_bucket_from_dns<'a>(partition: &str, domain: &'a str) -> Option<&'a str> {
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

pub(super) fn valid_region(value: &str) -> bool {
    crate::validate_region(value).is_ok()
        && value.contains('-')
        && value
            .bytes()
            .last()
            .is_some_and(|byte| byte.is_ascii_digit())
}
