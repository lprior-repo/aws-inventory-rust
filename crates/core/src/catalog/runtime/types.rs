mod plan;
use crate::{inventory::Kind, record::FieldId};
pub use plan::CallablePlan;
pub(crate) use plan::{EndpointTemplate, PlanParts};

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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationshipPlan {
    pub source_kind: Kind,
    pub source_path: Box<[FieldId]>,
    pub target_kind: Kind,
    pub target_path: Box<[FieldId]>,
    pub relation: crate::inventory::Relation,
    pub scope: RelationScope,
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ItemPlan {
    Root,
    Path(Box<[FieldId]>),
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Strings(Box<[String]>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingSource {
    Account,
    Region,
    Fixed(BindingValue),
    ParentIdentity {
        path: Box<[FieldId]>,
    },
    Operation {
        service: Box<str>,
        operation: Box<str>,
        path: Box<[FieldId]>,
    },
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenBinding {
    pub request_path: Box<[FieldId]>,
    pub response_path: Box<[FieldId]>,
    pub value_type: TokenType,
    pub request_placement: TokenPlacement,
    pub response_placement: TokenPlacement,
    pub optional: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageSizeBinding {
    pub path: Box<[FieldId]>,
    pub max: u32,
    pub placement: TokenPlacement,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PagePlan {
    pub items: ItemPlan,
    pub tokens: Box<[TokenBinding]>,
    pub page_size: Option<PageSizeBinding>,
    pub max_page_size: Option<u32>,
    pub truncated_path: Option<Box<[FieldId]>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewProvenance {
    pub source: String,
    pub reviewer: String,
    pub reviewed_on: String,
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
