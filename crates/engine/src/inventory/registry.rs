use super::{
    MAX_CATALOG_BYTES, RuntimeCatalog, SchemaRegistry, StoreError, relation_permissions,
    schema_sets_match,
};
use inventory_core::inventory::{KindSchema, ScopeKey};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, sync::Arc};
impl SchemaRegistry {
    pub fn new(catalog: &[u8], schemas: Vec<Arc<KindSchema>>) -> Result<Self, StoreError> {
        if catalog.is_empty() || catalog.len() > MAX_CATALOG_BYTES || schemas.is_empty() {
            return Err(StoreError::Invalid("catalog size or kind schema count"));
        }
        let runtime = RuntimeCatalog::from_bytes(catalog)?;
        let operations = operations(&runtime);
        let permissions = relation_permissions::Permissions::new(catalog, &runtime)?;
        let registry = Self::stored(
            Sha256::digest(catalog).into(),
            schemas,
            operations,
            permissions,
        )?;
        if !schema_sets_match(&registry.schemas, runtime.schemas()) {
            return Err(StoreError::Invalid(
                "kind schemas differ from checked runtime catalog",
            ));
        }
        Ok(registry)
    }

    pub(super) fn stored(
        digest: [u8; 32],
        schemas: Vec<Arc<KindSchema>>,
        operations: BTreeSet<(String, String)>,
        permissions: relation_permissions::Permissions,
    ) -> Result<Self, StoreError> {
        let mut ordered = schemas;
        ordered.sort_unstable_by(|left, right| left.kind().cmp(right.kind()));
        if ordered.is_empty()
            || ordered
                .windows(2)
                .any(|pair| matches!(pair, [left, right] if left.kind() == right.kind()))
        {
            return Err(StoreError::Invalid("missing or duplicate kind schema"));
        }
        Ok(Self {
            schemas: ordered.into(),
            digest,
            operations: Arc::new(operations),
            permissions,
        })
    }

    pub fn schemas(&self) -> &[Arc<KindSchema>] {
        &self.schemas
    }
    pub(super) fn reference_target(
        &self,
        account: inventory_core::inventory::AccountId,
        region: inventory_core::inventory::Region,
        kind: inventory_core::inventory::Kind,
        path: Vec<inventory_core::record::FieldId>,
        value: inventory_core::inventory::ResourceId,
    ) -> Result<inventory_core::inventory::ReferenceTarget, StoreError> {
        if path.is_empty() {
            let schema = self
                .schemas
                .iter()
                .find(|schema| schema.kind() == &kind)
                .ok_or(StoreError::Invalid(
                    "reference target kind is absent from schema registry",
                ))?;
            Ok(inventory_core::inventory::ReferenceTarget::root_text(
                account, region, schema, value,
            )?)
        } else {
            Ok(inventory_core::inventory::ReferenceTarget::new(
                account, region, kind, path, value,
            )?)
        }
    }
    pub fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
    pub(crate) fn operations(&self) -> &BTreeSet<(String, String)> {
        &self.operations
    }
    pub(crate) fn allows(&self, scope: &ScopeKey) -> bool {
        self.operations.contains(&(
            scope.service().as_str().to_owned(),
            scope.operation().as_str().to_owned(),
        ))
    }
}

fn operations(runtime: &RuntimeCatalog) -> BTreeSet<(String, String)> {
    runtime
        .plans()
        .iter()
        .map(|plan| {
            (
                plan.service_name().to_owned(),
                plan.operation_name().to_owned(),
            )
        })
        .collect()
}
