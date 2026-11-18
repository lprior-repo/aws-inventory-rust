// Pinned Smithy source: .local/aws-models/models/mediastore-data/service/2017-09-01/mediastore-data-2017-09-01.json
use super::policy::{Group, compile};
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &["DeleteObject", "PutObject"],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["DescribeObject", "GetObject", "ListItems"],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("mediastore-data", GROUPS, &[])
}
