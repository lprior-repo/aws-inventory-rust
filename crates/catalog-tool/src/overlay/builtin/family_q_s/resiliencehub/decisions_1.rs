use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("UpdateAppVersion", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateAppVersionAppComponent",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateAppVersionResource",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateResiliencyPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
];
