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

const DEFERRED: &[&str] = &["DescribeTunnel", "ListTagsForResource"];

const MUTATING: &[&str] = &[
    "CloseTunnel",
    "OpenTunnel",
    "RotateTunnelAccessToken",
    "TagResource",
    "UntagResource",
];
