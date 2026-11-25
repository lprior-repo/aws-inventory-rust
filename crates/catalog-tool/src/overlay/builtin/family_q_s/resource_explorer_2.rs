use super::{NameError, ServiceOverlay, construction};
mod calls_0;
mod decisions_0;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    construction::reviewed(
        "resource-explorer-2",
        &[calls_0::CALLS],
        &[decisions_0::DECISIONS],
    )
}
