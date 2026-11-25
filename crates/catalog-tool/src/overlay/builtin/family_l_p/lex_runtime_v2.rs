use super::policy::{Group, compile};
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &["DeleteSession", "PutSession"],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetSession",
            "RecognizeText",
            "RecognizeUtterance",
            "StartConversation",
        ],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("lex-runtime-v2", GROUPS, &[])
}
