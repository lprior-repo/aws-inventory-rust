use super::reference_candidates;
#[path = "commit/edges.rs"]
pub(super) mod edges;
#[path = "commit/references_tail.rs"]
mod references_tail;
use super::work::{ChangeStats, WorkLease};
use super::{BinaryStore, PageCommit, StoreError};
use inventory_core::inventory::{Edge, EdgeSource, Observation, ResourceKey, UnresolvedReference};
use inventory_core::record::FieldId;
use references_tail::*;
pub(super) use references_tail::{
    edge_source, enqueue, reference_path, reference_scope, source_key,
};
use sqlx::{QueryBuilder, Row, Sqlite, Transaction};

pub(super) async fn persist_page(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    page: &PageCommit,
) -> Result<ChangeStats, StoreError> {
    let direct_references = edges::references(&store.registry, page)?;
    let authority = super::relation_permissions::page_authority(&store.registry, &page.lease)?;
    let mut stats = ChangeStats::default();
    let mut tag_refreshes = page
        .tags
        .iter()
        .map(|tags| tags.resource())
        .collect::<std::collections::BTreeSet<_>>();
    for observation in &page.observations {
        let source_authority = authority
            .as_ref()
            .map(|authority| authority.source(observation))
            .transpose()?;
        if let Some(authority) = &source_authority {
            for (reference, target) in direct_references
                .iter()
                .filter(|(reference, _)| &reference.from == observation.key())
            {
                super::relation_permissions::mark_direct_reference(
                    authority, tx, reference, target,
                )
                .await?;
            }
            for reference in page
                .unresolved
                .iter()
                .filter(|reference| &reference.from == observation.key())
            {
                super::relation_permissions::mark_reference(authority, tx, reference).await?;
            }
            super::legacy::observe_source(
                tx,
                observation.key(),
                authority.source_facet(),
                page.lease.sweep(),
            )
            .await?;
        }
        match persist_observation(
            store,
            tx,
            page,
            observation,
            &direct_references,
            source_authority.as_ref(),
            &mut stats,
        )
        .await?
        {
            Some(true) => {
                tag_refreshes.remove(observation.key());
                stats.resources_changed = stats
                    .resources_changed
                    .checked_add(1)
                    .ok_or(StoreError::Invalid("change count overflow"))?
            }
            Some(false) => {
                tag_refreshes.remove(observation.key());
                stats.resources_inserted = stats
                    .resources_inserted
                    .checked_add(1)
                    .ok_or(StoreError::Invalid("insert count overflow"))?
            }
            None => {}
        }
    }
    for tags in &page.tags {
        persist_tags(
            tx,
            tags,
            tag_refreshes.contains(tags.resource()),
            page.observed_at_ms,
        )
        .await?;
    }
    Ok(stats)
}

async fn persist_observation(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    page: &PageCommit,
    observation: &Observation,
    direct_references: &[(UnresolvedReference, inventory_core::inventory::ResourceId)],
    authority: Option<&super::relation_permissions::SourceAuthority<'_, '_>>,
    stats: &mut ChangeStats,
) -> Result<Option<bool>, StoreError> {
    let key = observation.key();
    let now = page.observed_at_ms;
    let lease = &page.lease;
    let old = sqlx::query(
        "SELECT content_hash,name,last_changed,EXISTS(SELECT 1 FROM legacy_resources baseline WHERE baseline.account=resources.account AND baseline.region=resources.region AND baseline.kind=resources.kind AND baseline.id=resources.id) AS legacy FROM resources WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4",
    )
    .bind(key.account.as_str())
    .bind(key.region.as_str())
    .bind(key.kind.as_str())
    .bind(key.id.as_str())
    .fetch_optional(&mut **tx)
    .await?;
    let legacy = old
        .as_ref()
        .map(|row| row.try_get::<bool, _>("legacy"))
        .transpose()?
        .unwrap_or(false);
    let old_name = old
        .as_ref()
        .map(|row| row.try_get::<Option<String>, _>("name"))
        .transpose()?
        .flatten();
    let old_changed_at = old
        .as_ref()
        .map(|row| row.try_get::<i64, _>("last_changed"))
        .transpose()?
        .unwrap_or(now);
    let old_hash: Option<Vec<u8>> = old.map(|row| row.try_get("content_hash")).transpose()?;
    if old_hash.is_none() {
        upsert_resource(tx, observation, now, false).await?;
    }
    let aggregate = super::facets::apply(
        &store.registry,
        tx,
        page,
        observation,
        direct_references,
        authority,
    )
    .await?;
    let observation = &aggregate.observation;
    let changed = !legacy
        && (old_hash.as_deref() != Some(observation.content_hash().as_bytes())
            || old_name.as_deref() != observation.name());
    if changed {
        upsert_resource(tx, observation, now, true).await?;
        super::typed::upsert_kind(store, tx, observation, now).await?;
        refresh_incoming(store, tx, observation, stats).await?;
        enqueue(tx, key, false).await?;
    }
    if !legacy && !changed {
        let projection_changed = aggregate.record_changed
            && super::typed::upsert_kind(store, tx, observation, old_changed_at).await?;
        if projection_changed || aggregate.bound_context_changed {
            // Request-only evidence changes also invalidate incoming typed matches.
            refresh_incoming(store, tx, observation, stats).await?;
            enqueue(tx, key, false).await?;
        }
    }
    if !legacy && (changed || aggregate.relations_changed) {
        clear_source_relations(tx, key).await?;
        stats.edges_changed = stats
            .edges_changed
            .checked_add(restore_relations(store, tx, &aggregate).await?)
            .ok_or(StoreError::Invalid("edge count overflow"))?;
        if !changed {
            enqueue(tx, key, false).await?;
        }
    }
    observe_authority(tx, lease, key, now).await?;
    Ok(if changed {
        Some(old_hash.is_some())
    } else {
        None
    })
}

