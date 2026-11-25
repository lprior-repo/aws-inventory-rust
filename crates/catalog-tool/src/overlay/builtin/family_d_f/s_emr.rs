use super::support::{arn, bound, fixed_value, inventory, key, queried, reviewed, token};
use crate::classes::DenyReason;
use crate::overlay::{FixedValue, NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "emr",
        vec![
            queried(
                inventory(
                    "ListClusters",
                    "Clusters",
                    arn("ClusterArn")?,
                    token("Marker", "Marker")?,
                )?,
                &["ClusterArn", "Id", "Name"],
            )?,
            queried(
                inventory(
                    "ListSecurityConfigurations",
                    "SecurityConfigurations",
                    key(&["Name"])?,
                    token("Marker", "Marker")?,
                )?,
                &["Name"],
            )?,
            queried(
                inventory(
                    "ListStudios",
                    "Studios",
                    key(&["StudioId"])?,
                    token("Marker", "Marker")?,
                )?,
                &["StudioId", "Name", "VpcId"],
            )?,
            fixed_value(
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
                "From",
                FixedValue::Timestamp(inventory_core::record::Time::new(0, 0).map_err(|_| {
                    NameError {
                        value: "epoch timestamp".to_owned(),
                    }
                })?),
            )?,
            bound(
                queried(
                    inventory(
                        "ListStudioSessionMappings",
                        "SessionMappings",
                        key(&["StudioId", "IdentityType", "IdentityId"])?,
                        token("Marker", "Marker")?,
                    )?,
                    &["StudioId", "IdentityType", "IdentityId", "SessionPolicyArn"],
                )?,
                &[("StudioId", "ListStudios", "StudioId")],
            )?,
        ],
        &[
            super::s_emr_decisions_0::ENTRIES,
            super::s_emr_decisions_1::ENTRIES,
        ],
        &[
            ("ListSessions", DenyReason::Secret),
            ("GetSessionEndpoint", DenyReason::Secret),
        ],
    )
}
