use super::support::{arn, bound, inventory, key, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "detective",
        vec![
            queried(
                inventory(
                    "ListGraphs",
                    "GraphList",
                    arn("Arn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["Arn"],
            )?,
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
