// Entire pinned service accounted for; no runtime name-prefix authorization.
use super::{construction, NameError, ServiceOverlay};
mod calls_0;
mod calls_1;
mod calls_2;
mod decisions_0;
mod decisions_1;
mod decisions_2;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    construction::reviewed(
        "redshift",
        &[calls_0::CALLS, calls_1::CALLS, calls_2::CALLS],
        &[decisions_0::DECISIONS, decisions_1::DECISIONS, decisions_2::DECISIONS],
    )
}
