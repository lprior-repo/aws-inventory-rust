use super::{AccountId, InventoryError, Kind, OperationId, Region, ResourceId, ServiceId};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceKey {
    pub account: AccountId,
    pub region: Region,
    pub kind: Kind,
    pub id: ResourceId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ResourceKeyHash([u8; 32]);

impl ResourceKeyHash {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl ResourceKey {
    pub fn digest(&self) -> ResourceKeyHash {
        let mut hash = blake3::Hasher::new();
        hash.update(b"aws-inventory/resource-key/v1\0");
        [
            self.account.as_str(),
            self.region.as_str(),
            self.kind.as_str(),
            self.id.as_str(),
        ]
        .into_iter()
        .for_each(|part| {
            hash.update(part.as_bytes());
            hash.update(b"\0");
        });
        ResourceKeyHash(*hash.finalize().as_bytes())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeKey {
    account: AccountId,
    region: Region,
    service: ServiceId,
    operation: OperationId,
    parent: Option<ResourceKey>,
    branch: Option<String>,
    context: Option<[u8; 32]>,
}

impl ScopeKey {
    pub fn new(
        account: AccountId,
        region: Region,
        service: ServiceId,
        operation: OperationId,
        parent: Option<ResourceKey>,
    ) -> Result<Self, InventoryError> {
        check_parent(&account, parent.as_ref())?;
        Ok(Self {
            account,
            region,
            service,
            operation,
            parent,
            branch: None,
            context: None,
        })
    }

    pub fn account(&self) -> &AccountId {
        &self.account
    }
    pub fn region(&self) -> &Region {
        &self.region
    }
    pub fn service(&self) -> &ServiceId {
        &self.service
    }
    pub fn operation(&self) -> &OperationId {
        &self.operation
    }
    pub fn parent(&self) -> Option<&ResourceKey> {
        self.parent.as_ref()
    }
    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }
    pub fn with_branch(mut self, branch: String) -> Result<Self, InventoryError> {
        if branch.is_empty() || branch.len() > 64
            || !branch.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
        {
            return Err(InventoryError::Branch);
        }
        self.branch = Some(branch);
        Ok(self)
    }
    pub fn context(&self) -> Option<crate::record::ContentHash> {
        self.context.map(crate::record::ContentHash::from_bytes)
    }
    pub fn with_context(mut self, context: crate::record::ContentHash) -> Self {
        self.context = Some(*context.as_bytes());
        self
    }
}

fn check_parent(account: &AccountId, parent: Option<&ResourceKey>) -> Result<(), InventoryError> {
    if parent.is_some_and(|parent| &parent.account != account) {
        return Err(InventoryError::ParentAccount);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EpochMillis(i64);

impl EpochMillis {
    pub fn new(value: i64) -> Result<Self, InventoryError> {
        if value < 0 {
            return Err(InventoryError::Time);
        }
        Ok(Self(value))
    }

    pub fn value(self) -> i64 {
        self.0
    }
}
