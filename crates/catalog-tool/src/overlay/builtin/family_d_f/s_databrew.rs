use super::support::{inventory, key, queried, reviewed, tagged, token};
use crate::classes::DenyReason;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "databrew",
        vec![
            queried(
                tagged(
                    inventory(
                        "ListJobs",
                        "Jobs",
                        key(&["Name"])?,
                        token("NextToken", "NextToken")?,
                    )?,
                    "Tags",
                )?,
                &[
                    "ResourceArn",
                    "RoleArn",
                    "EncryptionKeyArn",
                    "AccountId",
                    "DatasetName",
                    "Name",
                    "Type",
                    "ProjectName",
                ],
            )?,
            queried(
                tagged(
                    inventory(
                        "ListProjects",
                        "Projects",
                        key(&["Name"])?,
                        token("NextToken", "NextToken")?,
                    )?,
                    "Tags",
                )?,
                &[
                    "ResourceArn",
                    "RoleArn",
                    "AccountId",
                    "DatasetName",
                    "Name",
                    "RecipeName",
                ],
            )?,
            queried(
                tagged(
                    inventory(
                        "ListRulesets",
                        "Rulesets",
                        key(&["Name"])?,
                        token("NextToken", "NextToken")?,
                    )?,
                    "Tags",
                )?,
                &["ResourceArn", "AccountId", "Name"],
            )?,
            queried(
                tagged(
                    inventory(
                        "ListSchedules",
                        "Schedules",
                        key(&["Name"])?,
                        token("NextToken", "NextToken")?,
                    )?,
                    "Tags",
                )?,
                &["ResourceArn", "AccountId", "Name"],
            )?,
        ],
        &[
            super::s_databrew_decisions_0::ENTRIES,
            super::s_databrew_decisions_1::ENTRIES,
        ],
        &[("ListDatasets", DenyReason::Content)],
    )
}
