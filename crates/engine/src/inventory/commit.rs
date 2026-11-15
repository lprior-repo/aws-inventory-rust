#[path = "commit/queue.rs"]
mod queue;
use queue::*;
#[path = "commit/references_tail.rs"]
mod references_tail;
use super::work::{ChangeStats, WorkLease};
use super::{BinaryStore, PageCommit, StoreError};
use inventory_core::inventory::{
    Edge, EdgeSource, Observation, ResourceKey, ScalarType, UnresolvedReference,
};
use inventory_core::record::{FieldId, ValueRef};
use references_tail::*;
pub(super) use references_tail::{
    enqueue, reference_path, reference_scope, source_key, text_field,
};
use sqlx::{QueryBuilder, Row, Sqlite, Transaction};

pub(super) async fn persist_page(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    page: &PageCommit,
) -> Result<ChangeStats, StoreError> {
    let mut stats = ChangeStats::default();
    for observation in &page.observations {
        match persist_observation(store, tx, &page.lease, observation, page.observed_at_ms).await? {
            Some(true) => {
                stats.resources_changed = stats
                    .resources_changed
                    .checked_add(1)
                    .ok_or(StoreError::Invalid("change count overflow"))?
            }
            Some(false) => {
                stats.resources_inserted = stats
                    .resources_inserted
                    .checked_add(1)
                    .ok_or(StoreError::Invalid("insert count overflow"))?
            }
            None => {}
        }
    }
    for edge in &page.edges {
        if persist_edge(tx, edge).await? {
            enqueue(tx, &edge.from, false).await?;
            enqueue(tx, &edge.to, false).await?;
            stats.edges_changed = stats
                .edges_changed
                .checked_add(1)
                .ok_or(StoreError::Invalid("edge count overflow"))?;
        }
    }
    for reference in &page.unresolved {
        persist_reference(tx, reference).await?;
        resolve_reference(store, tx, reference).await?;
    }
    for tag in &page.tags {
        persist_tag(tx, tag).await?;
    }
    Ok(stats)
}

