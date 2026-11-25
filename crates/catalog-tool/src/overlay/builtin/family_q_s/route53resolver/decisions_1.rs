use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "UpdateOutpostResolver",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateResolverConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateResolverDnssecConfig",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateResolverEndpoint",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateResolverRule", Decision::Denied(DenyReason::Mutating)),
];
