use super::{defer, deny};
use super::super::service;
use crate::{classes::DenyReason, overlay::{NameError, ServiceOverlay}};

// Reviewed against pinned Smithy documentation, not operation-name read approval.
pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service("uxc", vec![
        ("GetAccountCustomizations", defer("Account customization is a singleton with no returned account/resource identity; service lists are reference data.")),
        ("ListServices", deny(DenyReason::ReferenceData)),
        ("UpdateAccountCustomizations", deny(DenyReason::Mutating)),
    ])
}
