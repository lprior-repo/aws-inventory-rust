// Entire pinned service accounted for; no runtime name-prefix authorization.
use super::{construction, NameError, ServiceOverlay};
mod calls_0;
mod decisions_0;
mod decisions_1;
mod decisions_2;
mod decisions_3;
mod decisions_4;
mod decisions_5;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    construction::reviewed(
        "quicksight",
        &[calls_0::CALLS],
        &[decisions_0::DECISIONS, decisions_1::DECISIONS, decisions_2::DECISIONS, decisions_3::DECISIONS, decisions_4::DECISIONS, decisions_5::DECISIONS],
    )
}
