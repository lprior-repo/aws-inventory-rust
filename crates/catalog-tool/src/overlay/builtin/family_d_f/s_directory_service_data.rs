// Pinned model: .local/aws-models/models/directory-service-data/service/2023-05-31/directory-service-data-2023-05-31.json
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
