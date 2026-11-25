use super::support::{arn, bound, correlated, inventory, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "entityresolution",
        vec![
            queried(
                inventory(
                    "ListIdMappingWorkflows",
                    "workflowSummaries",
                    arn("workflowArn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["workflowArn", "workflowName"],
            )?,
            queried(
                inventory(
                    "ListIdNamespaces",
                    "idNamespaceSummaries",
                    arn("idNamespaceArn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["idNamespaceArn", "idNamespaceName", "type"],
            )?,
            queried(
                inventory(
                    "ListMatchingWorkflows",
                    "workflowSummaries",
                    arn("workflowArn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["workflowArn", "workflowName", "resolutionType"],
            )?,
            queried(
                inventory(
                    "ListSchemaMappings",
                    "schemaList",
                    arn("schemaArn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["schemaArn", "schemaName"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListIdMappingJobs",
                        "jobs",
                        correlated(&["workflowName"], &["jobId"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("workflowName", "ListIdMappingWorkflows", "workflowName")],
                )?,
                &["jobId", "status"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListMatchingJobs",
                        "jobs",
                        correlated(&["workflowName"], &["jobId"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("workflowName", "ListMatchingWorkflows", "workflowName")],
                )?,
                &["jobId", "status"],
            )?,
        ],
        &[super::s_entityresolution_decisions_0::ENTRIES],
        &[],
    )
}
