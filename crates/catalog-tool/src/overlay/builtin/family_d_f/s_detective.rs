// Pinned model: .local/aws-models/models/detective/service/2018-10-26/detective-2018-10-26.json
use super::support::{arn, bound, inventory, key, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "detective",
        vec![
            // Returns the list of behavior graphs that the calling account is an administrator account of.
            queried(
                inventory(
                    "ListGraphs",
                    "GraphList",
                    arn("Arn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["Arn"],
            )?,
            // Invitations are graph/account access metadata; no investigation or ingested graph data is retrieved.
            queried(
                inventory(
                    "ListInvitations",
                    "Invitations",
                    key(&["GraphArn", "AccountId"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["GraphArn", "AccountId", "Status"],
            )?,
            queried(
                inventory(
                    "ListOrganizationAdminAccounts",
                    "Administrators",
                    key(&["AccountId"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["AccountId", "GraphArn"],
            )?,
            // AccountId alone is not a membership identity: the selected response also carries GraphArn.
            queried(
                bound(
                    inventory(
                        "ListMembers",
                        "MemberDetails",
                        key(&["GraphArn", "AccountId"])?,
                        token("NextToken", "NextToken")?,
                    )?,
                    &[("GraphArn", "ListGraphs", "Arn")],
                )?,
                &["GraphArn", "AccountId", "Status", "AdministratorId"],
            )?,
        ],
        &[super::s_detective_decisions_0::ENTRIES],
        &[],
    )
}
