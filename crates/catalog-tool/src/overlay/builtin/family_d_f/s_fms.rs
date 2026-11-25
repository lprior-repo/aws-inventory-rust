use super::support::{
    arn, bound, correlated, fixed_integer, inventory, key, queried, reviewed, token,
};
use crate::overlay::Paginator;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "fms",
        vec![
            queried(
                inventory(
                    "ListPolicies",
                    "PolicyList",
                    arn("PolicyArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "PolicyArn",
                    "PolicyId",
                    "PolicyName",
                    "ResourceType",
                    "SecurityServiceType",
                    "PolicyStatus",
                ],
            )?,
            queried(
                inventory(
                    "ListResourceSets",
                    "ResourceSets",
                    key(&["Id"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["Id", "Name", "ResourceSetStatus"],
            )?,
            queried(
                fixed_integer(
                    inventory(
                        "ListAppsLists",
                        "AppsLists",
                        arn("ListArn")?,
                        token("NextToken", "NextToken")?,
                    )?,
                    "MaxResults",
                    100,
                )?,
                &["ListArn", "ListId", "ListName"],
            )?,
            queried(
                fixed_integer(
                    inventory(
                        "ListProtocolsLists",
                        "ProtocolsLists",
                        arn("ListArn")?,
                        token("NextToken", "NextToken")?,
                    )?,
                    "MaxResults",
                    100,
                )?,
                &["ListArn", "ListId", "ListName"],
            )?,
            queried(
                inventory(
                    "GetNotificationChannel",
                    "$",
                    key(&["SnsTopicArn"])?,
                    Paginator::None,
                )?,
                &["SnsTopicArn", "SnsRoleName"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListResourceSetResources",
                        "Items",
                        correlated(&["Identifier"], &["URI"])?,
                        token("NextToken", "NextToken")?,
                    )?,
                    &[("Identifier", "ListResourceSets", "Id")],
                )?,
                &["URI", "AccountId"],
            )?,
        ],
        &[
            super::s_fms_decisions_0::ENTRIES,
            super::s_fms_decisions_1::ENTRIES,
        ],
        &[],
    )
}
