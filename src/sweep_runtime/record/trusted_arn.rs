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
    if value.len() > 4096
        || value.chars().any(char::is_control)
        || partition.is_empty()
        || resource.is_empty()
        || ServiceId::parse(service).is_err()
        || (!account.is_empty() && AccountId::parse(account).is_err())
        || (!region.is_empty() && RegionId::parse(region).is_err())
    {
        return Err(invalid());
    }
    Ok(Arn {
        partition,
        service,
        region,
        account,
        resource,
    })
}

pub(in crate::sweep_runtime) fn validate_arn(
    value: &str,
    trusted: Option<&IdentityContext<'_>>,
) -> Result<(), SweepError> {
    let arn = parse(value)?;
    if let Some(trusted) = trusted {
        validate_scope(&arn, trusted, false)?;
    }
    Ok(())
}

pub(in crate::sweep_runtime) fn validate_kind_arn(
    kind: &Kind,
    value: &str,
) -> Result<(), SweepError> {
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

pub(in crate::sweep_runtime) fn resource_service(kind: &ArnResourceKind) -> &'static str {
    match kind {
        ArnResourceKind::SnsSubscription | ArnResourceKind::SnsTopic => "sns",
        ArnResourceKind::SqsQueue => "sqs",
    }
}

pub(in crate::sweep_runtime) fn validate_resource_arn(
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
        ArnResourceKind::SnsSubscription => {
            let (topic, id) = arn.resource.split_once(':').ok_or_else(invalid)?;
            if !valid_name(topic, 256)
                || id.is_empty()
                || id.len() > 128
                || !id
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            {
                return Err(invalid());
            }
        }
        ArnResourceKind::SnsTopic => {
            if !valid_name(arn.resource, 256) {
                return Err(invalid());
            }
        }
        ArnResourceKind::SqsQueue => {
            if !valid_name(arn.resource, 80) {
                return Err(invalid());
            }
            let url = request_url.ok_or(SweepError::Operation(
                "reviewed queue ARN omitted its request URL",
            ))?;
            if queue_url(url, trusted)? != arn.resource {
                return Err(SweepError::Operation(
                    "reviewed queue ARN does not match its request URL",
                ));
            }
        }
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
pub(in crate::sweep_runtime) fn queue_url<'a>(
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
    if region == "us-east-1" && host == "queue.amazonaws.com" {
        return true;
    }
    let legacy = matches!(
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
    );
    legacy && host.strip_prefix(region) == Some(".queue.amazonaws.com")
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
mod tests {
    use super::*;
    type TestResult = Result<(), Box<dyn std::error::Error>>;
    #[test]
    fn official_queue_endpoint_families_retain_case_sensitive_names() -> TestResult {
        let account = AccountId::parse("123456789012")?;
        for (region, host) in [
            ("us-east-1", "sqs.us-east-1.amazonaws.com"),
            ("us-east-1", "sqs-fips.us-east-1.api.aws"),
            ("us-east-1", "queue.amazonaws.com"),
            ("eu-west-1", "eu-west-1.queue.amazonaws.com"),
            ("cn-north-1", "sqs.cn-north-1.amazonaws.com.cn"),
            ("cn-north-1", "sqs.cn-north-1.api.amazonwebservices.com.cn"),
            ("us-iso-east-1", "sqs.us-iso-east-1.c2s.ic.gov"),
        ] {
            let region = RegionId::parse(region)?;
            let trusted = IdentityContext {
                account: &account,
                region: &region,
            };
            assert_eq!(
                queue_url(
                    &format!("https://{host}/123456789012/MyQueue.fifo"),
                    &trusted
                )?,
                "MyQueue.fifo"
            );
        }
        Ok(())
    }
    #[test]
    fn queue_url_rejects_spoofing_cross_scope_and_ambiguous_separators() -> TestResult {
        let account = AccountId::parse("123456789012")?;
        let region = RegionId::parse("us-east-1")?;
        let trusted = IdentityContext {
            account: &account,
            region: &region,
        };
        for url in [
            "http://sqs.us-east-1.amazonaws.com/123456789012/Queue",
            "https://sqs.us-east-1.amazonaws.com.evil.test/123456789012/Queue",
            "https://user@sqs.us-east-1.amazonaws.com/123456789012/Queue",
            "https://sqs.us-east-1.amazonaws.com:443/123456789012/Queue",
            "https://sqs.us-east-2.amazonaws.com/123456789012/Queue",
            "https://sqs.us-east-1.amazonaws.com/999999999999/Queue",
            "https://sqs.us-east-1.amazonaws.com/123456789012/Queue%2fOther",
            "https://sqs.us-east-1.amazonaws.com/123456789012/Queue/Other",
            "https://sqs.us-east-1.amazonaws.com/123456789012/Queue?x=1",
            "https://sqs.us-east-1.amazonaws.com/123456789012/Queue#fragment",
            "https://sqs.us-east-1.amazonaws.com/123456789012/",
        ] {
            assert!(queue_url(url, &trusted).is_err(), "{url}");
        }
        Ok(())
    }
    #[test]
    fn selected_queue_arn_must_match_requested_queue_and_resource_type() -> TestResult {
        let account = AccountId::parse("123456789012")?;
        let region = RegionId::parse("us-east-1")?;
        let trusted = IdentityContext {
            account: &account,
            region: &region,
        };
        let url = "https://sqs.us-east-1.amazonaws.com/123456789012/Queue";
        assert!(
            validate_resource_arn(
                "arn:aws:sqs:us-east-1:123456789012:Queue",
                &ArnResourceKind::SqsQueue,
                &trusted,
                Some(url)
            )
            .is_ok()
        );
        for arn in [
            "arn:aws:sqs:us-east-1:123456789012:Other",
            "arn:aws:sns:us-east-1:123456789012:Queue",
            "arn:aws:sqs:us-west-2:123456789012:Queue",
            "arn:aws-cn:sqs:us-east-1:123456789012:Queue",
            "arn:aws:sqs:us-east-1:999999999999:Queue",
            "arn:aws:sqs:us-east-1:123456789012:Queue:extra",
        ] {
            assert!(
                validate_resource_arn(arn, &ArnResourceKind::SqsQueue, &trusted, Some(url))
                    .is_err(),
                "{arn}"
            );
        }
        assert!(
            validate_resource_arn(
                "arn:aws:sqs:us-east-1:123456789012:Queue",
                &ArnResourceKind::SqsQueue,
                &trusted,
                None
            )
            .is_err()
        );
        Ok(())
    }
}
