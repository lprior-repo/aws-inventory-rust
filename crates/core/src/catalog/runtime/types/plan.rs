use super::super::RuntimeCatalogError;
use super::{
    BindingPlan, EndpointClass, EndpointPathMode, IdentityPlan, ItemPlan, PagePlan, PlanPurpose,
    PlanScope, ResolvedEndpoint, ReviewProvenance, RuntimeProtocol,
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
}

impl EndpointTemplate {
    pub(crate) fn resolve(
        &self,
        region: Option<&RegionId>,
    ) -> Result<ResolvedEndpoint, RuntimeCatalogError> {
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
    pub(crate) bindings: Box<[BindingPlan]>,
    pub(crate) pagination: PagePlan,
    pub(crate) exclusions: Exclusions,
    pub(crate) endpoint: EndpointTemplate,
    pub(crate) review: ReviewProvenance,
    pub(crate) purpose: PlanPurpose,
    pub(crate) protocol: RuntimeProtocol,
}

pub struct CallablePlan<'a> {
    parts: PlanParts<'a>,
}

impl<'a> CallablePlan<'a> {
    pub(crate) fn new(parts: PlanParts<'a>) -> Self {
        Self { parts }
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
