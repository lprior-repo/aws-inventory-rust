// Pinned model: .local/aws-models/models/dlm/service/2018-01-12/dlm-2018-01-12.json
use super::support::{inventory, key, queried, reviewed, tagged};
use crate::overlay::{NameError, Paginator, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "dlm",
        vec![
            // Gets summary information about all or the specified data lifecycle policies.
            queried(
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
            )?,
        ],
        &[super::s_dlm_decisions_0::ENTRIES],
        &[],
    )
}
