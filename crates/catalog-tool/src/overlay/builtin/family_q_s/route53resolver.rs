use super::{NameError, ServiceOverlay, construction};
mod calls_0;
mod calls_1;
mod decisions_0;
mod decisions_1;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    construction::reviewed(
        "route53resolver",
        &[calls_0::CALLS, calls_1::CALLS],
        &[decisions_0::DECISIONS, decisions_1::DECISIONS],
    )
}
