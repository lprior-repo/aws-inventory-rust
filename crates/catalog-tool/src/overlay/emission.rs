use super::{Identity, ItemSelector, KindClassification, MapKeyExclusion, MemberPath, ResourceProjection};

/// One additional resource selection from the same modeled HTTP response.
/// Its reviewed key is resolved through the central kind map, never a fake operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceEmission {
    pub key: String,
    pub items: ItemSelector,
    pub identity: Identity,
    pub projection: Option<ResourceProjection>,
    pub classification: Option<KindClassification>,
    pub exclude_from_hash: Vec<MemberPath>,
    pub exclude_map_keys: Vec<MapKeyExclusion>,
    pub queryable: Vec<MemberPath>,
    pub tags: Option<MemberPath>,
}

impl ResourceEmission {
    pub fn new(key: impl Into<String>, items: ItemSelector, identity: Identity) -> Self {
        Self {
            key: key.into(), items, identity, projection: None, classification: None,
            exclude_from_hash: Vec::new(), exclude_map_keys: Vec::new(),
            queryable: Vec::new(), tags: None,
        }
    }
}

/// The primary resource plus additional reviewed selections are statically bounded.
pub const MAX_RESOURCE_EMISSIONS: usize = 8;
