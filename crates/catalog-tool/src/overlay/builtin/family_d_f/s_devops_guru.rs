use super::support::{inventory, key, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "devops-guru",
        vec![queried(
            inventory(
                "ListNotificationChannels",
                "Channels",
                key(&["Id"])?,
                token("NextToken", "NextToken")?,
            )?,
            &["Id"],
        )?],
        &[super::s_devops_guru_decisions_0::ENTRIES],
        &[],
    )
}
