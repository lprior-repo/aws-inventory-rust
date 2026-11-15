use super::{
    AccountId, EdgeSource, InventoryError, Kind, Region, Relation, ResourceId, ResourceKey,
};
use crate::record::FieldId;

#[derive(Debug)]
pub struct ReferenceTarget {
    account: AccountId,
    region: Region,
    kind: Kind,
    path: Box<[FieldId]>,
    value: ResourceId,
}

impl ReferenceTarget {
    pub fn new(
        account: AccountId,
        region: Region,
        kind: Kind,
        path: Vec<FieldId>,
        value: ResourceId,
    ) -> Result<Self, InventoryError> {
        if path.is_empty() || path.len() > 8 {
            return Err(InventoryError::Path);
        }
        Ok(Self {
            account,
            region,
            kind,
            path: path.into_boxed_slice(),
            value,
        })
    }

    pub fn account(&self) -> &AccountId {
        &self.account
    }
    pub fn region(&self) -> &Region {
        &self.region
    }
    pub fn kind(&self) -> &Kind {
        &self.kind
    }
    pub fn path(&self) -> &[FieldId] {
        &self.path
    }
    pub fn value(&self) -> &ResourceId {
        &self.value
    }
}

#[derive(Debug)]
pub struct UnresolvedReference {
    pub from: ResourceKey,
    pub relation: Relation,
    pub source: EdgeSource,
    pub target: ReferenceTarget,
}
