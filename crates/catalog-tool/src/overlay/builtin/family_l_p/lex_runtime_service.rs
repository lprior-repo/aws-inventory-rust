// Pinned Smithy source: .local/aws-models/models/lex-runtime-service/service/2016-11-28/lex-runtime-service-2016-11-28.json
use super::policy::{Group, compile};
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "DeleteSession", "PutSession",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetSession", "PostContent", "PostText",
        ],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("lex-runtime-service", GROUPS, &[])
}
