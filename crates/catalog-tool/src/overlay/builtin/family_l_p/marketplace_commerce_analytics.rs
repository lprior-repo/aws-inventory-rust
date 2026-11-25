use super::policy::{Group, compile};
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[Group {
    operations: &["GenerateDataSet", "StartSupportDataExport"],
    class: OverlayClass::Deny(DenyReason::Mutating),
    reason: None,
}];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("marketplace-commerce-analytics", GROUPS, &[])
}
