//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/iotsecuretunneling/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![(
        "ListTunnels",
        policy::query(
            policy::enumerate(
                "tunnelSummaries",
                policy::arn("tunnelArn")?,
                Some(("nextToken", "nextToken")),
            )?,
            &["tunnelArn"],
        )?,
    )];
    policy::finish(
        "iotsecuretunneling",
        approved,
        DEFERRED,
        "Tunnel summaries omit source/destination access tokens and are approved. DescribeTunnel and other detail/capability paths are not approved; tunnel lifecycle mutations are denied.",
        &[(policy::DenyReason::Mutating, MUTATING)],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &["DescribeTunnel", "ListTagsForResource"];

const MUTATING: &[&str] = &[
    "CloseTunnel",
    "OpenTunnel",
    "RotateTunnelAccessToken",
    "TagResource",
    "UntagResource",
];
