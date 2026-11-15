//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/interconnect/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        ("ListConnections", policy::query(policy::enumerate("connections", policy::arn("arn")?, Some(("nextToken", "nextToken")))?, &["arn"])?),
    ];
    policy::finish(
        "interconnect", approved, DEFERRED,
        "Connection summaries expose resource metadata, not activation keys. Partner environment/attach-point reference discovery and proposal-by-activation-key reads are not approved.",
        &[(policy::DenyReason::Mutating, MUTATING)],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "DescribeConnectionProposal", "GetConnection", "GetEnvironment", "ListAttachPoints", "ListEnvironments",
    "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "AcceptConnectionProposal", "CreateConnection", "DeleteConnection", "TagResource", "UntagResource",
    "UpdateConnection",
];
