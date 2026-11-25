use super::policy::{Group, compile};
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[Group {
    operations: &[
        "PutActionInteractions",
        "PutActions",
        "PutEvents",
        "PutItems",
        "PutUsers",
    ],
    class: OverlayClass::Deny(DenyReason::Mutating),
    reason: None,
}];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("personalize-events", GROUPS, &[])
}
