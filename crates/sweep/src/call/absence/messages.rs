use aws_wire::response::{ResponseContext, errors};

enum Rule {
    GovCloud,
    Billing,
    FirewallManager,
    SecurityHub,
    Literal(&'static str),
}

impl Rule {
    fn matches(self, operation: &str, account: &str, region: &str, message: &str) -> bool {
        match self {
            Self::GovCloud => equals(
                message,
                &["No GovCloud account is linked to account ", account],
            ),
            Self::Billing => equals(
                message,
                &[
                    "Account ",
                    account,
                    " is not authorized to call ",
                    operation,
                    ". Only payer account is authorized",
                ],
            ),
            Self::FirewallManager => equals(
                message,
                &[
                    "No default admin could be found for account ",
                    account,
                    " in Region ",
                    region,
                ],
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
        }
    }
}

pub(super) fn confirms(
    service: &str,
    operation: &str,
    code: &str,
    context: ResponseContext<'_>,
    account: &str,
    region: &str,
) -> bool {
    let Some(rule) = rule(service, operation, code) else {
        return false;
    };
    matches!(errors::message(context), Ok(Some(message)) if rule.matches(operation, account, region, &message))
}

const SECURITY_LAKE_DISABLED: &str = "The request failed because Security Lake isn't enabled for your account in any Regions. Enable Security Lake for your account and then try again.";
const CACHE_GROUPS_UNAVAILABLE: &str =
    "Use of cache security groups is not permitted in this API version for your account.";
const CLUSTER_GROUPS_DISCONTINUED: &str = "Amazon Redshift has discontinued cluster security groups. For security group configuration, associate VPC security groups when creating or modifying your cluster.";

fn rule(service: &str, operation: &str, code: &str) -> Option<Rule> {
    match (service, operation, code) {
        ("account", "GetGovCloudAccountInformation", "ResourceNotFoundException") => {
            Some(Rule::GovCloud)
        }
        (
            "billingconductor",
            "ListBillingGroups" | "ListCustomLineItems" | "ListPricingPlans" | "ListPricingRules",
            "AccessDeniedException",
        ) => Some(Rule::Billing),
        (
            "fms",
            "GetNotificationChannel"
            | "ListAppsLists"
            | "ListPolicies"
            | "ListProtocolsLists"
            | "ListResourceSets",
            "AccessDeniedException",
        ) => Some(Rule::FirewallManager),
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
        _ => None,
    }
}

fn equals(message: &str, expected: &[&str]) -> bool {
    expected
        .iter()
        .try_fold(message, |rest, part| rest.strip_prefix(*part))
        == Some("")
}
