use super::facets::Context;
use crate::inventory::relation_permissions::{SourceAuthority, provenance::proof};
use crate::inventory::{SchemaRegistry, StoreError};
use inventory_core::inventory::{Observation, ResourceId, UnresolvedReference};
use sqlx::{Postgres, Transaction};

pub(super) struct Grant<'a> {
    proof: [u8; 32],
    target: Option<&'a ResourceId>,
    authority: [u8; 32],
    source_authority: [u8; 32],
}

pub(super) fn collect<'a>(
    context: &Context<'a>,
    observation: &Observation,
    authority: &SourceAuthority<'_, '_>,
    grants: &mut Vec<Grant<'a>>,
) -> Result<(), StoreError> {
    for (reference, target) in context.references(observation.key()) {
        grants.push(grant(context.registry, reference, target, authority)?);
    }
    Ok(())
}

fn grant<'a>(
    registry: &SchemaRegistry,
    reference: &UnresolvedReference,
    target: Option<&'a ResourceId>,
    authority: &SourceAuthority<'_, '_>,
) -> Result<Grant<'a>, StoreError> {
    Ok(Grant {
        proof: proof(registry, reference),
        target,
        authority: authority.reference_authority(reference)?,
        source_authority: *authority.source_authority(),
    })
}

pub(super) async fn persist(
    tx: &mut Transaction<'_, Postgres>,
    grants: &[Grant<'_>],
    source: &[u8],
) -> Result<(), StoreError> {
    if !grants.is_empty() {
        query(grants, source).execute(&mut **tx).await?;
    }
    Ok(())
}

fn query(
    grants: &[Grant<'_>],
    source: &[u8],
) -> sqlx::query::Query<'static, Postgres, sqlx::postgres::PgArguments> {
    let proofs: Vec<_> = grants.iter().map(|g| g.proof.as_slice()).collect();
    let targets: Vec<_> = grants
        .iter()
        .map(|g| g.target.map(ResourceId::as_str))
        .collect();
    let authorities: Vec<_> = grants.iter().map(|g| g.authority.as_slice()).collect();
    let sources: Vec<_> = grants
        .iter()
        .map(|g| g.source_authority.as_slice())
        .collect();
    sqlx::query("INSERT INTO reviewed_reference_grants(proof,target_id,authority,source_facet,source_authority) SELECT p,t,a,$5,s FROM UNNEST($1::bytea[],$2::text[],$3::bytea[],$4::bytea[]) u(p,t,a,s) ON CONFLICT DO NOTHING")
        .bind(proofs)
        .bind(targets)
        .bind(authorities)
        .bind(sources)
        .bind(source)
}
