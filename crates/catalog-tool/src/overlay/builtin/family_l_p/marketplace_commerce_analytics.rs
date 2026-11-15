// Pinned Smithy source: .local/aws-models/models/marketplace-commerce-analytics/service/2015-07-01/marketplace-commerce-analytics-2015-07-01.json
use super::policy::{Group, compile};
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "GenerateDataSet", "StartSupportDataExport",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("marketplace-commerce-analytics", GROUPS, &[])
}
