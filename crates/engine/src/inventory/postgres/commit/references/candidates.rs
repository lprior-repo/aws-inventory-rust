use super::{Row, storage};
use crate::inventory::StoreError;
use sqlx::{Postgres, QueryBuilder, Transaction};

pub(super) async fn persist(
    tx: &mut Transaction<'_, Postgres>,
    rows: &[Row<'_>],
) -> Result<(), StoreError> {
    for batch in rows.chunks(1000) {
        let (ordinals, kinds) = arrays(batch)?;
        let mut query = QueryBuilder::new("WITH input AS (SELECT * FROM ");
        storage::input(&mut query, batch);
        query
            .push("), candidates AS (SELECT * FROM UNNEST(")
            .push_bind(ordinals)
            .push("::bigint[],");
        query.push_bind(kinds).push("::text[]) u(ordinal,candidate_kind)) INSERT INTO unresolved_ref_candidates(from_account,from_region,from_kind,from_id,target_scope,target_kind,target_path,target_value,relation,source,candidate_kind) SELECT i.a,i.r,i.k,i.id,i.s,i.tk,i.p,i.v,i.rel,i.src,c.candidate_kind FROM input i JOIN candidates c USING(ordinal) JOIN unresolved_refs r ON r.from_account=i.a AND r.from_region=i.r AND r.from_kind=i.k AND r.from_id=i.id AND r.target_scope=i.s AND r.target_kind=i.tk AND r.target_path=i.p AND r.target_value=i.v AND r.relation=i.rel AND r.source=i.src AND r.target_kinds=i.ks ON CONFLICT DO NOTHING");
        query.build().execute(&mut **tx).await?;
    }
    Ok(())
}

fn arrays<'a>(rows: &'a [Row<'_>]) -> Result<(Vec<i64>, Vec<&'a str>), StoreError> {
    let capacity = capacity(rows)?;
    let mut ordinals = Vec::with_capacity(capacity);
    let mut kinds = Vec::with_capacity(capacity);
    for (index, row) in rows.iter().enumerate() {
        let ordinal = i64::try_from(index)
            .ok()
            .and_then(|value| value.checked_add(1))
            .ok_or(StoreError::Invalid("candidate ordinal overflow"))?;
        for kind in row.reference.target.candidate_kinds() {
            ordinals.push(ordinal);
            kinds.push(kind.as_str());
        }
    }
    Ok((ordinals, kinds))
}

fn capacity(rows: &[Row<'_>]) -> Result<usize, StoreError> {
    rows.iter().try_fold(0_usize, |total, row| {
        total
            .checked_add(row.reference.target.candidate_kinds().len())
            .ok_or(StoreError::Invalid("candidate count overflow"))
    })
}
