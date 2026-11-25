use super::support::reviewed;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "directory-service-data",
        vec![],
        &[super::s_directory_service_data_decisions_0::ENTRIES],
        &[],
    )
}
