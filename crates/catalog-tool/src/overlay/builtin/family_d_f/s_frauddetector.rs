// Pinned model: .local/aws-models/models/frauddetector/service/2019-11-15/frauddetector-2019-11-15.json
use super::support::{arn, bound, correlated, inventory, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};
use crate::classes::DenyReason;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "frauddetector",
        vec![
        // Gets all detectors or a single detector if a detectorId is specified.
        queried(inventory("GetDetectors", "detectors", arn("arn")?, token("nextToken", "nextToken")?)?, &["arn", "detectorId", "eventTypeName"])?,
        // Gets all entity types or a specific entity type if a name is specified.
        queried(inventory("GetEntityTypes", "entityTypes", arn("arn")?, token("nextToken", "nextToken")?)?, &["arn", "name"])?,
        // Gets all event types or a specific event type if name is provided.
        queried(inventory("GetEventTypes", "eventTypes", arn("arn")?, token("nextToken", "nextToken")?)?, &["arn", "name"])?,
        // Gets all labels or a specific label if name is provided.
        queried(inventory("GetLabels", "labels", arn("arn")?, token("nextToken", "nextToken")?)?, &["arn", "name"])?,
        // Gets one or more models.
        queried(inventory("GetModels", "models", arn("arn")?, token("nextToken", "nextToken")?)?, &["arn", "modelId", "modelType", "eventTypeName"])?,
        // Gets one or more outcomes.
        queried(inventory("GetOutcomes", "outcomes", arn("arn")?, token("nextToken", "nextToken")?)?, &["arn", "name"])?,
        // Gets the metadata of either all the lists under the account or the specified list.
        queried(inventory("GetListsMetadata", "lists", arn("arn")?, token("nextToken", "nextToken")?)?, &["arn", "name", "variableType"])?,
        // Returns job metadata and S3 locations, not the imported records.
        queried(inventory("GetBatchImportJobs", "batchImports", arn("arn")?, token("nextToken", "nextToken")?)?, &["arn", "jobId", "status", "iamRoleArn"])?,
        // Returns existing job metadata; does not execute predictions or retrieve prediction results.
        queried(inventory("GetBatchPredictionJobs", "batchPredictions", arn("arn")?, token("nextToken", "nextToken")?)?, &["arn", "jobId", "status", "iamRoleArn"])?,
        // Version summaries contain lifecycle metadata, not rules, event records or predictions.
        queried(bound(inventory("DescribeDetector", "detectorVersionSummaries", correlated(&["detectorId"], &["detectorVersionId"])?, token("nextToken", "nextToken")?)?,
            &[("detectorId", "GetDetectors", "detectorId")])?, &["detectorVersionId", "status"])?,
        ],
        &[super::s_frauddetector_decisions_0::ENTRIES, super::s_frauddetector_decisions_1::ENTRIES],
        &[
            // Evaluates an event against a detector version.
            ("GetEventPrediction", DenyReason::Paid),
            // Retrieves details of events stored with Amazon Fraud Detector.
            ("GetEvent", DenyReason::Content),
            // Gets details of the past fraud predictions for the specified event ID, event type, detector ID, and detector version ID that was generated in the specified time period.
            ("GetEventPredictionMetadata", DenyReason::Content),
            // Gets a list of past predictions.
            ("ListEventPredictions", DenyReason::Content),
            // Gets all the elements in the specified list.
            ("GetListElements", DenyReason::Content),
            // JSON/CSV input templates are customer workload payload construction programs.
            ("GetExternalModels", DenyReason::Content),
            // Rule.expression is the stored executable fraud-detection expression, not a rule summary.
            ("GetRules", DenyReason::Content),
        ],
    )
}
