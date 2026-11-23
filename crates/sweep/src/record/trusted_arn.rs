mod wisdom_ai_prompt;
use super::{IdentityContext, SweepError};
use inventory_core::{
    catalog::runtime::ArnResourceKind,
    inventory::{AccountId, Kind, RegionId, ServiceId},
};

struct Arn<'a> {
    partition: &'a str,
    service: &'a str,
    region: &'a str,
    account: &'a str,
    resource: &'a str,
}

fn parse(value: &str) -> Result<Arn<'_>, SweepError> {
    let invalid = || SweepError::Operation("modeled ARN is malformed");
    let mut parts = value.splitn(6, ':');
    let (Some("arn"), Some(partition), Some(service), Some(region), Some(account), Some(resource)) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return Err(invalid());
    };
    let arn = Arn { partition, service, region, account, resource };
    validate_parts(value, &arn)?;
    Ok(arn)
}

fn validate_parts(value: &str, arn: &Arn<'_>) -> Result<(), SweepError> {
    if value.len() > 4096
        || value.chars().any(char::is_control)
        || arn.partition.is_empty()
        || arn.resource.is_empty()
        || ServiceId::parse(arn.service).is_err()
        || (!arn.account.is_empty() && AccountId::parse(arn.account).is_err())
        || (!arn.region.is_empty() && RegionId::parse(arn.region).is_err()) {
        return Err(SweepError::Operation("modeled ARN is malformed"));
    }
    Ok(())
}

pub(crate) fn validate_arn(
    value: &str,
    trusted: Option<&IdentityContext<'_>>,
) -> Result<(), SweepError> {
    let arn = parse(value)?;
    if let Some(trusted) = trusted {
        validate_scope(&arn, trusted, false)?;
    }
    Ok(())
}

