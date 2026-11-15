use super::{defer, deny};
use super::super::service;
use crate::{classes::DenyReason, overlay::{NameError, ServiceOverlay}};

// Reviewed against pinned Smithy documentation, not operation-name read approval.
pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service("taxsettings", vec![
        ("BatchDeleteTaxRegistration", deny(DenyReason::Mutating)),
        ("BatchGetTaxExemptions", deny(DenyReason::Content)),
        ("BatchPutTaxRegistration", deny(DenyReason::Mutating)),
        ("DeleteSupplementalTaxRegistration", deny(DenyReason::Mutating)),
        ("DeleteTaxRegistration", deny(DenyReason::Mutating)),
        ("GetTaxExemptionTypes", deny(DenyReason::ReferenceData)),
        ("GetTaxInheritance", defer("Tax account configuration has no independently identifiable inventory resource; tax registration documents and financial records are excluded.")),
        ("GetTaxRegistration", deny(DenyReason::Content)),
        ("GetTaxRegistrationDocument", deny(DenyReason::Content)),
        ("ListSupplementalTaxRegistrations", deny(DenyReason::Content)),
        ("ListTaxExemptions", deny(DenyReason::Content)),
        ("ListTaxRegistrations", deny(DenyReason::Content)),
        ("PutSupplementalTaxRegistration", deny(DenyReason::Mutating)),
        ("PutTaxExemption", deny(DenyReason::Mutating)),
        ("PutTaxInheritance", deny(DenyReason::Mutating)),
        ("PutTaxRegistration", deny(DenyReason::Mutating)),
    ])
}
