use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("CreateToken", Decision::Denied(DenyReason::Secret)),
    ("CreateTokenWithIAM", Decision::Denied(DenyReason::Secret)),
    ("RegisterClient", Decision::Denied(DenyReason::Secret)),
    (
        "StartDeviceAuthorization",
        Decision::Denied(DenyReason::Mutating),
    ),
];
