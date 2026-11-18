// Pinned Smithy source: .local/aws-models/models/marketplace-metering/service/2016-01-14/marketplace-metering-2016-01-14.json
use super::policy::{Group, compile};
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &["BatchMeterUsage", "RegisterUsage"],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["MeterUsage", "ResolveCustomer"],
        class: OverlayClass::Defer,
        reason: Some(
            "Metering submits billable consumption or resolves a supplied registration token; no metadata resource discovery exists.",
        ),
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("marketplace-metering", GROUPS, &[])
}
