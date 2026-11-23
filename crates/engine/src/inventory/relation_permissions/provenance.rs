mod schema;
pub(in crate::inventory) use schema::{initialize, invalidate_prior_authority};

use super::{Permissions, StoreError, source_authority::SourceAuthority};
use crate::inventory::SchemaRegistry;
use inventory_core::inventory::{ResourceId, ResourceKey, UnresolvedReference};
use sha2::{Digest, Sha256};
use sqlx::{Row, Sqlite, Transaction};
use std::sync::Arc;

/// The digest is carried by the strict codec; the facet identity comes only from
/// the owning SQL row, never from an untrusted relation tuple.
pub(in crate::inventory) struct ReferenceProvenance {
    pub(in crate::inventory) authority: [u8; 32],
    pub(in crate::inventory) source_facet: Arc<[u8]>,
    pub(in crate::inventory) source_authority: [u8; 32],
}

pub(in crate::inventory) async fn mark_reference(
    authority: &SourceAuthority<'_, '_>,
    tx: &mut Transaction<'_, Sqlite>,
    reference: &UnresolvedReference,
) -> Result<(), StoreError> {
    mark(authority, tx, reference, None).await
}

pub(in crate::inventory) async fn mark_direct_reference(
    authority: &SourceAuthority<'_, '_>,
    tx: &mut Transaction<'_, Sqlite>,
    reference: &UnresolvedReference,
    target: &ResourceId,
) -> Result<(), StoreError> {
    if reference.target.candidate_kinds().len() != 1 {
        return Err(super::unreviewed());
    }
    mark(authority, tx, reference, Some(target)).await
}

async fn mark(
    authority: &SourceAuthority<'_, '_>,
    tx: &mut Transaction<'_, Sqlite>,
    reference: &UnresolvedReference,
    target: Option<&ResourceId>,
) -> Result<(), StoreError> {
    let evidence = authority.reference_authority(reference)?;
    sqlx::query("INSERT OR IGNORE INTO reviewed_reference_grants(proof,target_id,authority,source_facet,source_authority) VALUES(?1,?2,?3,?4,?5)")
        .bind(proof(authority.registry(), reference).as_slice()).bind(target.map(ResourceId::as_str))
        .bind(evidence.as_slice()).bind(authority.source_facet()).bind(authority.source_authority().as_slice()).execute(&mut **tx).await?;
    Ok(())
}

pub(in crate::inventory) async fn activate_reference(
    registry: &SchemaRegistry,
    tx: &mut Transaction<'_, Sqlite>,
    reference: &UnresolvedReference,
    target: Option<&ResourceId>,
    provenance: &ReferenceProvenance,
) -> Result<(), StoreError> {
    registry.permissions.validate_reference(reference)?;
    sqlx::query("INSERT OR IGNORE INTO reviewed_reference_evidence(proof,from_account,from_region,from_kind,from_id,target_id,authority,source_facet,source_authority) SELECT proof,?2,?3,?4,?5,target_id,authority,source_facet,source_authority FROM reviewed_reference_grants WHERE proof=?1 AND target_id IS ?6 AND authority=?7 AND source_facet=?8 AND source_authority=?9 AND EXISTS(SELECT 1 FROM resource_facets WHERE account=?2 AND region=?3 AND kind=?4 AND id=?5 AND source_key=?8 AND source_authority=?9)")
        .bind(proof(registry, reference).as_slice())
        .bind(reference.from.account.as_str()).bind(reference.from.region.as_str()).bind(reference.from.kind.as_str()).bind(reference.from.id.as_str())
        .bind(target.map(ResourceId::as_str)).bind(provenance.authority.as_slice()).bind(provenance.source_facet.as_ref())
        .bind(provenance.source_authority.as_slice()).execute(&mut **tx).await?;
    Ok(())
}

pub(in crate::inventory) async fn clear_active(
    tx: &mut Transaction<'_, Sqlite>,
    source: &ResourceKey,
) -> Result<(), StoreError> {
    sqlx::query("DELETE FROM reviewed_reference_evidence WHERE from_account=?1 AND from_region=?2 AND from_kind=?3 AND from_id=?4 AND authority IS NOT NULL")
        .bind(source.account.as_str()).bind(source.region.as_str()).bind(source.kind.as_str()).bind(source.id.as_str()).execute(&mut **tx).await?;
    Ok(())
}

