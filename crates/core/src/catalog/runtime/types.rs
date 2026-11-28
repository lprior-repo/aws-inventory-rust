mod capabilities;
mod emission;
mod emissions;
mod fixed_input;
mod identity_text;
mod plan;
mod provenance;
mod routing;
mod row_policy;
mod selection;
use crate::{inventory::Kind, record::FieldId};
pub use capabilities::{
    ArnOwnerRule, ArnRegionRule, KindClassificationPlan, ReferenceOwnerRule, ResourceProjectionPlan,
};
pub use emission::ResourceEmissionPlan;
pub use emissions::ResourceEmissions;
pub use fixed_input::BindingValue;
pub(crate) use identity_text::IdentityPatternCache;
pub use identity_text::{IdentityTextOperand, IdentityTextPlan};
pub use plan::CallablePlan;
pub(crate) use plan::{EndpointException, EndpointTemplate, PlanParts};
pub use provenance::{CatalogSourceProvenance, ModelSourceProvenance};
pub use routing::{ExecutionBranch, RequestRouting};
pub use row_policy::{
    ChildEligibilityPlan, MissingSelected, RowOperandPlan, RowPolicyPlan, RowPredicatePlan,
    RowValueType,
};
pub use selection::{
    BindingAlternativePlan, ContextProjectionPlan, ContextSource, Emission, IdentityBranchPlan,
    IdentityChoicePlan, IdentityFieldPlan, IdentityLeafPlan, IdentityPredicatePlan, SelectionKind,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanScope {
    Region,
    Account,
    HomeRegion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KindCoverage {
    pub cfn_type: Option<String>,
    pub kind: Option<Kind>,
    pub mapped: bool,
    pub reason: Option<String>,
    pub api_only: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KindNamePath {
    pub kind: Kind,
    pub path: Box<[FieldId]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationScope {
    AccountRegion,
    Account,
    HomeRegion,
    ArnRegion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationSource {
    SelectedOutput,
    Request,
    Parent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationPathMode {
    Fields,
    RootText,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationTargetMode {
    SelectedFields,
    SelectedRootText,
    BoundContextFields,
    BoundContextRootText,
    CanonicalId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationGuard {
    pub source: RelationSource,
    pub path_mode: RelationPathMode,
    pub path: Box<[FieldId]>,
    pub equals: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationshipComponentPlan {
    pub source: RelationSource,
    pub source_path: Box<[FieldId]>,
    pub source_path_mode: RelationPathMode,
    pub target_mode: RelationTargetMode,
    pub target_path: Box<[FieldId]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationshipPlan {
    pub source_kind: Kind,
    pub source: RelationSource,
    pub edge_source: crate::inventory::EdgeSource,
    pub source_path: Box<[FieldId]>,
    pub source_path_mode: RelationPathMode,
    pub target_kind: Kind,
    pub target_kinds: Box<[Kind]>,
    pub target_path: Box<[FieldId]>,
    pub relation: crate::inventory::Relation,
    pub target_mode: RelationTargetMode,
    pub guard: Option<RelationGuard>,
    pub components: Box<[RelationshipComponentPlan]>,
    pub scope: RelationScope,
    pub owner: ReferenceOwnerRule,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeProtocol {
    AwsJson10,
    AwsJson11,
    RpcV2Cbor,
    AwsQuery,
    Ec2Query,
    RestJson1,
    RestXml,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanPurpose {
    Enumeration,
    AccountIdentity {
        account_path: Box<[FieldId]>,
    },
    RegionDiscovery {
        regions_path: Box<[FieldId]>,
        region_name_path: Box<[FieldId]>,
        opt_in_status_path: Option<Box<[FieldId]>>,
    },
    EndpointDiscovery,
    TagEndpoint(super::TagEndpointPlan),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ItemPlan {
    Root,
    Path(Box<[FieldId]>),
    Select {
        path: Box<[FieldId]>,
        kind: SelectionKind,
    },
    Nested {
        parents: Box<[FieldId]>,
        items: FieldId,
        owner: FieldId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentityPart {
    Literal(String),
    Field(Box<[FieldId]>),
    Partition,
    Account,
    Region,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentityPlan {
    Arn(Box<[FieldId]>),
    Template(Box<[IdentityPart]>),
    Key(Box<[Box<[FieldId]>]>),
    Choice(IdentityChoicePlan),
    CorrelatedKey(Box<[IdentityFieldPlan]>),
    MapArn {
        path: Box<[FieldId]>,
        key: String,
        resource_kind: ArnResourceKind,
        request_url: Option<Box<[FieldId]>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TagPlan {
    List {
        path: Box<[FieldId]>,
        key: FieldId,
        value: FieldId,
    },
    Map {
        path: Box<[FieldId]>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingSource {
    Account,
    Region,
    RegionList,
    Alternatives(Box<[BindingAlternativePlan]>),
    ContextFromBranch {
        service: Box<str>,
        operation: Box<str>,
        branch: Box<str>,
        source: ContextSource,
        path: Box<[FieldId]>,
    },
    Fixed(BindingValue),
    ParentIdentity {
        path: Box<[FieldId]>,
    },
    Operation {
        service: Box<str>,
        operation: Box<str>,
        path: Box<[FieldId]>,
    },
    OperationArn {
        service: Box<str>,
        operation: Box<str>,
        path: Box<[FieldId]>,
        arn_service: Box<str>,
        resource_kind: ArnResourceKind,
    },
    OperationUrl {
        service: Box<str>,
        operation: Box<str>,
        path: Box<[FieldId]>,
        resource_kind: UrlResourceKind,
    },
    Context {
        service: Box<str>,
        operation: Box<str>,
        source: ContextSource,
        path: Box<[FieldId]>,
    },
    Transformed {
        source: Box<BindingSource>,
        transform: BindingTransform,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArnResourceKind {
    SnsSubscription,
    SqsQueue,
    SnsTopic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UrlResourceKind {
    SqsQueue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingTransform {
    StripPrefix(Box<str>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingPlan {
    pub path: Box<[FieldId]>,
    pub source: BindingSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenType {
    String,
    Integer,
    Boolean,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenPlacement {
    Path(String),
    Query(String),
    Header(String),
    Body(Box<[FieldId]>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenSelection {
    Value,
    LastListItem,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenBinding {
    pub request_path: Box<[FieldId]>,
    pub response_path: Box<[FieldId]>,
    pub value_type: TokenType,
    pub request_placement: TokenPlacement,
    pub response_placement: TokenPlacement,
    pub optional: bool,
    pub selection: TokenSelection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageSizeType {
    String,
    Integer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageSizeBinding {
    pub path: Box<[FieldId]>,
    pub max: u32,
    pub value_type: PageSizeType,
    pub placement: TokenPlacement,
}

impl PageSizeBinding {
    pub fn value(&self) -> crate::record::Value {
        self.capped(None)
    }

    pub fn capped(&self, cap: Option<u32>) -> crate::record::Value {
        let size = cap.map_or(self.max, |cap| self.max.min(cap));
        match self.value_type {
            PageSizeType::String => crate::record::Value::Text(size.to_string()),
            PageSizeType::Integer => crate::record::Value::Int(i64::from(size)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PagePlan {
    pub items: ItemPlan,
    pub tokens: Box<[TokenBinding]>,
    pub page_size: Option<PageSizeBinding>,
    pub max_page_size: Option<u32>,
    pub truncated_path: Option<Box<[FieldId]>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReviewEvidence {
    pub machine_reviewed: bool,
    pub manually_reviewed: bool,
    pub fixture_tested: bool,
    pub emulator_tested: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewProvenance {
    pub source: String,
    pub reviewer: String,
    pub reviewed_on: String,
    pub tool_version: String,
    pub evidence: ReviewEvidence,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewDecision {
    pub service: String,
    pub operation: String,
    pub disposition: ReviewDisposition,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReviewDisposition {
    Callable,
    Denied { reason: String },
    Deferred { reason: String },
    Unclassified,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndpointClass {
    Global,
    Regional,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndpointPathMode {
    Standard,
    S3,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedEndpoint {
    pub url: String,
    pub signing_region: String,
    pub signing_name: String,
    pub class: EndpointClass,
    pub path_mode: EndpointPathMode,
}