async fn refresh_incoming(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    observation: &Observation,
    stats: &mut ChangeStats,
) -> Result<(), StoreError> {
    stats.edges_changed = stats
        .edges_changed
        .checked_add(super::references::refresh(store, tx, observation.key(), observation).await?)
        .ok_or(StoreError::Invalid("edge count overflow"))?;
    Ok(())
}

pub(super) async fn retire_source(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    key: &ResourceKey,
    source: &[u8],
    at: i64,
) -> Result<ChangeStats, StoreError> {
    let Some(aggregate) = super::facets::retire_source(&store.registry, tx, key, source).await?
    else {
        return Ok(ChangeStats::default());
    };
    if super::legacy::contains(tx, key).await? {
        return Ok(ChangeStats::default());
    }
    publish_aggregate(store, tx, &aggregate, at).await
}

pub(super) async fn publish_aggregate(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    aggregate: &super::facets::Aggregate,
    at: i64,
) -> Result<ChangeStats, StoreError> {
    let key = aggregate.observation.key();
    let observation = &aggregate.observation;
    let old = sqlx::query("SELECT content_hash,name,last_changed FROM resources WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .fetch_one(&mut **tx).await?;
    let changed = old.try_get::<Vec<u8>, _>("content_hash")?.as_slice()
        != observation.content_hash().as_bytes()
        || old.try_get::<Option<String>, _>("name")?.as_deref() != observation.name();
    let changed_at = if changed {
        at
    } else {
        old.try_get("last_changed")?
    };
    if changed {
        upsert_resource(tx, observation, at, true).await?;
    }
    super::typed::upsert_kind(store, tx, observation, changed_at).await?;
    let incoming_edges = super::references::refresh(store, tx, key, observation).await?;
    clear_source_relations(tx, key).await?;
    let edges_changed = incoming_edges
        .checked_add(restore_relations(store, tx, aggregate).await?)
        .ok_or(StoreError::Invalid("edge count overflow"))?;
    enqueue(tx, key, false).await?;
    Ok(ChangeStats {
        resources_changed: u64::from(changed),
        edges_changed,
        ..ChangeStats::default()
    })
}

async fn restore_relations(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    aggregate: &super::facets::Aggregate,
) -> Result<u64, StoreError> {
    let mut inserted = 0_u64;
    for (reference, pin, provenance) in &aggregate.references {
        super::relation_permissions::activate_reference(
            &store.registry,
            tx,
            reference,
            pin.as_ref(),
            provenance,
        )
        .await?;
    }
    for (reference, _, _) in &aggregate.references {
        persist_reference(tx, reference).await?;
        inserted = inserted
            .checked_add(resolve_reference(store, tx, reference).await?)
            .ok_or(StoreError::Invalid("edge count overflow"))?;
    }
    Ok(inserted)
}
async fn clear_source_relations(
    tx: &mut Transaction<'_, Sqlite>,
    key: &ResourceKey,
) -> Result<(), StoreError> {
    super::relation_permissions::clear_active(tx, key).await?;
    for table in ["edges", "unresolved_refs"] {
        let columns = ["from_account", "from_region", "from_kind", "from_id"];
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

async fn observe_authority(
    tx: &mut Transaction<'_, Sqlite>,
    lease: &WorkLease,
    child: &ResourceKey,
    at: i64,
) -> Result<(), StoreError> {
    let scope = super::work::ownership_scope_bytes(lease.scope(), lease.bindings())?;
    sqlx::query("INSERT INTO resource_scopes(account,region,kind,id,scope_key,last_seen,missing_complete,observed_sweep) VALUES(?1,?2,?3,?4,?5,?6,0,?7) ON CONFLICT(account,region,kind,id,scope_key) DO UPDATE SET last_seen=excluded.last_seen,missing_complete=0,observed_sweep=excluded.observed_sweep")
        .bind(child.account.as_str()).bind(child.region.as_str()).bind(child.kind.as_str()).bind(child.id.as_str())
        .bind(&scope).bind(at).bind(lease.sweep().as_str()).execute(&mut **tx).await?;
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

pub(super) async fn persist_edge(
    tx: &mut Transaction<'_, Sqlite>,
    edge: &Edge,
) -> Result<bool, StoreError> {
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
    let path = super::reference_candidates::stored_path(target)?;
    // Persist the reference row with the full candidate-kind blob.
    let blob_bytes = reference_candidates::encode(target)?;
    sqlx::query("INSERT INTO unresolved_refs(from_account,from_region,from_kind,from_id,target_scope,target_kind,target_path,target_value,relation,source,target_kinds) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11) ON CONFLICT(from_account,from_region,from_kind,from_id,target_scope,target_kind,target_path,target_value,relation,source) DO UPDATE SET target_kinds=?11")
        .bind(reference.from.account.as_str()).bind(reference.from.region.as_str())
        .bind(reference.from.kind.as_str()).bind(reference.from.id.as_str())
        .bind(&scope).bind(target.kind().as_str()).bind(&path).bind(target.value().as_str())
        .bind(reference.relation.as_str()).bind(reference.source.as_str())
        .bind(&blob_bytes)
        .execute(&mut **tx).await?;
    // Refresh the derived candidate index rows for this reference.
    refresh_ref_candidates(tx, reference, &scope, &path).await?;
    Ok(())
}

async fn refresh_ref_candidates(
    tx: &mut Transaction<'_, Sqlite>,
    reference: &UnresolvedReference,
    scope: &[u8],
    path: &[u8],
) -> Result<(), StoreError> {
    let target = &reference.target;
    // Delete old candidate rows for this reference.
    sqlx::query("DELETE FROM unresolved_ref_candidates WHERE from_account=?1 AND from_region=?2 AND from_kind=?3 AND from_id=?4 AND target_scope=?5 AND target_kind=?6 AND target_path=?7 AND target_value=?8 AND relation=?9 AND source=?10")
        .bind(reference.from.account.as_str()).bind(reference.from.region.as_str())
        .bind(reference.from.kind.as_str()).bind(reference.from.id.as_str())
        .bind(scope).bind(target.kind().as_str()).bind(path)
        .bind(target.value().as_str())
        .bind(reference.relation.as_str()).bind(reference.source.as_str())
        .execute(&mut **tx).await?;
    // Insert one candidate row per candidate kind (including primary).
    let kinds = target.candidate_kinds();
    for kind in kinds.iter() {
        sqlx::query("INSERT INTO unresolved_ref_candidates(target_scope,candidate_kind,from_account,from_region,from_kind,from_id,target_kind,target_path,target_value,relation,source) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)")
            .bind(scope).bind(kind.as_str())
            .bind(reference.from.account.as_str()).bind(reference.from.region.as_str())
            .bind(reference.from.kind.as_str()).bind(reference.from.id.as_str())
            .bind(target.kind().as_str()).bind(path)
            .bind(target.value().as_str())
            .bind(reference.relation.as_str()).bind(reference.source.as_str())
            .execute(&mut **tx).await?;
    }
    Ok(())
}
async fn resolve_reference(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    reference: &UnresolvedReference,
) -> Result<u64, StoreError> {
    reference_candidates::resolve(store, tx, reference).await
}

pub(super) async fn request_graph_rebuild(store: &BinaryStore) -> Result<(), StoreError> {
    let (_turn, mut tx) = store.begin_write().await?;
    let revision: i64 = sqlx::query_scalar("UPDATE metadata SET value=CAST(value AS INTEGER)+1 WHERE key='projection_revision' RETURNING value")
        .fetch_one(&mut *tx)
        .await?;
    enqueue_rebuild(&mut tx, revision).await?;
    tx.commit().await?;
    Ok(())
}
