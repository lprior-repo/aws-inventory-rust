use super::{defer, deny};
use super::super::service;
use crate::{classes::DenyReason, overlay::{NameError, ServiceOverlay}};

// Reviewed against pinned Smithy documentation, not operation-name read approval.
pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service("trustedadvisor", vec![
        ("BatchUpdateRecommendationResourceExclusion", deny(DenyReason::Mutating)),
        ("GetOrganizationRecommendation", defer("Recommendation APIs document global results but have regional endpoints; no reviewed account home-region dispatch is available.")),
        ("GetRecommendation", defer("Recommendation APIs document global results but have regional endpoints; no reviewed account home-region dispatch is available.")),
        ("ListChecks", deny(DenyReason::ReferenceData)),
        ("ListOrganizationRecommendationAccounts", defer("Recommendation APIs document global results but have regional endpoints; no reviewed account home-region dispatch is available.")),
        ("ListOrganizationRecommendationResources", defer("Recommendation APIs document global results but have regional endpoints; no reviewed account home-region dispatch is available.")),
        ("ListOrganizationRecommendations", defer("Recommendation APIs document global results but have regional endpoints; no reviewed account home-region dispatch is available.")),
        ("ListRecommendationResources", defer("Recommendation APIs document global results but have regional endpoints; no reviewed account home-region dispatch is available.")),
        ("ListRecommendations", defer("Recommendation APIs document global results but have regional endpoints; no reviewed account home-region dispatch is available.")),
        ("ListRecommendationsForResource", defer("Recommendation APIs document global results but have regional endpoints; no reviewed account home-region dispatch is available.")),
        ("UpdateOrganizationRecommendationLifecycle", deny(DenyReason::Mutating)),
        ("UpdateRecommendationLifecycle", deny(DenyReason::Mutating)),
    ])
}
