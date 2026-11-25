use super::{NameError, ServiceOverlay, construction};
mod decisions_0;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    construction::reviewed("sagemaker-a2i-runtime", &[], &[decisions_0::DECISIONS])
}
