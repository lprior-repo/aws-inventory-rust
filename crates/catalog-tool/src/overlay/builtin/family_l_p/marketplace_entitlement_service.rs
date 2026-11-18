// Pinned Smithy source: .local/aws-models/models/marketplace-entitlement-service/service/2017-01-11/marketplace-entitlement-service-2017-01-11.json
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
