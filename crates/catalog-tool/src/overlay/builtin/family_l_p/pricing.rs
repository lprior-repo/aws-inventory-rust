// Pinned Smithy source: .local/aws-models/models/pricing/service/2017-10-15/pricing-2017-10-15.json
use super::policy::{Group, compile};
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "DescribeServices", "GetAttributeValues", "GetPriceListFileUrl", "GetProducts", 
            "ListPriceLists",
        ],
        class: OverlayClass::Deny(DenyReason::ReferenceData),
        reason: None,
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("pricing", GROUPS, &[])
}
