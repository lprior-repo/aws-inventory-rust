// Pinned model: .local/aws-models/models/emr/service/2009-03-31/emr-2009-03-31.json
use super::support::{arn, inventory, key, queried, reviewed, token};
use crate::classes::DenyReason;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "emr",
        vec![
            // Provides the status of all clusters visible to this Amazon Web Services account.
            queried(
                inventory(
                    "ListClusters",
                    "Clusters",
                    arn("ClusterArn")?,
                    token("Marker", "Marker")?,
                )?,
                &["ClusterArn", "Id", "Name"],
            )?,
            // Lists all the security configurations visible to this account, providing their creation dates and times, and their names.
            queried(
                inventory(
                    "ListSecurityConfigurations",
                    "SecurityConfigurations",
                    key(&["Name"])?,
                    token("Marker", "Marker")?,
                )?,
                &["Name"],
            )?,
            // Returns a list of all Amazon EMR Studios associated with the Amazon Web Services account.
            queried(
                inventory(
                    "ListStudios",
                    "Studios",
                    key(&["StudioId"])?,
                    token("Marker", "Marker")?,
                )?,
                &["StudioId", "Name", "VpcId"],
            )?,
            // Summary metadata does not contain notebook source, parameters, or execution output.
            queried(
                inventory(
                    "ListNotebookExecutions",
                    "NotebookExecutions",
                    key(&["NotebookExecutionId"])?,
                    token("Marker", "Marker")?,
                )?,
                &[
                    "NotebookExecutionId",
                    "EditorId",
                    "NotebookExecutionName",
                    "Status",
                ],
            )?,
            // IdentityId is unique within its studio and principal type; keep both contextual key members.
            queried(
                inventory(
                    "ListStudioSessionMappings",
                    "SessionMappings",
                    key(&["StudioId", "IdentityType", "IdentityId"])?,
                    token("Marker", "Marker")?,
                )?,
                &["StudioId", "IdentityType", "IdentityId", "SessionPolicyArn"],
            )?,
        ],
        &[
            super::s_emr_decisions_0::ENTRIES,
            super::s_emr_decisions_1::ENTRIES,
        ],
        &[
            // EngineConfigurations.Properties returns arbitrary customer configuration values.
            ("ListSessions", DenyReason::Secret),
            // This endpoint API explicitly generates time-limited connection credentials.
            ("GetSessionEndpoint", DenyReason::Secret),
        ],
    )
}
