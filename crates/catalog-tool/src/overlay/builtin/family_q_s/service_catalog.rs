// Entire pinned service accounted for; no runtime name-prefix authorization.
use super::{construction, NameError, ServiceOverlay};
mod calls_0;
mod decisions_0;
mod decisions_1;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    construction::reviewed(
        "service-catalog",
        &[calls_0::CALLS],
        &[decisions_0::DECISIONS, decisions_1::DECISIONS],
    )
}
