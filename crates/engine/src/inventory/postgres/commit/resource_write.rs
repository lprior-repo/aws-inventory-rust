use super::keys::Keys;
use super::resources::{Prepared, State};
use crate::inventory::{PageCommit, StoreError};
use inventory_core::inventory::ResourceKey;
use sqlx::{Postgres, QueryBuilder, Transaction};
use std::collections::BTreeMap;

pub(super) async fn persist(
    tx: &mut Transaction<'_, Postgres>,
    states: &BTreeMap<&ResourceKey, State<'_>>,
    page: &PageCommit,
    scope: &[u8],
) -> Result<(), StoreError> {
    let mut changed = Vec::new();
    for row in states.values().filter_map(|state| state.prepared.as_ref()) {
        if row.content_changed {
            changed.push(row);
        }
    }
    if !changed.is_empty() {
        let mut query = statement(&changed, page, scope);
        query.build().execute(&mut **tx).await?;
    }
    Ok(())
}

fn statement(rows: &[&Prepared], page: &PageCommit, scope: &[u8]) -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new("WITH input AS (SELECT * FROM UNNEST(");
    keys(&mut query, rows);
    metadata(&mut query, rows);
    query.push(") u(account,region,kind,id,name,content_hash,first_seen,last_changed,is_new)), saved AS (INSERT INTO resources(account,region,kind,id,name,content_hash,first_seen,last_changed) SELECT account,region,kind,id,name,content_hash,first_seen,last_changed FROM input WHERE true ON CONFLICT(account,region,kind,id) DO UPDATE SET name=EXCLUDED.name,content_hash=EXCLUDED.content_hash,last_changed=EXCLUDED.last_changed WHERE resources.content_hash IS DISTINCT FROM EXCLUDED.content_hash OR resources.name IS DISTINCT FROM EXCLUDED.name RETURNING account,region,kind,id), observed AS (INSERT INTO resource_scopes(account,region,kind,id,scope_key,last_seen,missing_complete,observed_sweep) SELECT account,region,kind,id,");
    query
        .push_bind(scope)
        .push(",")
        .push_bind(page.observed_at_ms)
        .push(",0,")
        .push_bind(page.lease.sweep().as_str());
    query.push(" FROM input JOIN saved USING(account,region,kind,id) WHERE is_new ON CONFLICT(account,region,kind,id,scope_key) DO UPDATE SET last_seen=EXCLUDED.last_seen,missing_complete=0,observed_sweep=EXCLUDED.observed_sweep)");
    parents(&mut query, page, scope);
    query.push(" SELECT count(*) FROM saved");
    query
}

fn keys(query: &mut QueryBuilder<Postgres>, rows: &[&Prepared]) {
    Keys::new(rows.iter().map(|row| row.aggregate.observation.key())).bind_arrays(query);
    query.push(",");
}

fn metadata(query: &mut QueryBuilder<Postgres>, rows: &[&Prepared]) {
    let names: Vec<_> = rows
        .iter()
        .map(|row| row.aggregate.observation.name())
        .collect();
    let hashes: Vec<_> = rows.iter().map(|row| row.hash.as_slice()).collect();
    let first: Vec<_> = rows.iter().map(|row| row.first_seen).collect();
    let last: Vec<_> = rows.iter().map(|row| row.last_changed).collect();
    let new: Vec<_> = rows.iter().map(|row| row.new).collect();
    query.push_bind(names).push("::text[],");
    query.push_bind(hashes).push("::bytea[],");
    query.push_bind(first).push("::bigint[],");
    query.push_bind(last).push("::bigint[],");
    query.push_bind(new).push("::boolean[]");
}

fn parents(query: &mut QueryBuilder<Postgres>, page: &PageCommit, scope: &[u8]) {
    if let Some(parent) = page.lease.scope().parent() {
        query.push(", parents AS (INSERT INTO resource_parents(parent_account,parent_region,parent_kind,parent_id,child_account,child_region,child_kind,child_id,scope_key) SELECT ");
        query
            .push_bind(parent.account.as_str())
            .push(",")
            .push_bind(parent.region.as_str())
            .push(",")
            .push_bind(parent.kind.as_str())
            .push(",")
            .push_bind(parent.id.as_str())
            .push(",account,region,kind,id,")
            .push_bind(scope);
        query.push(" FROM input JOIN saved USING(account,region,kind,id) WHERE is_new ON CONFLICT DO NOTHING)");
    }
}
