use super::super::RuntimeCatalogError;
use super::{
    BindingPlan, ContextProjectionPlan, Emission, EndpointClass, EndpointPathMode, ExecutionBranch,
    IdentityPlan, IdentityTextPlan, ItemPlan, KindClassificationPlan, PagePlan, PlanPurpose,
    PlanScope, RequestRouting, ResolvedEndpoint, ResourceEmissions, ResourceProjectionPlan,
    ReviewProvenance, RowPolicyPlan, RuntimeProtocol, TagPlan,
};
use crate::{
    inventory::{Kind, RegionId},
    record::Exclusions,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EndpointTemplate {
    pub(crate) url: String,
    pub(crate) signing_region: String,
    pub(crate) signing_name: String,
    pub(crate) path_mode: EndpointPathMode,
    pub(crate) class: EndpointClass,
    pub(crate) exceptions: Box<[EndpointException]>,
}

/// Concrete endpoint that overrides the template for one region label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EndpointException {
    /// Region the compiled catalog names; `-` for a request without a region.
    pub(crate) region: String,
    pub(crate) url: String,
    pub(crate) signing_name: String,
    pub(crate) signing_region: String,
}

impl EndpointTemplate {
    pub(crate) fn resolve(
        &self,
        region: Option<&RegionId>,
    ) -> Result<ResolvedEndpoint, RuntimeCatalogError> {
        let label = region.map_or("-", RegionId::as_str);
        if let Some(exception) = self.exceptions.iter().find(|item| item.region == label) {
            return Ok(ResolvedEndpoint {
                url: exception.url.clone(),
                signing_region: exception.signing_region.clone(),
                signing_name: exception.signing_name.clone(),
                class: self.class,
                path_mode: self.path_mode,
            });
        }
        let (url, signing_region) = match self.class {
            EndpointClass::Regional => {
                let region = region.ok_or(RuntimeCatalogError::Endpoint)?;
                (
                    substitute_region(&self.url, region.as_str())?,
                    substitute_region(&self.signing_region, region.as_str())?,
                )
            }
            EndpointClass::Global => (self.url.clone(), self.signing_region.clone()),
        };
        Ok(ResolvedEndpoint {
            url,
            signing_region,
            signing_name: self.signing_name.clone(),
            class: self.class,
            path_mode: self.path_mode,
        })
    }
}

fn substitute_region(value: &str, region: &str) -> Result<String, RuntimeCatalogError> {
    if value.len() > 16_384 || region.len() > 63 {
        return Err(RuntimeCatalogError::Bounds);
    }
    let replaced = value.replace("{region}", region);
    if replaced.len() > 16_384 || replaced.contains('{') || replaced.contains('}') {
        return Err(RuntimeCatalogError::Endpoint);
    }
    Ok(replaced)
}

pub(crate) struct PlanParts<'a> {
    pub(crate) service_name: Box<str>,
    pub(crate) operation_name: Box<str>,
    pub(crate) service: crate::catalog::Service<'a>,
    pub(crate) operation: crate::catalog::Operation<'a>,
    pub(crate) kind: Kind,
    pub(crate) scope: PlanScope,
    pub(crate) items: ItemPlan,
    pub(crate) identity: IdentityPlan,
    pub(crate) tags: Option<TagPlan>,
    pub(crate) bindings: Box<[BindingPlan]>,
    pub(crate) pagination: PagePlan,
    pub(crate) exclusions: Exclusions,
    pub(crate) endpoint: EndpointTemplate,
    pub(crate) review: ReviewProvenance,
    pub(crate) purpose: PlanPurpose,
    pub(crate) protocol: RuntimeProtocol,
    pub(crate) facet_precedence: u16,
    pub(crate) routing: RequestRouting,
    pub(crate) branches: Box<[ExecutionBranch]>,
    pub(crate) projection: Option<ResourceProjectionPlan>,
    pub(crate) classification: Option<KindClassificationPlan>,
    pub(crate) emission: Emission,
    pub(crate) resources: ResourceEmissions,
    pub(crate) identity_text: Box<[(Box<str>, IdentityTextPlan<'a>)]>,
    pub(crate) row_policy: RowPolicyPlan,
    pub(crate) context_projection: ContextProjectionPlan,
}

pub struct CallablePlan<'a> {
    parts: PlanParts<'a>,
}

