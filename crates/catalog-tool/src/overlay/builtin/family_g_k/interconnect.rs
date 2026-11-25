use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![(
        "ListConnections",
        policy::query(
            policy::enumerate(
                "connections",
                policy::arn("arn")?,
                Some(("nextToken", "nextToken")),
            )?,
            &["arn"],
        )?,
    )];
    policy::finish(
        "interconnect",
        approved,
        DEFERRED,
        "Connection summaries expose resource metadata, not activation keys. Partner environment/attach-point reference discovery and proposal-by-activation-key reads are not approved.",
        &[(policy::DenyReason::Mutating, MUTATING)],
    )
}

const DEFERRED: &[&str] = &[
    "DescribeConnectionProposal",
    "GetConnection",
    "GetEnvironment",
    "ListAttachPoints",
    "ListEnvironments",
    "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "AcceptConnectionProposal",
    "CreateConnection",
    "DeleteConnection",
    "TagResource",
    "UntagResource",
    "UpdateConnection",
];
