// Entire pinned service accounted for; no runtime name-prefix authorization.
use super::{NameError, ServiceOverlay, construction};
mod calls_0;
mod calls_1;
mod decisions_0;
mod decisions_1;
mod decisions_2;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    construction::reviewed(
        "ssm",
        &[calls_0::CALLS, calls_1::CALLS],
        &[
            decisions_0::DECISIONS,
            decisions_1::DECISIONS,
            decisions_2::DECISIONS,
        ],
    )
}
