use super::policy::{Group, compile};
use crate::overlay::{NameError, OverlayClass, ServiceOverlay};

const GROUPS: &[Group] = &[Group {
    operations: &["GetEntitlements"],
    class: OverlayClass::Defer,
    reason: Some(
        "GetEntitlements requires seller ProductCode; no product discovery or account-scoped inventory operation supplies it.",
    ),
}];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("marketplace-entitlement-service", GROUPS, &[])
}
