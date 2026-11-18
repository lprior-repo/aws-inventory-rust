// Pinned model: .local/aws-models/models/fms/service/2018-01-01/fms-2018-01-01.json
use super::support::{
    arn, bound, correlated, fixed_integer, inventory, key, queried, reviewed, token,
};
use crate::overlay::Paginator;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "fms",
        vec![
            // Returns an array of PolicySummary objects.
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
            // Returns an array of ResourceSetSummary objects.
            queried(
                inventory(
                    "ListResourceSets",
                    "ResourceSets",
                    key(&["Id"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["Id", "Name", "ResourceSetStatus"],
            )?,
            // MaxResults is required by the pinned input shape and bounded to 1..=100.
            // DefaultLists remains false: only caller-owned configuration lists are inventoried.
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
            // The channel is configuration, not the referenced SNS topic or SNS message content.
            queried(
                inventory(
                    "GetNotificationChannel",
                    "$",
                    key(&["SnsTopicArn"])?,
                    Paginator::None,
                )?,
                &["SnsTopicArn", "SnsRoleName"],
            )?,
            // Memberships are parent-qualified association nodes; URI does not prove a resource was collected.
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
