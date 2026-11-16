mod decode;
mod types;
mod tag_endpoint;
pub use tag_endpoint::{TagEndpointPlan, TagEndpointParentPlan, TagEndpointRequestPlan, TagEndpointSource};

use crate::{
    catalog::{Catalog, CatalogError},
    inventory::{InventoryError, KindSchema},
    record::RecordError,
};
use std::sync::Arc;
pub use types::{
    CatalogSourceProvenance, ModelSourceProvenance,
    RowPolicyPlan, RowPredicatePlan, RowOperandPlan, RowValueType, MissingSelected, ChildEligibilityPlan,
    RequestRouting, ExecutionBranch, ResourceProjectionPlan, KindClassificationPlan,
    ReferenceOwnerRule, ArnOwnerRule, ArnRegionRule, ContextSource, Emission, SelectionKind,
    IdentityFieldPlan, IdentityLeafPlan, IdentityChoicePlan, IdentityBranchPlan, IdentityPredicatePlan,
    ContextProjectionPlan, BindingAlternativePlan,
    ArnResourceKind, UrlResourceKind, BindingTransform,
    BindingPlan, BindingSource, BindingValue, CallablePlan, EndpointClass, EndpointPathMode,
    IdentityPart, IdentityPlan, ItemPlan, KindCoverage, KindNamePath, PagePlan, PageSizeBinding, PageSizeType,
    PlanPurpose, PlanScope, RelationScope, RelationSource, RelationshipPlan, ResolvedEndpoint, ReviewDecision,
    RelationPathMode, RelationTargetMode, RelationGuard,
    RelationshipComponentPlan,
    ResourceEmissionPlan, ResourceEmissions,
    ReviewDisposition, ReviewProvenance, RuntimeProtocol, TagPlan, TokenBinding, TokenPlacement, TokenType,
    TokenSelection,
};

const MAX_PLANS: usize = 65_536;

/// Hashes the canonical compiled capability fields without including their source receipt.
pub fn compiled_inputs_sha256(extension: crate::record::MapRef<'_>) -> Result<[u8; 32], RuntimeCatalogError> {
    decode::compiled_inputs_sha256(extension)
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RuntimeCatalogError {
    #[error(transparent)]
    Catalog(#[from] CatalogError),
    #[error(transparent)]
    Record(#[from] RecordError),
    #[error(transparent)]
    Inventory(#[from] InventoryError),
    #[error(
        "compiled catalog is missing runtime capability metadata; recompile with reviewed runtime metadata"
    )]
    MissingExtension,
    #[error("unsupported runtime capability metadata version {0}")]
    ExtensionVersion(i64),
    #[error("runtime capability metadata is malformed or inconsistent")]
    Capability,
    #[error("incomplete resource kind coverage: unknown discriminator {discriminator:?}")]
    UnclassifiedKind { discriminator: String },
    #[error("runtime capability metadata exceeds a validated size bound")]
    Bounds,
    #[error("verified endpoint metadata cannot be resolved for this region")]
    Endpoint,
}

pub struct RuntimeCatalog<'a> {
    bytes: &'a [u8],
    catalog: Catalog<'a>,
    plans: Box<[CallablePlan<'a>]>,
    schemas: Box<[Arc<KindSchema>]>,
    name_paths: Box<[KindNamePath]>,
    relations: Box<[RelationshipPlan]>,
    coverage: Box<[KindCoverage]>,
    decisions: Box<[ReviewDecision]>,
    provenance: CatalogSourceProvenance,
}
impl<'a> RuntimeCatalog<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuntimeCatalogError> {
        let catalog = Catalog::borrow(bytes)?;
        let decoded = decode::read(bytes, catalog)?;
        if decoded.plans.len() > MAX_PLANS {
            return Err(RuntimeCatalogError::Bounds);
        }
        Ok(Self {
            bytes,
            catalog,
            plans: decoded.plans.into_boxed_slice(),
            schemas: decoded.schemas.into_boxed_slice(),
            name_paths: decoded.name_paths.into_boxed_slice(),
            relations: decoded.relations.into_boxed_slice(),
            coverage: decoded.coverage.into_boxed_slice(),
            decisions: decoded.decisions.into_boxed_slice(),
            provenance: decoded.provenance,
        })
    }

    pub fn catalog(&self) -> Catalog<'a> {
        self.catalog
    }

    pub fn source_provenance(&self) -> &CatalogSourceProvenance {
        &self.provenance
    }

    pub fn plans(&self) -> &[CallablePlan<'a>] {
        &self.plans
    }

    pub fn lookup(&self, service: &str, operation: &str) -> Option<&CallablePlan<'a>> {
        self.plans
            .iter()
            .find(|plan| plan.service_name() == service && plan.operation_name() == operation)
    }

    pub fn schemas(&self) -> &[Arc<KindSchema>] {
        &self.schemas
    }

    pub fn name_paths(&self) -> &[KindNamePath] {
        &self.name_paths
    }

    pub fn relationships(&self) -> &[RelationshipPlan] {
        &self.relations
    }

    pub fn coverage(&self) -> &[KindCoverage] {
        &self.coverage
    }
    pub fn decisions(&self) -> &[ReviewDecision] {
        &self.decisions
    }

    pub fn raw_bytes(&self) -> &'a [u8] {
        self.bytes
    }
}
