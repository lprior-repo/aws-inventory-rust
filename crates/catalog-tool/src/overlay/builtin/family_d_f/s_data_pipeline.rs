// Pinned model: .local/aws-models/models/data-pipeline/service/2012-10-29/data-pipeline-2012-10-29.json
use super::support::{inventory, key, queried, reviewed, truncated};
use crate::overlay::{NameError, ServiceOverlay};
use crate::classes::DenyReason;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "data-pipeline",
        vec![
        // Lists the pipeline identifiers for all active pipelines that you have permission to access.
        queried(inventory("ListPipelines", "pipelineIdList", key(&["id"])?, truncated("marker", "marker", "hasMoreResults")?)?, &["id", "name"])?,
        ],
        &[super::s_data_pipeline_decisions_0::ENTRIES],
        &[
            // Gets the definition of the specified pipeline.
            ("GetPipelineDefinition", DenyReason::Content),
            // Task runners call EvaluateExpression to evaluate a string in the context of the specified object.
            ("EvaluateExpression", DenyReason::Content),
        ],
    )
}
