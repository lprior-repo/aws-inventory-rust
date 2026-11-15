//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/iam-toolbox/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![

    ];
    policy::finish(
        "iam-toolbox", approved, DEFERRED,
        "Authorization troubleshooting returns request/evaluation context, not an account-wide resource listing; caller-provided request ID and sensitive-context review are unresolved. No callable policy is approved.",
        &[],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "GetRequestAuthorizationDetails",
];

