use super::support::{inventory, key, queried, reviewed, tagged};
use crate::overlay::{NameError, Paginator, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "dlm",
        vec![queried(
            tagged(
                inventory(
                    "GetLifecyclePolicies",
                    "Policies",
                    key(&["PolicyId"])?,
                    Paginator::None,
                )?,
                "Tags",
            )?,
            &["PolicyId", "State", "PolicyType"],
        )?],
        &[super::s_dlm_decisions_0::ENTRIES],
        &[],
    )
}
