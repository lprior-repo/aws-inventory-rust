use super::{NameError, ServiceOverlay, construction};
mod decisions_0;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    construction::reviewed("sagemaker-runtime-http2", &[], &[decisions_0::DECISIONS])
}
