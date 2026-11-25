use super::support::{inventory, key, queried, reviewed, truncated};
use crate::classes::DenyReason;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "data-pipeline",
        vec![queried(
            inventory(
                "ListPipelines",
                "pipelineIdList",
                key(&["id"])?,
                truncated("marker", "marker", "hasMoreResults")?,
            )?,
            &["id", "name"],
        )?],
        &[super::s_data_pipeline_decisions_0::ENTRIES],
        &[
            ("GetPipelineDefinition", DenyReason::Content),
            ("EvaluateExpression", DenyReason::Content),
        ],
    )
}
