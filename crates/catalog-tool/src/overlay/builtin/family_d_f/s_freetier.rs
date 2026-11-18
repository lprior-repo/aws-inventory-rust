// Pinned model: .local/aws-models/models/freetier/service/2023-09-07/freetier-2023-09-07.json
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
