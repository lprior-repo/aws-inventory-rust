use super::support::{arn, bound, inventory, queried, reviewed, token};
use crate::classes::DenyReason;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "emr-serverless",
        vec![
            queried(
                inventory(
                    "ListApplications",
                    "applications",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["arn", "id", "name", "type", "state", "stateDetails"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListJobRuns",
                        "jobRuns",
                        arn("arn")?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("applicationId", "ListApplications", "id")],
                )?,
                &[
                    "arn",
                    "applicationId",
                    "id",
                    "name",
                    "executionRole",
                    "state",
                    "type",
                ],
            )?,
            queried(
                bound(
                    inventory(
                        "ListSessions",
                        "sessions",
                        arn("arn")?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("applicationId", "ListApplications", "id")],
                )?,
                &[
                    "arn",
                    "applicationId",
                    "sessionId",
                    "name",
                    "executionRoleArn",
                    "state",
                ],
            )?,
        ],
        &[super::s_emr_serverless_decisions_0::ENTRIES],
        &[("GetSessionEndpoint", DenyReason::Secret)],
    )
}
