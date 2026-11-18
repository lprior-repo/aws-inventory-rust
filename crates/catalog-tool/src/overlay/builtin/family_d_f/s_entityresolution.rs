// Pinned model: .local/aws-models/models/entityresolution/service/2018-05-10/entityresolution-2018-05-10.json
use super::support::{arn, bound, correlated, inventory, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "entityresolution",
        vec![
            // Returns a list of all the IdMappingWorkflows that have been created for an Amazon Web Services account.
            queried(
                inventory(
                    "ListIdMappingWorkflows",
                    "workflowSummaries",
                    arn("workflowArn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["workflowArn", "workflowName"],
            )?,
            // Returns a list of all ID namespaces.
            queried(
                inventory(
                    "ListIdNamespaces",
                    "idNamespaceSummaries",
                    arn("idNamespaceArn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["idNamespaceArn", "idNamespaceName", "type"],
            )?,
            // Returns a list of all the MatchingWorkflows that have been created for an Amazon Web Services account.
            queried(
                inventory(
                    "ListMatchingWorkflows",
                    "workflowSummaries",
                    arn("workflowArn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["workflowArn", "workflowName", "resolutionType"],
            )?,
            // Returns a list of all the SchemaMappings that have been created for an Amazon Web Services account.
            queried(
                inventory(
                    "ListSchemaMappings",
                    "schemaList",
                    arn("schemaArn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["schemaArn", "schemaName"],
            )?,
            // JobSummary contains IDs, status and timestamps only; matched records are never fetched.
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
