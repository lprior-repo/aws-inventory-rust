// Pinned model: .local/aws-models/models/emr-serverless/service/2021-07-13/emr-serverless-2021-07-13.json
use super::support::{arn, bound, inventory, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};
use crate::classes::DenyReason;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "emr-serverless",
        vec![
        // Lists applications based on a set of parameters.
        queried(inventory("ListApplications", "applications", arn("arn")?, token("nextToken", "nextToken")?)?, &["arn", "id", "name", "type", "state", "stateDetails"])?,
        // These summary operations omit the job driver, arguments and executable session content.
        queried(bound(inventory("ListJobRuns", "jobRuns", arn("arn")?, token("nextToken", "nextToken")?)?,
            &[("applicationId", "ListApplications", "id")])?, &["arn", "applicationId", "id", "name", "executionRole", "state", "type"])?,
        queried(bound(inventory("ListSessions", "sessions", arn("arn")?, token("nextToken", "nextToken")?)?,
            &[("applicationId", "ListApplications", "id")])?, &["arn", "applicationId", "sessionId", "name", "executionRoleArn", "state"])?,
        ],
        &[super::s_emr_serverless_decisions_0::ENTRIES],
        &[
            // Unlike ListSessions, this API returns an authentication token, not session metadata.
            ("GetSessionEndpoint", DenyReason::Secret),
        ],
    )
}
