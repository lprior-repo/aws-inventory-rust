use super::{NameError, ServiceOverlay, construction};
mod decisions_0;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    construction::reviewed("rds-data", &[], &[decisions_0::DECISIONS])
}
