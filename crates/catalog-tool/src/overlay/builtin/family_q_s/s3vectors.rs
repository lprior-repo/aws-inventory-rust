// Entire pinned service accounted for; no runtime name-prefix authorization.
use super::{construction, NameError, ServiceOverlay};
mod calls_0;
mod decisions_0;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    construction::reviewed(
        "s3vectors",
        &[calls_0::CALLS],
        &[decisions_0::DECISIONS],
    )
}
