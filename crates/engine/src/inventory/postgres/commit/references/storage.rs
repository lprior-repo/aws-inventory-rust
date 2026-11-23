use super::Row;
use crate::inventory::StoreError;
use crate::inventory::postgres::commit::keys::Keys;
use sqlx::{Postgres, QueryBuilder, Transaction};

pub(super) async fn persist(
    tx: &mut Transaction<'_, Postgres>,
    rows: &[Row<'_>],
) -> Result<(), StoreError> {
    for batch in rows.chunks(1000) {
        let mut query = QueryBuilder::new("WITH input AS (SELECT * FROM ");
        input(&mut query, batch);
        query.push(") INSERT INTO unresolved_refs(from_account,from_region,from_kind,from_id,target_scope,target_kind,target_path,target_value,relation,source,target_kinds) SELECT DISTINCT ON (a,r,k,id,s,tk,p,v,rel,src) a,r,k,id,s,tk,p,v,rel,src,ks FROM input ORDER BY a,r,k,id,s,tk,p,v,rel,src,ordinal DESC ON CONFLICT(from_account,from_region,from_kind,from_id,target_scope,target_kind,target_path,target_value,relation,source) DO UPDATE SET target_kinds=EXCLUDED.target_kinds");
        query.build().execute(&mut **tx).await?;
    }
    Ok(())
}

pub(super) fn input(query: &mut QueryBuilder<Postgres>, rows: &[Row<'_>]) {
    query.push("UNNEST(");
    Keys::new(rows.iter().map(|row| &row.reference.from)).bind_arrays(query);
    target(query, rows);
    details(query, rows);
    query.push(") WITH ORDINALITY u(a,r,k,id,s,tk,p,v,rel,src,ks,ordinal)");
}

fn target(query: &mut QueryBuilder<Postgres>, rows: &[Row<'_>]) {
    let scopes: Vec<_> = rows.iter().map(|row| row.scope.as_slice()).collect();
    let kinds: Vec<_> = rows
        .iter()
        .map(|row| row.reference.target.kind().as_str())
        .collect();
    let paths: Vec<_> = rows.iter().map(|row| row.path.as_slice()).collect();
    query.push(",").push_bind(scopes).push("::bytea[],");
    query.push_bind(kinds).push("::text[],");
    query.push_bind(paths).push("::bytea[],");
}

fn details(query: &mut QueryBuilder<Postgres>, rows: &[Row<'_>]) {
    let values: Vec<_> = rows
        .iter()
        .map(|row| row.reference.target.value().as_str())
        .collect();
    let relations: Vec<_> = rows
        .iter()
        .map(|row| row.reference.relation.as_str())
        .collect();
    let sources: Vec<_> = rows
        .iter()
        .map(|row| row.reference.source.as_str())
        .collect();
    let kinds: Vec<_> = rows.iter().map(|row| row.kinds.as_slice()).collect();
    query.push_bind(values).push("::text[],");
    query.push_bind(relations).push("::text[],");
    query.push_bind(sources).push("::text[],");
    query.push_bind(kinds).push("::bytea[]");
}
