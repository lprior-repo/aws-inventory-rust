// Entire pinned service accounted for; no runtime name-prefix authorization.
use super::{NameError, ServiceOverlay, construction};
mod calls_0;
mod calls_1;
mod calls_2;
mod calls_3;
mod calls_4;
mod calls_5;
mod calls_7;
mod decisions_0;
mod decisions_1;
mod decisions_2;
mod decisions_3;
mod decisions_4;
mod decisions_5;
mod decisions_6;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    construction::reviewed(
        "sagemaker",
        &[
            calls_0::CALLS,
            calls_1::CALLS,
            calls_2::CALLS,
            calls_3::CALLS,
            calls_4::CALLS,
            calls_5::CALLS,
            calls_7::CALLS,
        ],
        &[
            decisions_0::DECISIONS,
            decisions_1::DECISIONS,
            decisions_2::DECISIONS,
            decisions_3::DECISIONS,
            decisions_4::DECISIONS,
            decisions_5::DECISIONS,
            decisions_6::DECISIONS,
        ],
    )
}
