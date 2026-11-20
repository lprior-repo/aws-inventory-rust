use super::commit::{enqueue, quote};
use super::work::{ChangeStats, SweepId};
use super::{BinaryStore, StoreError};
use inventory_core::inventory::ResourceKey;
use sqlx::{QueryBuilder, Row, Sqlite, Transaction};
use std::collections::{BTreeMap, BTreeSet};

pub(super) async fn drain_sweep(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    sweep: &SweepId,
    at: i64,
) -> Result<ChangeStats, StoreError> {
    // Child admission, terminal transition and this check share the writer
    // transaction: no undiscovered execution may race a negative observation.
    let pending: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM work_queue WHERE sweep_id=?1 AND state<>2)",
    )
    .bind(sweep.as_str())
    .fetch_one(&mut **tx)
    .await?;
    if pending {
        return Ok(ChangeStats::default());
    }
    super::unobserved::record(tx, sweep).await?;
    let rows = sqlx::query("SELECT work.scope_key,work.bindings,work.authority_complete,coalesce(outcomes.outcome,'partial') AS outcome FROM work_queue work LEFT JOIN scope_outcomes outcomes ON outcomes.sweep_id=work.sweep_id AND outcomes.scope_key=work.scope_key WHERE work.sweep_id=?1 AND work.authority_generation=work.generation AND work.generation>0")
        .bind(sweep.as_str()).fetch_all(&mut **tx).await?;
    let mut authorities = BTreeMap::<Vec<u8>, (bool, bool)>::new();
    for row in rows {
        let execution = super::work::persisted_scope(row.try_get("scope_key")?)?;
        let source = super::work::ownership_scope_bytes(&execution, row.try_get("bindings")?)?;
        let complete = row.try_get::<&str, _>("outcome")? == "complete";
        let prefix: bool = row.try_get("authority_complete")?;
        authorities
            .entry(source)
            .and_modify(|all| {
                all.0 &= complete;
                all.1 &= prefix;
            })
            .or_insert((complete, prefix));
    }
    let mut stats = ChangeStats::default();
    for (source, (complete, prefix)) in authorities {
        if complete {
            add_stats(
                &mut stats,
                complete_authority(store, tx, &source, sweep, at, prefix).await?,
            )?;
        }
    }
    add_stats(
        &mut stats,
        super::legacy::publish_ready(store, tx, at).await?,
    )?;
    Ok(stats)
}

async fn complete_authority(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    source: &[u8],
    sweep: &SweepId,
    at: i64,
    complete_scan: bool,
) -> Result<ChangeStats, StoreError> {
    let checked = store.registry.permissions.catalog().is_some();
    let rows = sqlx::query("SELECT current.account,current.region,current.kind,current.id,current.missing_complete FROM resource_scopes current LEFT JOIN legacy_authorities pending USING(account,region,kind,id,scope_key) LEFT JOIN legacy_observations fresh ON fresh.account=current.account AND fresh.region=current.region AND fresh.kind=current.kind AND fresh.id=current.id AND fresh.scope_key=current.scope_key AND fresh.sweep_id=?2 WHERE current.scope_key=?1 AND ?4 AND (?3 OR NOT EXISTS(SELECT 1 FROM legacy_resources baseline WHERE baseline.account=current.account AND baseline.region=current.region AND baseline.kind=current.kind AND baseline.id=current.id)) AND CASE WHEN pending.scope_key IS NOT NULL THEN fresh.scope_key IS NULL AND pending.v3_completed_sweep<>?2 ELSE current.observed_sweep<>?2 AND current.completed_sweep<>?2 END")
        .bind(source).bind(sweep.as_str()).bind(checked).bind(complete_scan).fetch_all(&mut **tx).await?;
    let mut stats = ChangeStats::default();
    for row in rows {
        let misses: i64 = row.try_get("missing_complete")?;
        let key = resource_key_from_row(row)?;
        let updated = sqlx::query("UPDATE resource_scopes SET missing_complete=missing_complete+1,last_seen=?5,completed_sweep=?7 WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 AND scope_key=?6")
            .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
            .bind(at).bind(source).bind(sweep.as_str()).execute(&mut **tx).await?;
        if updated.rows_affected() == 0 {
            continue;
        }
        let legacy_retirement = super::legacy::observe_miss(tx, &key, source, sweep).await?;
        if misses < 1 || legacy_retirement == Some(false) {
            continue;
        }
        if legacy_retirement == Some(true) {
            super::legacy::retire_source(tx, &key, source).await?;
        }
        let remaining = live_owners(tx, &key).await?;
        if remaining == 0 {
            add_stats(&mut stats, delete_tree(store, tx, &key, at).await?)?;
        } else if misses == 1 || legacy_retirement == Some(true) {
            add_stats(
                &mut stats,
                super::commit::retire_source(store, tx, &key, source, at).await?,
            )?;
        }
    }
    if checked {
        super::legacy::complete_source(tx, source, sweep).await?;
    }
    Ok(stats)
}