impl<'a> CallablePlan<'a> {
    /// Effective reviewed stage inputs, with exact-path overrides replacing common inputs.
    pub fn effective_bindings<'p>(
        &'p self,
        branch: Option<&'p ExecutionBranch>,
    ) -> impl Iterator<Item = &'p BindingPlan> {
        self.bindings()
            .iter()
            .filter(move |binding| {
                !branch.is_some_and(|stage| {
                    stage
                        .bindings
                        .iter()
                        .any(|overridden| overridden.path == binding.path)
                })
            })
            .chain(branch.into_iter().flat_map(|stage| stage.bindings.iter()))
    }

    pub fn effective_emission(&self, branch: Option<&ExecutionBranch>) -> Emission {
        branch
            .and_then(|stage| stage.emission)
            .unwrap_or(self.emission())
    }
    pub(crate) fn new(parts: PlanParts<'a>) -> Self {
        Self { parts }
    }

    pub fn facet_precedence(&self) -> u16 {
        self.parts.facet_precedence
    }

    pub(crate) fn set_facet_precedence(&mut self, precedence: u16) {
        self.parts.facet_precedence = precedence;
    }

    pub fn routing(&self) -> &RequestRouting {
        &self.parts.routing
    }

    pub fn branches(&self) -> &[ExecutionBranch] {
        &self.parts.branches
    }

    pub fn projection(&self) -> Option<&ResourceProjectionPlan> {
        self.parts.projection.as_ref()
    }

    pub fn classification(&self) -> Option<&KindClassificationPlan> {
        self.parts.classification.as_ref()
    }

    pub fn emission(&self) -> Emission {
        self.parts.emission
    }
    pub fn resource_emissions(&self) -> &ResourceEmissions {
        &self.parts.resources
    }
    pub fn row_policy(&self) -> &RowPolicyPlan {
        &self.parts.row_policy
    }

    pub fn context_projection(&self) -> &ContextProjectionPlan {
        &self.parts.context_projection
    }

    pub fn service(&self) -> crate::catalog::Service<'a> {
        self.parts.service
    }

    pub fn operation(&self) -> crate::catalog::Operation<'a> {
        self.parts.operation
    }

    pub fn kind(&self) -> &Kind {
        &self.parts.kind
    }

    pub fn scope(&self) -> PlanScope {
        self.parts.scope
    }

    pub fn items(&self) -> &ItemPlan {
        &self.parts.items
    }

    pub fn identity(&self) -> &IdentityPlan {
        &self.parts.identity
    }

    pub fn identity_text(
        &self,
        emission_key: &str,
    ) -> Result<&IdentityTextPlan<'a>, RuntimeCatalogError> {
        self.parts
            .identity_text
            .iter()
            .find(|(key, _)| key.as_ref() == emission_key)
            .map(|(_, plan)| plan)
            .ok_or(RuntimeCatalogError::Capability)
    }

    /// `None` means tags were not observed, not a complete empty snapshot.
    pub fn tags(&self) -> Option<&TagPlan> {
        self.parts.tags.as_ref()
    }

    pub fn bindings(&self) -> &[BindingPlan] {
        &self.parts.bindings
    }

    pub fn pagination(&self) -> &PagePlan {
        &self.parts.pagination
    }

    pub fn hash_exclusions(&self) -> &Exclusions {
        &self.parts.exclusions
    }

    pub fn review(&self) -> &ReviewProvenance {
        &self.parts.review
    }

    pub fn purpose(&self) -> &PlanPurpose {
        &self.parts.purpose
    }

    pub fn protocol(&self) -> RuntimeProtocol {
        self.parts.protocol
    }

    pub fn http(&self) -> Result<Option<crate::catalog::HttpOperation<'a>>, RuntimeCatalogError> {
        self.parts
            .operation
            .http()
            .map_err(RuntimeCatalogError::Catalog)
    }

    pub fn endpoint(&self, region: &RegionId) -> Result<ResolvedEndpoint, RuntimeCatalogError> {
        self.parts.endpoint.resolve(Some(region))
    }

    pub fn endpoint_for(
        &self,
        region: Option<&RegionId>,
    ) -> Result<ResolvedEndpoint, RuntimeCatalogError> {
        self.parts.endpoint.resolve(region)
    }

    pub fn endpoint_class(&self) -> EndpointClass {
        self.parts.endpoint.class
    }

    pub fn service_name(&self) -> &str {
        &self.parts.service_name
    }

    pub fn operation_name(&self) -> &str {
        &self.parts.operation_name
    }
}
