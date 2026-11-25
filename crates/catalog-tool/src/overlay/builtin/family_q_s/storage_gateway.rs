use super::{NameError, ServiceOverlay, construction};
mod calls_0;
mod decisions_0;
mod decisions_1;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    construction::reviewed(
        "storage-gateway",
        &[calls_0::CALLS],
        &[decisions_0::DECISIONS, decisions_1::DECISIONS],
    )
}
