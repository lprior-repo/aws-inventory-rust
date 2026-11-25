use super::support::reviewed;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "freetier",
        vec![],
        &[super::s_freetier_decisions_0::ENTRIES],
        &[],
    )
}
