use super::policy::{Group, compile};
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[Group {
    operations: &[
        "DescribeServices",
        "GetAttributeValues",
        "GetPriceListFileUrl",
        "GetProducts",
        "ListPriceLists",
    ],
    class: OverlayClass::Deny(DenyReason::ReferenceData),
    reason: None,
}];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("pricing", GROUPS, &[])
}
