use super::Row;
use crate::inventory::StoreError;
use crate::inventory::postgres::commit::keys::Keys;
use inventory_core::inventory::ResourceId;
use sqlx::{Postgres, QueryBuilder, Transaction};

pub(super) async fn persist(
    tx: &mut Transaction<'_, Postgres>,
    rows: &[Row<'_>],
) -> Result<(), StoreError> {
    for batch in rows.chunks(1000) {
        let mut query = QueryBuilder::new("WITH input AS (SELECT * FROM UNNEST(");
        Keys::new(batch.iter().map(|row| &row.reference.from)).bind_arrays(&mut query);
        proof(&mut query, batch);
        authority(&mut query, batch);
        query.push(") u(a,r,k,id,p,t,auth,sf,sa)) INSERT INTO reviewed_reference_evidence(proof,from_account,from_region,from_kind,from_id,target_id,authority,source_facet,source_authority) SELECT i.p,i.a,i.r,i.k,i.id,g.target_id,g.authority,g.source_facet,g.source_authority FROM input i JOIN reviewed_reference_grants g ON g.proof=i.p AND g.target_id IS NOT DISTINCT FROM i.t AND g.authority=i.auth AND g.source_facet=i.sf AND g.source_authority=i.sa JOIN resource_facets f ON f.account=i.a AND f.region=i.r AND f.kind=i.k AND f.id=i.id AND f.source_key=i.sf AND f.source_authority=i.sa ON CONFLICT DO NOTHING");
        query.build().execute(&mut **tx).await?;
    }
    Ok(())
}

fn proof(query: &mut QueryBuilder<Postgres>, rows: &[Row<'_>]) {
    let proofs: Vec<_> = rows.iter().map(|row| row.proof.as_slice()).collect();
    let targets: Vec<_> = rows
        .iter()
        .map(|row| row.target.map(ResourceId::as_str))
        .collect();
    query.push(",").push_bind(proofs).push("::bytea[],");
    query.push_bind(targets).push("::text[],");
}

fn authority(query: &mut QueryBuilder<Postgres>, rows: &[Row<'_>]) {
    let authorities: Vec<_> = rows
        .iter()
        .map(|row| row.provenance.authority.as_slice())
        .collect();
    let facets: Vec<_> = rows
        .iter()
        .map(|row| row.provenance.source_facet.as_ref())
        .collect();
    let sources: Vec<_> = rows
        .iter()
        .map(|row| row.provenance.source_authority.as_slice())
        .collect();
    query.push_bind(authorities).push("::bytea[],");
    query.push_bind(facets).push("::bytea[],");
    query.push_bind(sources).push("::bytea[]");
}
