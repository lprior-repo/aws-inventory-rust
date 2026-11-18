// Pinned model: .local/aws-models/models/fis/service/2020-12-01/fis-2020-12-01.json
use super::support::{arn, bound, correlated, inventory, queried, reviewed, tagged, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "fis",
        vec![
            // Lists your experiment templates.
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
            // Lists your experiments.
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
            // Owned configuration references a target account; it does not collect that account.
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
            // The pinned experiment operation has tokens but no paginated trait.
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
