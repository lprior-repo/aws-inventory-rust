use super::policy::{Group, compile};
use crate::overlay::{NameError, OverlayClass, ServiceOverlay};

const GROUPS: &[Group] = &[Group {
    operations: &["GetBuyerDashboard"],
    class: OverlayClass::Defer,
    reason: Some("Buyer dashboard is an embedded commerce-report URL, not resource inventory."),
}];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("marketplace-reporting", GROUPS, &[])
}
