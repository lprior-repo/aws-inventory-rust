use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "UpdateSpacePermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateSpaceResources",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateTemplate", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateTemplateAlias",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateTemplatePermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateTheme", Decision::Denied(DenyReason::Mutating)),
    ("UpdateThemeAlias", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateThemePermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateTopic", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateTopicPermissions",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateTopicPermissionsV2",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateTopicRefreshSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateTopicV2", Decision::Denied(DenyReason::Mutating)),
    ("UpdateUser", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateUserCustomPermission",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateVPCConnection",
        Decision::Denied(DenyReason::Mutating),
    ),
];
