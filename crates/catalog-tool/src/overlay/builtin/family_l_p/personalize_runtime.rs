use super::policy::{Group, compile};
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[Group {
    operations: &[
        "GetActionRecommendations",
        "GetPersonalizedRanking",
        "GetRecommendations",
    ],
    class: OverlayClass::Deny(DenyReason::Paid),
    reason: None,
}];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("personalize-runtime", GROUPS, &[])
}
