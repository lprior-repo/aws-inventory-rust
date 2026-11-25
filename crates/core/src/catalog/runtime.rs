mod decode;
mod index;
mod service_selection;
mod tag_endpoint;
mod tag_sources;
mod types;
pub use tag_endpoint::{
    TagEndpointParentPlan, TagEndpointPlan, TagEndpointRequestPlan, TagEndpointSource,
};
pub use tag_sources::kinds_without_tag_source;

use crate::{
    catalog::{Catalog, CatalogError},
    inventory::{InventoryError, KindSchema},
    record::RecordError,
};
use std::sync::Arc;
pub use types::{
    ArnOwnerRule, ArnRegionRule, ArnResourceKind, BindingAlternativePlan, BindingPlan,
    BindingSource, BindingTransform, BindingValue, CallablePlan, CatalogSourceProvenance,
    ChildEligibilityPlan, ContextProjectionPlan, ContextSource, Emission, EndpointClass,
    EndpointPathMode, ExecutionBranch, IdentityBranchPlan, IdentityChoicePlan, IdentityFieldPlan,
    IdentityLeafPlan, IdentityPart, IdentityPlan, IdentityPredicatePlan, IdentityTextOperand,
    IdentityTextPlan, ItemPlan, KindClassificationPlan, KindCoverage, KindNamePath,
    MissingSelected, ModelSourceProvenance, PagePlan, PageSizeBinding, PageSizeType, PlanPurpose,
    PlanScope, ReferenceOwnerRule, RelationGuard, RelationPathMode, RelationScope, RelationSource,
    RelationTargetMode, RelationshipComponentPlan, RelationshipPlan, RequestRouting,
    ResolvedEndpoint, ResourceEmissionPlan, ResourceEmissions, ResourceProjectionPlan,
    ReviewDecision, ReviewDisposition, ReviewEvidence, ReviewProvenance, RowOperandPlan,
    RowPolicyPlan, RowPredicatePlan, RowValueType, RuntimeProtocol, SelectionKind, TagPlan,
    TokenBinding, TokenPlacement, TokenSelection, TokenType, UrlResourceKind,
};

const MAX_PLANS: usize = 65_536;

/// Hashes the canonical compiled capability fields without including their source receipt.
pub fn compiled_inputs_sha256(
    extension: crate::record::MapRef<'_>,
) -> Result<[u8; 32], RuntimeCatalogError> {
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
    #[error("unknown callable service or operation selector: {0}")]
    Selection(String),
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
    index: index::Index,
}
impl<'a> RuntimeCatalog<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuntimeCatalogError> {
        let catalog = Catalog::borrow(bytes)?;
        let decoded = decode::read(bytes, catalog)?;
        if decoded.plans.len() > MAX_PLANS {
            return Err(RuntimeCatalogError::Bounds);
        }
        let plans = decoded.plans.into_boxed_slice();
        let name_paths = decoded.name_paths.into_boxed_slice();
        let relations = decoded.relations.into_boxed_slice();
        let index = index::Index::new(&plans, &name_paths, &relations)?;
        Ok(Self {
            bytes,
            catalog,
            plans,
            schemas: decoded.schemas.into_boxed_slice(),
            name_paths,
            relations,
            coverage: decoded.coverage.into_boxed_slice(),
            decisions: decoded.decisions.into_boxed_slice(),
            provenance: decoded.provenance,
            index,
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
        self.index.lookup(&self.plans, service, operation)
    }

    /// Returns the position of a reviewed operation in this catalog's plans.
    pub fn plan_index(&self, service: &str, operation: &str) -> Option<usize> {
        self.index.lookup_index(&self.plans, service, operation)
    }

    pub fn schemas(&self) -> &[Arc<KindSchema>] {
        &self.schemas
    }

    pub fn name_paths(&self) -> &[KindNamePath] {
        &self.name_paths
    }

    pub fn name_path(&self, kind: &crate::inventory::Kind) -> Option<&KindNamePath> {
        self.index.name_path(&self.name_paths, kind)
    }

    pub fn relationships(&self) -> &[RelationshipPlan] {
        &self.relations
    }

    pub fn relationships_from<'s>(
        &'s self,
        kind: &crate::inventory::Kind,
    ) -> impl Iterator<Item = &'s RelationshipPlan> + 's {
        self.index.relationships_from(&self.relations, kind)
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
