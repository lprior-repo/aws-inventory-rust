// Entire pinned service accounted for; no runtime name-prefix authorization.
use super::{construction, NameError, ServiceOverlay};
mod decisions_0;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    construction::reviewed(
        "sagemakertrainingsessionruntime",
        &[],
        &[decisions_0::DECISIONS],
    )
}
