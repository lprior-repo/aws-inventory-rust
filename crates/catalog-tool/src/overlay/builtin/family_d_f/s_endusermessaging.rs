use super::support::{arn, inventory, key, queried, reviewed, token};
use crate::classes::DenyReason;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "endusermessaging",
        vec![
            queried(
                inventory(
                    "ListBrandProfiles",
                    "brandProfiles",
                    arn("brandProfileArn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &[
                    "brandProfileArn",
                    "brandProfileId",
                    "brandProfileName",
                    "status",
                ],
            )?,
            queried(
                inventory(
                    "ListJobs",
                    "jobs",
                    key(&["jobId"])?,
                    token("nextToken", "nextToken")?,
                )?,
                &["jobId", "brandProfileId", "status", "operationType"],
            )?,
        ],
        &[super::s_endusermessaging_decisions_0::ENTRIES],
        &[
            ("ListNotifyCodeConfigurations", DenyReason::Content),
            ("GetNotifyCodeConfiguration", DenyReason::Content),
        ],
    )
}
