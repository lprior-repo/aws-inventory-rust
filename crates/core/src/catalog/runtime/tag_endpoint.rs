use super::TagPlan;
use crate::record::FieldId;

/// A single-owner endpoint whose resource key comes exclusively from durable parent work.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagEndpointPlan {
    pub parents: Box<[TagEndpointParentPlan]>,
    pub tags: TagPlan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagEndpointParentPlan {
    pub operation: Box<str>,
    pub request: Option<TagEndpointRequestPlan>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagEndpointRequestPlan {
    pub input: Box<[FieldId]>,
    pub parent: Box<[FieldId]>,
    pub source: TagEndpointSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagEndpointSource {
    Response,
    Request,
    OwnerIdentity,
}
