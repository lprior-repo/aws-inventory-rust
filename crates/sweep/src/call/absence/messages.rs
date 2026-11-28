use aws_wire::response::{ResponseContext, errors};

#[derive(Clone, Copy)]
enum Rule {
    GovCloud,
    SecurityHub,
    Literal(&'static str),
    NotEnabled(http::StatusCode, &'static str),
}

impl Rule {
    fn matches(self, account: &str, region: &str, message: &str) -> bool {
        match self {
            Self::GovCloud => equals(
                message,
                &["No GovCloud account is linked to account ", account],
            ),
            Self::SecurityHub => equals(
                message,
                &[
                    "Account ",
                    account,
                    " is not subscribed to Security Hub CSPM in Region ",
                    region,
                ],
            ),
            Self::Literal(expected) => message == expected,
            Self::NotEnabled(_, expected) => message == expected,
        }
    }
}

pub(super) enum Classification {
    Empty,
    NotEnabled(String),
}

pub(super) fn classify(
    service: &str,
    operation: &str,
    code: &str,
    context: ResponseContext<'_>,
    account: &str,
    region: &str,
) -> Option<Classification> {
    let rule = rule(service, operation, code)?;
    if let Rule::NotEnabled(status, _) = rule
        && context.status != status
    {
        return None;
    }
    let message = errors::message(context).ok().flatten()?;
    rule.matches(account, region, &message).then(|| match rule {
        Rule::NotEnabled(_, _) => Classification::NotEnabled(message),
        _ => Classification::Empty,
    })
}

const SECURITY_LAKE_DISABLED: &str = "The request failed because Security Lake isn't enabled for your account in any Regions. Enable Security Lake for your account and then try again.";
const CACHE_GROUPS_UNAVAILABLE: &str =
    "Use of cache security groups is not permitted in this API version for your account.";
const CLUSTER_GROUPS_DISCONTINUED: &str = "Amazon Redshift has discontinued cluster security groups. For security group configuration, associate VPC security groups when creating or modifying your cluster.";
const REDSHIFT_SERVERLESS_NOT_CREATED: &str = "Workgroup and namespace not found. Set up resources with the create-namespace and create-workgroups operations.";
const SESV2_SANDBOX: &str = "Your account is still in the sandbox.";

fn rule(service: &str, operation: &str, code: &str) -> Option<Rule> {
    match (service, operation, code) {
        ("account", "GetGovCloudAccountInformation", "ResourceNotFoundException") => {
            Some(Rule::GovCloud)
        }
        ("macie2", "ListCustomDataIdentifiers" | "ListMembers", "AccessDeniedException") => {
            Some(Rule::Literal("Macie is not enabled."))
        }
        (
            "securityhub",
            "DescribeActionTargets"
            | "DescribeHub"
            | "GetEnabledStandards"
            | "ListAutomationRules"
            | "ListFindingAggregators"
            | "ListMembers",
            "InvalidAccessException",
        ) => Some(Rule::SecurityHub),
        ("snow-device-management", "ListDevices", "AccessDeniedException") => {
            Some(Rule::Literal("No devices found in this region."))
        }
        ("securitylake", "ListSubscribers", "ResourceNotFoundException") => {
            Some(Rule::Literal(SECURITY_LAKE_DISABLED))
        }
        ("elasticache", "DescribeCacheSecurityGroups", "InvalidParameterValue") => {
            Some(Rule::Literal(CACHE_GROUPS_UNAVAILABLE))
        }
        ("redshift", "DescribeClusterSecurityGroups", "InvalidParameterValue") => {
            Some(Rule::Literal(CLUSTER_GROUPS_DISCONTINUED))
        }
        (
            "redshift-serverless",
            "ListTableRestoreStatus",
            "ResourceNotFoundException",
        ) => Some(Rule::NotEnabled(
            http::StatusCode::NOT_FOUND,
            REDSHIFT_SERVERLESS_NOT_CREATED,
        )),
        ("sesv2", "ListImportJobs", "BadRequestException") => Some(Rule::NotEnabled(
            http::StatusCode::BAD_REQUEST,
            SESV2_SANDBOX,
        )),
        _ => None,
    }
}

fn equals(message: &str, expected: &[&str]) -> bool {
    expected
        .iter()
        .try_fold(message, |rest, part| rest.strip_prefix(*part))
        == Some("")
}
