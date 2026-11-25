use super::super::copy::{self, Field};
use super::keys::Keys;
use super::resources::{Prepared, State};
use crate::inventory::{PageCommit, StoreError};
use inventory_core::inventory::ResourceKey;
use sqlx::{Postgres, QueryBuilder, Transaction};
use std::collections::BTreeMap;
use std::num::NonZeroUsize;

const CREATE_STAGE: &str = "CREATE TEMPORARY TABLE IF NOT EXISTS stg_resources (LIKE inventory.resources, is_new boolean NOT NULL) ON COMMIT DELETE ROWS";
const COPY_STAGE: &str = "COPY pg_temp.stg_resources FROM STDIN (FORMAT binary)";

enum Input<'a> {
    Unnest(&'a [&'a Prepared]),
    Staging,
}

pub(super) async fn persist(
    tx: &mut Transaction<'_, Postgres>,
    states: &BTreeMap<&ResourceKey, State<'_>>,
    page: &PageCommit,
    scope: &[u8],
    copy_min_rows: Option<NonZeroUsize>,
) -> Result<(), StoreError> {
    let mut changed = Vec::new();
    for row in states.values().filter_map(|state| state.prepared.as_ref()) {
        if row.content_changed {
            changed.push(row);
        }
    }
    if changed.is_empty() {
        return Ok(());
    }
    let input = if copy_min_rows.is_some_and(|min| changed.len() >= min.get()) {
        copy::load(tx, CREATE_STAGE, COPY_STAGE, &changed, |buffer, row| {
            encode(buffer, row)
        })
        .await?;
        Input::Staging
    } else {
        Input::Unnest(&changed)
    };
    statement(input, page, scope)
        .build()
        .execute(&mut **tx)
        .await?;
    Ok(())
}

fn encode(buffer: &mut Vec<u8>, row: &Prepared) -> Result<(), StoreError> {
    let observation = &row.aggregate.observation;
    let key = observation.key();
    copy::row(buffer, 9)?;
    [
        Field::Text(key.account.as_str()),
        Field::Text(key.region.as_str()),
        Field::Text(key.kind.as_str()),
        Field::Text(key.id.as_str()),
        observation.name().map_or(Field::Null, Field::Text),
        Field::Bytes(&row.hash),
        Field::Bigint(row.first_seen),
        Field::Bigint(row.last_changed),
        Field::Boolean(row.new),
    ]
    .into_iter()
    .try_for_each(|field| copy::field(buffer, field))
}

fn statement(input: Input<'_>, page: &PageCommit, scope: &[u8]) -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new("WITH input AS (SELECT * FROM ");
    match input {
        Input::Unnest(rows) => {
            query.push("UNNEST(");
            keys(&mut query, rows);
            metadata(&mut query, rows);
            query.push(
                ") u(account,region,kind,id,name,content_hash,first_seen,last_changed,is_new)",
            );
        }
        Input::Staging => {
            query.push("pg_temp.stg_resources");
        }
    }
    query.push("), saved AS (INSERT INTO resources(account,region,kind,id,name,content_hash,first_seen,last_changed) SELECT account,region,kind,id,name,content_hash,first_seen,last_changed FROM input WHERE true ON CONFLICT(account,region,kind,id) DO UPDATE SET name=EXCLUDED.name,content_hash=EXCLUDED.content_hash,last_changed=EXCLUDED.last_changed WHERE resources.content_hash IS DISTINCT FROM EXCLUDED.content_hash OR resources.name IS DISTINCT FROM EXCLUDED.name RETURNING account,region,kind,id), observed AS (INSERT INTO resource_scopes(account,region,kind,id,scope_key,last_seen,missing_complete,observed_sweep) SELECT account,region,kind,id,");
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

pub(in crate::inventory::postgres) async fn retire(
    tx: &mut Transaction<'_, Postgres>,
    states: &BTreeMap<&ResourceKey, State<'_>>,
) -> Result<(), StoreError> {
    let rows: Vec<_> = states
        .values()
        .filter_map(|state| state.prepared.as_ref())
        .filter(|row| row.content_changed)
        .collect();
    if rows.is_empty() {
        return Ok(());
    }
    let mut query = QueryBuilder::new("WITH input AS (SELECT * FROM UNNEST(");
    keys(&mut query, &rows);
    metadata(&mut query, &rows);
    query.push(") u(account,region,kind,id,name,content_hash,first_seen,last_changed,is_new)) UPDATE resources r SET name=u.name,content_hash=u.content_hash,last_changed=u.last_changed FROM input u WHERE (r.account,r.region,r.kind,r.id)=(u.account,u.region,u.kind,u.id) AND (r.content_hash IS DISTINCT FROM u.content_hash OR r.name IS DISTINCT FROM u.name)");
    query.build().execute(&mut **tx).await?;
    Ok(())
}