pub(in crate::inventory) async fn promotable(
    registry: &SchemaRegistry,
    tx: &mut Transaction<'_, Sqlite>,
    reference: &UnresolvedReference,
) -> Result<bool, StoreError> {
    if crate::inventory::legacy::contains(tx, &reference.from).await? {
        return Ok(false);
    }
    if matches!(registry.permissions, Permissions::Unavailable) {
        return Ok(false);
    }
    let present: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM reviewed_reference_evidence e WHERE proof=?1 AND authority IS NOT NULL AND source_facet IS NOT NULL AND source_authority IS NOT NULL AND EXISTS(SELECT 1 FROM resource_facets f WHERE f.account=e.from_account AND f.region=e.from_region AND f.kind=e.from_kind AND f.id=e.from_id AND f.source_key=e.source_facet AND f.source_authority=e.source_authority))")
        .bind(proof(registry, reference).as_slice()).fetch_one(&mut **tx).await?;
    Ok(present)
}

pub(in crate::inventory) async fn targets(
    registry: &SchemaRegistry,
    tx: &mut Transaction<'_, Sqlite>,
    reference: &UnresolvedReference,
) -> Result<Vec<Option<ResourceId>>, StoreError> {
    let rows = sqlx::query("SELECT DISTINCT target_id FROM reviewed_reference_evidence e WHERE proof=?1 AND authority IS NOT NULL AND source_facet IS NOT NULL AND source_authority IS NOT NULL AND EXISTS(SELECT 1 FROM resource_facets f WHERE f.account=e.from_account AND f.region=e.from_region AND f.kind=e.from_kind AND f.id=e.from_id AND f.source_key=e.source_facet AND f.source_authority=e.source_authority) ORDER BY target_id")
        .bind(proof(registry, reference).as_slice()).fetch_all(&mut **tx).await?;
    rows.into_iter()
        .map(|row| {
            row.try_get::<Option<String>, _>("target_id")
                .map_err(StoreError::from)
                .and_then(|target| {
                    target
                        .map(|target| ResourceId::parse(&target))
                        .transpose()
                        .map_err(StoreError::from)
                })
        })
        .collect()
}

pub(in crate::inventory) fn proof(
    registry: &SchemaRegistry,
    reference: &UnresolvedReference,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"aws-inventory-reviewed-reference-v3");
    hash.update(registry.digest());
    for text in [
        reference.from.account.as_str(),
        reference.from.region.as_str(),
        reference.from.kind.as_str(),
        reference.from.id.as_str(),
        reference.target.account().as_str(),
        reference.target.region().as_str(),
        reference.target.kind().as_str(),
        reference.target.value().as_str(),
        reference.relation.as_str(),
        reference.source.as_str(),
    ] {
        hash.update(text.len().to_le_bytes());
        hash.update(text.as_bytes());
    }
    hash.update(reference.target.path().len().to_le_bytes());
    for field in reference.target.path() {
        hash.update(field.value().to_le_bytes());
    }
    hash.update(reference.target.candidate_kinds().len().to_le_bytes());
    for kind in reference.target.candidate_kinds() {
        hash.update(kind.as_str().len().to_le_bytes());
        hash.update(kind.as_str().as_bytes());
    }
    if !reference.target.components().is_empty()
        || !matches!(
            reference.target.mode(),
            inventory_core::catalog::runtime::RelationTargetMode::SelectedFields
                | inventory_core::catalog::runtime::RelationTargetMode::SelectedRootText
        )
    {
        hash.update(b"typed-reference-v4");
        let target = &reference.target;
        for (mode, path, value) in std::iter::once((target.mode(), target.path(), target.value()))
            .chain(
                target
                    .components()
                    .iter()
                    .map(|term| (term.mode, term.path.as_ref(), &term.value)),
            )
        {
            hash.update([crate::inventory::reference_candidates::mode_code(mode)]);
            hash.update(path.len().to_le_bytes());
            for field in path {
                hash.update(field.value().to_le_bytes());
            }
            hash.update(value.as_str().len().to_le_bytes());
            hash.update(value.as_str().as_bytes());
        }
    }
    hash.finalize().into()
}
