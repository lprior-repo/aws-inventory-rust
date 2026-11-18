// Entire pinned service accounted for; no runtime name-prefix authorization.
use super::{NameError, ServiceOverlay, construction};
mod decisions_0;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    construction::reviewed("ssm-guiconnect", &[], &[decisions_0::DECISIONS])
}
