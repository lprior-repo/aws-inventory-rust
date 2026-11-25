use super::{NameError, ServiceOverlay, construction};
mod decisions_0;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    construction::reviewed("route53-recovery-cluster", &[], &[decisions_0::DECISIONS])
}
