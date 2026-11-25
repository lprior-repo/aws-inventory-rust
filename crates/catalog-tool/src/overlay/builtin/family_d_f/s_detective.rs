use super::support::{arn, bound, excluded, inventory, key, queried, reviewed, token};
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
                excluded(
                    inventory(
                        "ListInvitations",
                        "Invitations",
                        key(&["GraphArn", "AccountId"])?,
                        token("NextToken", "NextToken")?,
                    )?,
                    &["Invitations.EmailAddress"],
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
                excluded(
                    bound(
                        inventory(
                            "ListMembers",
                            "MemberDetails",
                            key(&["GraphArn", "AccountId"])?,
                            token("NextToken", "NextToken")?,
                        )?,
                        &[("GraphArn", "ListGraphs", "Arn")],
                    )?,
                    &["MemberDetails.EmailAddress"],
                )?,
                &["GraphArn", "AccountId", "Status", "AdministratorId"],
            )?,
        ],
        &[super::s_detective_decisions_0::ENTRIES],
        &[],
    )
}
