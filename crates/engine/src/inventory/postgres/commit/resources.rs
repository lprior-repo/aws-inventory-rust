use super::facets::{self, Context, Facet};
use super::keys::Keys;
use crate::inventory::facets::Aggregate;
use crate::inventory::{PageCommit, SchemaRegistry, StoreError};
use futures_util::TryStreamExt;
use inventory_core::inventory::ResourceKey;
use sqlx::{Postgres, Row, Transaction, postgres::PgRow};
use std::collections::BTreeMap;

const LOAD: &str = "WITH input AS (SELECT * FROM UNNEST($1::text[],$2::text[],$3::text[],$4::text[]) WITH ORDINALITY u(account,region,kind,id,ordinal)), seen AS (INSERT INTO resource_scopes(account,region,kind,id,scope_key,last_seen,missing_complete,observed_sweep) SELECT account,region,kind,id,$5,$6,0,$7 FROM input JOIN resources USING(account,region,kind,id) ON CONFLICT(account,region,kind,id,scope_key) DO UPDATE SET last_seen=EXCLUDED.last_seen,missing_complete=0,observed_sweep=EXCLUDED.observed_sweep), parents AS (INSERT INTO resource_parents(parent_account,parent_region,parent_kind,parent_id,child_account,child_region,child_kind,child_id,scope_key) SELECT $8,$9,$10,$11,account,region,kind,id,$5 FROM input JOIN resources USING(account,region,kind,id) WHERE $8::text IS NOT NULL ON CONFLICT DO NOTHING) SELECT input.ordinal,r.content_hash,r.first_seen,r.last_changed,r.name AS resource_name,f.* FROM input JOIN resources r USING(account,region,kind,id) LEFT JOIN resource_facets f USING(account,region,kind,id) ORDER BY input.ordinal";

pub(super) struct State<'a> {
    pub key: &'a ResourceKey,
    pub old_hash: Option<[u8; 32]>,
    pub old_name: Option<String>,
    pub first_seen: i64,
    pub last_changed: i64,
    pub facets: Vec<Facet<'a>>,
    pub facet_changed: bool,
    pub prepared: Option<Prepared>,
}

pub(super) struct Prepared {
    pub aggregate: Aggregate,
    pub hash: [u8; 32],
    pub content_changed: bool,
    pub first_seen: i64,
    pub last_changed: i64,
    pub new: bool,
}

pub(super) fn states(page: &PageCommit) -> BTreeMap<&ResourceKey, State<'_>> {
    page.observations
        .iter()
        .map(|observation| {
            let key = observation.key();
            (key, initial(key, page.observed_at_ms))
        })
        .collect()
}

fn initial(key: &ResourceKey, at: i64) -> State<'_> {
    State {
        key,
        old_hash: None,
        old_name: None,
        first_seen: at,
        last_changed: at,
        facets: Vec::new(),
        facet_changed: false,
        prepared: None,
    }
}

pub(super) async fn load(
    tx: &mut Transaction<'_, Postgres>,
    states: &mut BTreeMap<&ResourceKey, State<'_>>,
    context: &Context<'_>,
) -> Result<(), StoreError> {
    let ordered = states.keys().copied().collect::<Vec<_>>();
    let keys = Keys::new(ordered.iter().copied());
    let mut rows = select(&keys, context.page, &context.identity.source).fetch(&mut **tx);
    while let Some(row) = rows.try_next().await? {
        let index = ordinal(&row)?;
        let key = ordered
            .get(index)
            .ok_or(StoreError::Invalid("resource index exceeds bounds"))?;
        let state = states
            .get_mut(key)
            .ok_or(StoreError::Invalid("resource key is absent"))?;
        append(state, &row, context)?;
    }
    Ok(())
}

pub(super) fn ordinal(row: &PgRow) -> Result<usize, StoreError> {
    usize::try_from(row.try_get::<i64, _>("ordinal")?)
        .ok()
        .and_then(|value| value.checked_sub(1))
        .ok_or(StoreError::Invalid("resource index exceeds bounds"))
}

fn append(state: &mut State<'_>, row: &PgRow, context: &Context<'_>) -> Result<(), StoreError> {
    if state.old_hash.is_none() {
        state.old_hash = Some(
            row.try_get::<&[u8], _>("content_hash")?
                .try_into()
                .map_err(|_| StoreError::Invalid("resource hash is not 32 bytes"))?,
        );
        state.old_name = row.try_get("resource_name")?;
        state.first_seen = row.try_get("first_seen")?;
        state.last_changed = row.try_get("last_changed")?;
    }
    if row.try_get::<Option<&[u8]>, _>("source_key")?.is_some() {
        if state.facets.len() >= 4096 {
            return Err(StoreError::Invalid("resource facet count exceeds bounds"));
        }
        state.facets.push(facets::decode(row, context)?);
    }
    Ok(())
}

pub(super) fn prepare(
    registry: &SchemaRegistry,
    state: &mut State<'_>,
    at: i64,
) -> Result<(), StoreError> {
    let aggregate = facets::build(registry, state.key, &mut state.facets)?;
    let hash = *aggregate.observation.content_hash().as_bytes();
    let content_changed =
        state.old_hash != Some(hash) || state.old_name.as_deref() != aggregate.observation.name();
    state.prepared = Some(Prepared {
        aggregate,
        hash,
        content_changed,
        first_seen: state.first_seen,
        last_changed: if content_changed {
            at
        } else {
            state.last_changed
        },
        new: state.old_hash.is_none(),
    });
    Ok(())
}

fn select(
    keys: &Keys<'_>,
    page: &PageCommit,
    scope: &[u8],
) -> sqlx::query::Query<'static, Postgres, sqlx::postgres::PgArguments> {
    let parent = page.lease.scope().parent();
    sqlx::query(LOAD)
        .bind(&keys.accounts)
        .bind(&keys.regions)
        .bind(&keys.kinds)
        .bind(&keys.ids)
        .bind(scope)
        .bind(page.observed_at_ms)
        .bind(page.lease.sweep().as_str())
        .bind(parent.map(|key| key.account.as_str()))
        .bind(parent.map(|key| key.region.as_str()))
        .bind(parent.map(|key| key.kind.as_str()))
        .bind(parent.map(|key| key.id.as_str()))
}
