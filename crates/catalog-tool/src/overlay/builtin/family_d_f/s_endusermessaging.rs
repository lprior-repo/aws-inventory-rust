// Pinned model: .local/aws-models/models/endusermessaging/service/2026-09-21/endusermessaging-2026-09-21.json
use super::support::{arn, inventory, key, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};
use crate::classes::DenyReason;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "endusermessaging",
        vec![
        // Retrieves a paginated list of the brand profiles in your account.
        queried(inventory("ListBrandProfiles", "brandProfiles", arn("brandProfileArn")?, token("nextToken", "nextToken")?)?, &["brandProfileArn", "brandProfileId", "brandProfileName", "status"])?,
        // Existing asynchronous operation summaries, never customer messages or verification-code sends.
        queried(inventory("ListJobs", "jobs", key(&["jobId"])?, token("nextToken", "nextToken")?)?, &["jobId", "brandProfileId", "status", "operationType"])?,
        ],
        &[super::s_endusermessaging_decisions_0::ENTRIES],
        &[
            // Retrieves a paginated list of the notify code configurations in your account.
            ("ListNotifyCodeConfigurations", DenyReason::Content),
            // Retrieves a notify code configuration.
            ("GetNotifyCodeConfiguration", DenyReason::Content),
        ],
    )
}