async fn persist_observation(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    lease: &WorkLease,
    observation: &Observation,
    now: i64,
) -> Result<Option<bool>, StoreError> {
    let key = observation.key();
    let old = sqlx::query(
        "SELECT content_hash FROM resources WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4",
    )
    .bind(key.account.as_str())
    .bind(key.region.as_str())
    .bind(key.kind.as_str())
    .bind(key.id.as_str())
    .fetch_optional(&mut **tx)
    .await?;
    let old_hash: Option<Vec<u8>> = old.map(|row| row.try_get("content_hash")).transpose()?;
    let changed = old_hash.as_deref() != Some(observation.content_hash().as_bytes());
    if changed {
        clear_source_relations(tx, key).await?;
        upsert_resource(tx, observation, now, old_hash.is_some()).await?;
        super::typed::upsert_kind(store, tx, observation, now).await?;
        super::references::reconcile_inbound(store, tx, key, observation).await?;
        resolve_references(store, tx, key, observation).await?;
        enqueue(tx, key, false).await?;
    }
    sqlx::query("INSERT INTO resource_scopes(account,region,kind,id,scope_key,last_seen,missing_complete,observed_sweep) VALUES(?1,?2,?3,?4,?5,?6,0,?7) ON CONFLICT(account,region,kind,id,scope_key) DO UPDATE SET last_seen=excluded.last_seen,missing_complete=0,observed_sweep=excluded.observed_sweep")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .bind(super::work::scope_bytes(lease.scope())?).bind(now).bind(lease.sweep().as_str()).execute(&mut **tx).await?;
    persist_parent(tx, lease, key).await?;
    Ok(if changed {
        Some(old_hash.is_some())
    } else {
        None
    })
}
async fn clear_source_relations(
    tx: &mut Transaction<'_, Sqlite>,
    key: &ResourceKey,
) -> Result<(), StoreError> {
    for table in ["edges", "tags", "unresolved_refs"] {
        let prefix = table != "tags";
        let columns = if prefix {
            ["from_account", "from_region", "from_kind", "from_id"]
        } else {
            ["account", "region", "kind", "id"]
        };
        let mut query = QueryBuilder::<Sqlite>::new("DELETE FROM ");
        query
            .push(table)
            .push(" WHERE ")
            .push(quote(columns[0]))
            .push("=")
            .push_bind(key.account.as_str());
        query
            .push(" AND ")
            .push(quote(columns[1]))
            .push("=")
            .push_bind(key.region.as_str());
        query
            .push(" AND ")
            .push(quote(columns[2]))
            .push("=")
            .push_bind(key.kind.as_str());
        query
            .push(" AND ")
            .push(quote(columns[3]))
            .push("=")
            .push_bind(key.id.as_str())
            .build()
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

async fn persist_parent(
    tx: &mut Transaction<'_, Sqlite>,
    lease: &WorkLease,
    child: &ResourceKey,
) -> Result<(), StoreError> {
    let scope = super::work::scope_bytes(lease.scope())?;
    sqlx::query("DELETE FROM resource_parents WHERE child_account=?1 AND child_region=?2 AND child_kind=?3 AND child_id=?4 AND scope_key=?5")
        .bind(child.account.as_str()).bind(child.region.as_str()).bind(child.kind.as_str()).bind(child.id.as_str()).bind(&scope)
        .execute(&mut **tx).await?;
    if let Some(parent) = lease.scope().parent() {
        sqlx::query("INSERT OR IGNORE INTO resource_parents(parent_account,parent_region,parent_kind,parent_id,child_account,child_region,child_kind,child_id,scope_key) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)")
            .bind(parent.account.as_str()).bind(parent.region.as_str()).bind(parent.kind.as_str()).bind(parent.id.as_str())
            .bind(child.account.as_str()).bind(child.region.as_str()).bind(child.kind.as_str()).bind(child.id.as_str()).bind(scope)
            .execute(&mut **tx).await?;
    }
    Ok(())
}
async fn upsert_resource(
    tx: &mut Transaction<'_, Sqlite>,
    observation: &Observation,
    now: i64,
    exists: bool,
) -> Result<(), StoreError> {
    let key = observation.key();
    let first_seen = if exists {
        sqlx::query(
            "SELECT first_seen FROM resources WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4",
        )
        .bind(key.account.as_str())
        .bind(key.region.as_str())
        .bind(key.kind.as_str())
        .bind(key.id.as_str())
        .fetch_one(&mut **tx)
        .await?
        .try_get::<i64, _>("first_seen")?
    } else {
        now
    };
    sqlx::query("INSERT INTO resources(account,region,kind,id,name,content_hash,first_seen,last_changed) VALUES(?1,?2,?3,?4,?5,?6,?7,?8) ON CONFLICT(account,region,kind,id) DO UPDATE SET name=excluded.name,content_hash=excluded.content_hash,last_changed=excluded.last_changed")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .bind(observation.name()).bind(observation.content_hash().as_bytes().as_slice()).bind(first_seen).bind(now).execute(&mut **tx).await?;
    Ok(())
}

pub(super) fn quote(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

async fn persist_edge(tx: &mut Transaction<'_, Sqlite>, edge: &Edge) -> Result<bool, StoreError> {
    let result = sqlx::query("INSERT OR IGNORE INTO edges(from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)")
        .bind(edge.from.account.as_str()).bind(edge.from.region.as_str()).bind(edge.from.kind.as_str()).bind(edge.from.id.as_str())
        .bind(edge.to.account.as_str()).bind(edge.to.region.as_str()).bind(edge.to.kind.as_str()).bind(edge.to.id.as_str())
        .bind(edge.relation.as_str()).bind(edge.source.as_str()).execute(&mut **tx).await?;
    Ok(result.rows_affected() == 1)
}
async fn persist_reference(
    tx: &mut Transaction<'_, Sqlite>,
    reference: &UnresolvedReference,
) -> Result<(), StoreError> {
    let target = &reference.target;
    let scope = reference_scope(target.account().as_str(), target.region().as_str())?;
    let path = reference_path(target.path());
    sqlx::query("INSERT OR IGNORE INTO unresolved_refs(from_account,from_region,from_kind,from_id,target_scope,target_kind,target_path,target_value,relation,source) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)")
        .bind(reference.from.account.as_str()).bind(reference.from.region.as_str()).bind(reference.from.kind.as_str()).bind(reference.from.id.as_str())
        .bind(scope).bind(target.kind().as_str()).bind(path).bind(target.value().as_str()).bind(reference.relation.as_str()).bind(reference.source.as_str())
        .execute(&mut **tx).await?;
    Ok(())
}
async fn resolve_reference(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    reference: &UnresolvedReference,
) -> Result<(), StoreError> {
    let target = &reference.target;
    let schema = store
        .registry
        .schemas()
        .iter()
        .find(|schema| schema.kind() == target.kind())
        .ok_or(StoreError::Invalid(
            "referenced kind is absent from schema registry",
        ))?;
    let Some(column) = schema
        .columns()
        .iter()
        .find(|column| column.path() == target.path() && column.scalar() == ScalarType::Text)
    else {
        return Ok(());
    };
    let mut query = QueryBuilder::<Sqlite>::new("SELECT id FROM ");
    query
        .push(quote(schema.table().as_str()))
        .push(" WHERE account=")
        .push_bind(target.account().as_str())
        .push(" AND region=")
        .push_bind(target.region().as_str())
        .push(" AND ")
        .push(quote(column.name()))
        .push("=")
        .push_bind(target.value().as_str())
        .push(" LIMIT 2");
    let candidates = query.build().fetch_all(&mut **tx).await?;
    if candidates.len() != 1 {
        return Ok(());
    }
    let id: String = candidates
        .first()
        .ok_or(StoreError::Invalid(
            "unique reference candidate disappeared",
        ))?
        .try_get("id")?;
    let to = ResourceKey {
        account: inventory_core::inventory::AccountId::parse(target.account().as_str())?,
        region: inventory_core::inventory::Region::parse(target.region().as_str())?,
        kind: inventory_core::inventory::Kind::parse(target.kind().as_str())?,
        id: inventory_core::inventory::ResourceId::parse(&id)?,
    };
    if persist_edge(
        tx,
        &Edge {
            from: reference.from.clone(),
            to: to.clone(),
            relation: reference.relation.clone(),
            source: reference.source,
        },
    )
    .await?
    {
        enqueue(tx, &reference.from, false).await?;
        enqueue(tx, &to, false).await?;
    }
    Ok(())
}