fn add_stats(stats: &mut ChangeStats, added: ChangeStats) -> Result<(), StoreError> {
    for (current, amount) in [
        (&mut stats.resources_inserted, added.resources_inserted),
        (&mut stats.resources_changed, added.resources_changed),
        (&mut stats.resources_deleted, added.resources_deleted),
        (&mut stats.edges_changed, added.edges_changed),
    ] {
        *current = current
            .checked_add(amount)
            .ok_or(StoreError::Invalid("completion count overflow"))?;
    }
    Ok(())
}

async fn live_owners(
    tx: &mut Transaction<'_, Sqlite>,
    key: &ResourceKey,
) -> Result<i64, StoreError> {
    Ok(sqlx::query_scalar("SELECT (SELECT COUNT(*) FROM resource_scopes WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 AND missing_complete<2)+(SELECT COUNT(*) FROM legacy_authorities WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4)+(SELECT COUNT(*) FROM legacy_unknown_authorities WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4)")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .fetch_one(&mut **tx).await?)
}

async fn delete_tree(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    root: &ResourceKey,
    at: i64,
) -> Result<ChangeStats, StoreError> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resources")
        .fetch_one(&mut **tx)
        .await?;
    let mut pending = BTreeSet::from([root.clone()]);
    let mut deleted = BTreeSet::new();
    let mut stats = ChangeStats::default();
    // The transaction inserts no resources; each original key is visited at
    // most once, even if the parent graph contains shared paths or cycles.
    for _ in 0..count {
        let Some(key) = pending.pop_first() else {
            break;
        };
        deleted.insert(key.clone());
        let (children, changed) = retire_children(store, tx, &key, at).await?;
        add_stats(&mut stats, changed)?;
        for child in children {
            if !deleted.contains(&child) {
                pending.insert(child);
            }
        }
        enqueue(tx, &key, true).await?;
        delete_resource_rows(store, tx, &key).await?;
        stats.resources_deleted = stats
            .resources_deleted
            .checked_add(1)
            .ok_or(StoreError::Invalid("resource deletion count overflow"))?;
    }
    if !pending.is_empty() {
        return Err(StoreError::Invalid(
            "resource retirement traversal exceeded resource count",
        ));
    }
    for key in &deleted {
        super::references::refresh_missing(store, tx, key).await?;
    }
    Ok(stats)
}

async fn retire_children(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    parent: &ResourceKey,
    at: i64,
) -> Result<(Vec<ResourceKey>, ChangeStats), StoreError> {
    let rows = sqlx::query("SELECT child_account AS account,child_region AS region,child_kind AS kind,child_id AS id,scope_key FROM resource_parents WHERE parent_account=?1 AND parent_region=?2 AND parent_kind=?3 AND parent_id=?4")
        .bind(parent.account.as_str()).bind(parent.region.as_str()).bind(parent.kind.as_str()).bind(parent.id.as_str())
        .fetch_all(&mut **tx).await?;
    let mut children = Vec::new();
    let mut stats = ChangeStats::default();
    for row in rows {
        let source: Vec<u8> = row.try_get("scope_key")?;
        let child = resource_key_from_row(row)?;
        sqlx::query("UPDATE resource_scopes SET missing_complete=max(missing_complete,2) WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 AND scope_key=?5")
            .bind(child.account.as_str()).bind(child.region.as_str()).bind(child.kind.as_str()).bind(child.id.as_str())
            .bind(&source).execute(&mut **tx).await?;
        super::legacy::retire_source(tx, &child, &source).await?;
        if live_owners(tx, &child).await? == 0 {
            // The child is deleted after this parent; do not publish an empty
            // intermediate canonical row or its outgoing relationships.
            super::facets::retire_source(&store.registry, tx, &child, &source).await?;
            children.push(child);
        } else {
            add_stats(
                &mut stats,
                super::commit::retire_source(store, tx, &child, &source, at).await?,
            )?;
        }
    }
    Ok((children, stats))
}

fn resource_key_from_row(row: sqlx::sqlite::SqliteRow) -> Result<ResourceKey, StoreError> {
    Ok(ResourceKey {
        account: inventory_core::inventory::AccountId::parse(
            &row.try_get::<String, _>("account")?,
        )?,
        region: inventory_core::inventory::Region::parse(&row.try_get::<String, _>("region")?)?,
        kind: inventory_core::inventory::Kind::parse(&row.try_get::<String, _>("kind")?)?,
        id: inventory_core::inventory::ResourceId::parse(&row.try_get::<String, _>("id")?)?,
    })
}

async fn delete_resource_rows(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    key: &ResourceKey,
) -> Result<(), StoreError> {
    if let Some(schema) = store
        .registry
        .schemas()
        .iter()
        .find(|schema| schema.kind() == &key.kind)
    {
        let mut query = QueryBuilder::<Sqlite>::new("DELETE FROM ");
        query
            .push(quote(schema.table().as_str()))
            .push(" WHERE account=")
            .push_bind(key.account.as_str())
            .push(" AND region=")
            .push_bind(key.region.as_str())
            .push(" AND id=")
            .push_bind(key.id.as_str())
            .build()
            .execute(&mut **tx)
            .await?;
    }
    sqlx::query("DELETE FROM resources WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4")
        .bind(key.account.as_str())
        .bind(key.region.as_str())
        .bind(key.kind.as_str())
        .bind(key.id.as_str())
        .execute(&mut **tx)
        .await?;
    Ok(())
}
