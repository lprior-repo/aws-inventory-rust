use super::selection::Reference;
use crate::inventory::relation_permissions::provenance::proof;
use crate::inventory::{SchemaRegistry, StoreError};
use futures_util::TryStreamExt;
use inventory_core::inventory::ResourceId;
use sqlx::{Postgres, Row, Transaction};

const GRANTS: &str = "SELECT DISTINCT i.ordinal,e.target_id FROM UNNEST($1::bytea[]) WITH ORDINALITY i(proof,ordinal) JOIN reviewed_reference_evidence e ON e.proof=i.proof JOIN resource_facets f ON (f.account,f.region,f.kind,f.id,f.source_key,f.source_authority)=(e.from_account,e.from_region,e.from_kind,e.from_id,e.source_facet,e.source_authority) WHERE e.authority IS NOT NULL ORDER BY i.ordinal,e.target_id";

pub(super) async fn load(
    tx: &mut Transaction<'_, Postgres>,
    registry: &SchemaRegistry,
    references: &mut [Reference],
) -> Result<(), StoreError> {
    let proofs: Vec<_> = references
        .iter()
        .map(|row| proof(registry, &row.value))
        .collect();
    let bytes: Vec<_> = proofs.iter().map(|proof| proof.as_slice()).collect();
    let mut rows = sqlx::query(GRANTS).bind(bytes).fetch(&mut **tx);
    while let Some(row) = rows.try_next().await? {
        let index = ordinal(&row)?;
        let reference = references
            .get_mut(index)
            .ok_or(StoreError::Invalid("reference index exceeds bounds"))?;
        insert(reference, row.try_get("target_id")?)?;
    }
    for reference in references.iter().filter(|row| !row.grants.is_empty()) {
        registry.permissions.validate_reference(&reference.value)?;
    }
    Ok(())
}

fn ordinal(row: &sqlx::postgres::PgRow) -> Result<usize, StoreError> {
    usize::try_from(row.try_get::<i64, _>("ordinal")?)
        .ok()
        .and_then(|index| index.checked_sub(1))
        .ok_or(StoreError::Invalid("reference index exceeds bounds"))
}

fn insert(reference: &mut Reference, target: Option<&str>) -> Result<(), StoreError> {
    let target = target.map(ResourceId::parse).transpose()?;
    reference.grants.insert(target);
    if reference.grants.len() > 4096 {
        return Err(StoreError::Invalid("reference grant count exceeds bounds"));
    }
    Ok(())
}
