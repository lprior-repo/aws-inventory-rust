use super::support::{arn, bound, correlated, inventory, queried, reviewed, tagged, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "fis",
        vec![
            queried(
                tagged(
                    inventory(
                        "ListExperimentTemplates",
                        "experimentTemplates",
                        arn("arn")?,
                        token("nextToken", "nextToken")?,
                    )?,
                    "tags",
                )?,
                &["arn", "id"],
            )?,
            queried(
                tagged(
                    inventory(
                        "ListExperiments",
                        "experiments",
                        arn("arn")?,
                        token("nextToken", "nextToken")?,
                    )?,
                    "tags",
                )?,
                &["arn", "id", "experimentTemplateId"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListTargetAccountConfigurations",
                        "targetAccountConfigurations",
                        correlated(&["experimentTemplateId"], &["accountId"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("experimentTemplateId", "ListExperimentTemplates", "id")],
                )?,
                &["accountId", "roleArn"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListExperimentTargetAccountConfigurations",
                        "targetAccountConfigurations",
                        correlated(&["experimentId"], &["accountId"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("experimentId", "ListExperiments", "id")],
                )?,
                &["accountId", "roleArn"],
            )?,
        ],
        &[super::s_fis_decisions_0::ENTRIES],
        &[],
    )
}
