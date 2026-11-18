// Pinned model: .local/aws-models/models/devops-guru/service/2020-12-01/devops-guru-2020-12-01.json
use super::support::{inventory, key, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "devops-guru",
        vec![
            // Returns a list of notification channels configured for DevOps Guru.
            queried(
                inventory(
                    "ListNotificationChannels",
                    "Channels",
                    key(&["Id"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["Id"],
            )?,
        ],
        &[super::s_devops_guru_decisions_0::ENTRIES],
        &[],
    )
}