pub(crate) fn validate_kind_arn(kind: &Kind, value: &str) -> Result<(), SweepError> {
    match kind.as_str() {
        "AWS::Wisdom::AIPrompt" => {
            let arn = parse(value)?;
            if arn.service != "wisdom" || !wisdom_ai_prompt::valid(arn.resource) {
                return Err(SweepError::Operation("modeled ARN is malformed"));
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_scope(
    arn: &Arn<'_>,
    trusted: &IdentityContext<'_>,
    exact: bool,
) -> Result<(), SweepError> {
    let (partition, _, _) = partition(trusted.region.as_str());
    if arn.partition != partition
        || ((exact || !arn.account.is_empty()) && arn.account != trusted.account.as_str())
        || ((exact || !arn.region.is_empty()) && arn.region != trusted.region.as_str())
    {
        return Err(SweepError::Operation(
            "modeled ARN does not match trusted request scope",
        ));
    }
    Ok(())
}

pub(crate) fn resource_service(kind: &ArnResourceKind) -> &'static str {
    match kind {
        ArnResourceKind::SnsSubscription | ArnResourceKind::SnsTopic => "sns",
        ArnResourceKind::SqsQueue => "sqs",
    }
}

pub(crate) fn validate_resource_arn(
    value: &str,
    kind: &ArnResourceKind,
    trusted: &IdentityContext<'_>,
    request_url: Option<&str>,
) -> Result<(), SweepError> {
    let invalid = || SweepError::Operation("reviewed resource ARN is malformed");
    if value.len() > 512
        || !value.is_ascii()
        || value.bytes().any(|byte| byte.is_ascii_whitespace())
    {
        return Err(invalid());
    }
    let arn = parse(value)?;
    validate_scope(&arn, trusted, true)?;
    if arn.service != resource_service(kind) {
        return Err(invalid());
    }
    match kind {
        ArnResourceKind::SnsSubscription => validate_subscription(arn.resource),
        ArnResourceKind::SnsTopic => validate_name(arn.resource, 256),
        ArnResourceKind::SqsQueue => validate_queue(arn.resource, trusted, request_url),
    }
}

fn validate_name(value: &str, max: usize) -> Result<(), SweepError> {
    if !valid_name(value, max) {
        return Err(SweepError::Operation("reviewed resource ARN is malformed"));
    }
    Ok(())
}

fn validate_subscription(resource: &str) -> Result<(), SweepError> {
    let invalid = || SweepError::Operation("reviewed resource ARN is malformed");
    let (topic, id) = resource.split_once(':').ok_or_else(invalid)?;
    if !valid_name(topic, 256) || id.is_empty() || id.len() > 128
        || !id.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-') {
        return Err(invalid());
    }
    Ok(())
}

fn validate_queue(resource: &str, trusted: &IdentityContext<'_>, request_url: Option<&str>) -> Result<(), SweepError> {
    validate_name(resource, 80)?;
    let url = request_url.ok_or(SweepError::Operation("reviewed queue ARN omitted its request URL"))?;
    if queue_url(url, trusted)? != resource {
        return Err(SweepError::Operation("reviewed queue ARN does not match its request URL"));
    }
    Ok(())
}

fn valid_name(value: &str, max: usize) -> bool {
    let base = value.strip_suffix(".fifo").unwrap_or(value);
    !base.is_empty()
        && value.len() <= max
        && base
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

/// Queue identifiers: docs.aws.amazon.com/AWSSimpleQueueService/latest/SQSDeveloperGuide/sqs-queue-message-identifiers.html
/// Endpoint families: docs.aws.amazon.com/general/latest/gr/sqs-service.html
pub(crate) fn queue_url<'a>(
    url: &'a str,
    trusted: &IdentityContext<'_>,
) -> Result<&'a str, SweepError> {
    let invalid =
        || SweepError::Operation("reviewed queue URL is malformed or outside its trusted scope");
    if url.len() > 512 || !url.is_ascii() {
        return Err(invalid());
    }
    let (host, path) = url
        .strip_prefix("https://")
        .and_then(|value| value.split_once('/'))
        .ok_or_else(invalid)?;
    let (account, queue) = path.split_once('/').ok_or_else(invalid)?;
    if account != trusted.account.as_str()
        || !valid_name(queue, 80)
        || !queue_host(host, trusted.region.as_str())
    {
        return Err(invalid());
    }
    Ok(queue)
}

fn queue_host(host: &str, region: &str) -> bool {
    regional_queue_host(host, region)
        || (region == "us-east-1" && host == "queue.amazonaws.com")
        || (legacy_queue_region(region) && host.strip_prefix(region) == Some(".queue.amazonaws.com"))
}

fn regional_queue_host(host: &str, region: &str) -> bool {
    let (_, dns, dual) = partition(region);
    for service in ["sqs", "sqs-fips"] {
        if let Some(suffix) = host
            .strip_prefix(service)
            .and_then(|host| host.strip_prefix('.'))
            .and_then(|host| host.strip_prefix(region))
            .and_then(|host| host.strip_prefix('.'))
            && (suffix == dns || suffix == dual)
        {
            return true;
        }
    }
    false
}

fn legacy_queue_region(region: &str) -> bool {
    matches!(
        region,
        "us-east-2"
            | "us-west-1"
            | "us-west-2"
            | "af-south-1"
            | "ap-south-1"
            | "ap-northeast-3"
            | "ap-northeast-2"
            | "ap-southeast-1"
            | "ap-southeast-2"
            | "ap-northeast-1"
            | "ca-central-1"
            | "cn-north-1"
            | "cn-northwest-1"
            | "eu-central-1"
            | "eu-west-1"
            | "eu-west-2"
            | "eu-west-3"
            | "eu-north-1"
            | "sa-east-1"
    )
}

// AWS SDK partition outputs: github.com/aws/aws-sdk-go-v2/blob/main/internal/endpoints/awsrulesfn/partitions.json
fn partition(region: &str) -> (&'static str, &'static str, &'static str) {
    if region.starts_with("cn-") {
        ("aws-cn", "amazonaws.com.cn", "api.amazonwebservices.com.cn")
    } else if region.starts_with("us-gov-") {
        ("aws-us-gov", "amazonaws.com", "api.aws")
    } else if region.starts_with("us-isob-") {
        ("aws-iso-b", "sc2s.sgov.gov", "api.aws.scloud")
    } else if region.starts_with("us-iso-") {
        ("aws-iso", "c2s.ic.gov", "api.aws.ic.gov")
    } else if region.starts_with("eu-isoe-") {
        ("aws-iso-e", "cloud.adc-e.uk", "api.cloud-aws.adc-e.uk")
    } else if region.starts_with("us-isof-") {
        ("aws-iso-f", "csp.hci.ic.gov", "api.aws.hci.ic.gov")
    } else if region.starts_with("eusc-") {
        ("aws-eusc", "amazonaws.eu", "api.amazonwebservices.eu")
    } else {
        ("aws", "amazonaws.com", "api.aws")
    }
}

#[cfg(test)]
mod tests;
