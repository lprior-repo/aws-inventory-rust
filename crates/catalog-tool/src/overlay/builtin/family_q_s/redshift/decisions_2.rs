use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("RegisterNamespace", Decision::Denied(DenyReason::Mutating)),
    ("RejectDataShare", Decision::Denied(DenyReason::Mutating)),
    (
        "ResetClusterParameterGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("ResizeCluster", Decision::Denied(DenyReason::Secret)),
    (
        "RestoreFromClusterSnapshot",
        Decision::Denied(DenyReason::Secret),
    ),
    (
        "RestoreTableFromClusterSnapshot",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("ResumeCluster", Decision::Denied(DenyReason::Secret)),
    (
        "RevokeClusterSecurityGroupIngress",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RevokeEndpointAccess",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RevokeSnapshotAccess",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("RotateEncryptionKey", Decision::Denied(DenyReason::Secret)),
    (
        "UpdatePartnerStatus",
        Decision::Denied(DenyReason::Mutating),
    ),
];
