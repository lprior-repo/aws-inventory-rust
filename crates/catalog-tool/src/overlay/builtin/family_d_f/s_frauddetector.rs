use super::support::{arn, bound, correlated, inventory, queried, reviewed, token};
use crate::classes::DenyReason;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "frauddetector",
        vec![
            queried(
                inventory(
                    "GetDetectors",
                    "detectors",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["arn", "detectorId", "eventTypeName"],
            )?,
            queried(
                inventory(
                    "GetEntityTypes",
                    "entityTypes",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["arn", "name"],
            )?,
            queried(
                inventory(
                    "GetEventTypes",
                    "eventTypes",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["arn", "name"],
            )?,
            queried(
                inventory(
                    "GetLabels",
                    "labels",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["arn", "name"],
            )?,
            queried(
                inventory(
                    "GetModels",
                    "models",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["arn", "modelId", "modelType", "eventTypeName"],
            )?,
            queried(
                inventory(
                    "GetOutcomes",
                    "outcomes",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["arn", "name"],
            )?,
            queried(
                inventory(
                    "GetListsMetadata",
                    "lists",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["arn", "name", "variableType"],
            )?,
            queried(
                inventory(
                    "GetBatchImportJobs",
                    "batchImports",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["arn", "jobId", "status", "iamRoleArn"],
            )?,
            queried(
                inventory(
                    "GetBatchPredictionJobs",
                    "batchPredictions",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["arn", "jobId", "status", "iamRoleArn"],
            )?,
            queried(
                bound(
                    inventory(
                        "DescribeDetector",
                        "detectorVersionSummaries",
                        correlated(&["detectorId"], &["detectorVersionId"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("detectorId", "GetDetectors", "detectorId")],
                )?,
                &["detectorVersionId", "status"],
            )?,
        ],
        &[
            super::s_frauddetector_decisions_0::ENTRIES,
            super::s_frauddetector_decisions_1::ENTRIES,
        ],
        &[
            ("GetEventPrediction", DenyReason::Paid),
            ("GetEvent", DenyReason::Content),
            ("GetEventPredictionMetadata", DenyReason::Content),
            ("ListEventPredictions", DenyReason::Content),
            ("GetListElements", DenyReason::Content),
            ("GetExternalModels", DenyReason::Content),
            ("GetRules", DenyReason::Content),
        ],
    )
}
