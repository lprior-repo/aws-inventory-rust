use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![];
    policy::finish(
        "iam-toolbox",
        approved,
        DEFERRED,
        "Authorization troubleshooting returns request/evaluation context, not an account-wide resource listing; caller-provided request ID and sensitive-context review are unresolved. No callable policy is approved.",
        &[],
    )
}

const DEFERRED: &[&str] = &["GetRequestAuthorizationDetails"];
