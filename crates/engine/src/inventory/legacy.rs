use super::{BinaryStore, StoreError, facets, work::{ChangeStats, SweepId}};
use inventory_core::inventory::{AccountId, Kind, Region, ResourceId, ResourceKey};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};

pub(super) async fn create(tx: &mut Transaction<'_, Sqlite>) -> Result<(), StoreError> {
    for statement in [
        "CREATE TABLE IF NOT EXISTS legacy_resources(account TEXT NOT NULL,region TEXT NOT NULL,kind TEXT NOT NULL,id TEXT NOT NULL,PRIMARY KEY(account,region,kind,id),FOREIGN KEY(account,region,kind,id) REFERENCES resources(account,region,kind,id) ON DELETE CASCADE) WITHOUT ROWID",
        "CREATE TABLE IF NOT EXISTS legacy_authorities(account TEXT NOT NULL,region TEXT NOT NULL,kind TEXT NOT NULL,id TEXT NOT NULL,scope_key BLOB NOT NULL,v3_misses INTEGER NOT NULL DEFAULT 0 CHECK(v3_misses BETWEEN 0 AND 2),v3_completed_sweep TEXT NOT NULL DEFAULT '',PRIMARY KEY(account,region,kind,id,scope_key),FOREIGN KEY(account,region,kind,id) REFERENCES legacy_resources(account,region,kind,id) ON DELETE CASCADE) WITHOUT ROWID",
        "CREATE TABLE IF NOT EXISTS legacy_refreshes(account TEXT NOT NULL,region TEXT NOT NULL,kind TEXT NOT NULL,id TEXT NOT NULL,scope_key BLOB NOT NULL,PRIMARY KEY(account,region,kind,id,scope_key),FOREIGN KEY(account,region,kind,id) REFERENCES legacy_resources(account,region,kind,id) ON DELETE CASCADE) WITHOUT ROWID",
        "CREATE TABLE IF NOT EXISTS legacy_unknown_authorities(account TEXT NOT NULL,region TEXT NOT NULL,kind TEXT NOT NULL,id TEXT NOT NULL,scope_key BLOB NOT NULL,PRIMARY KEY(account,region,kind,id,scope_key),FOREIGN KEY(account,region,kind,id) REFERENCES legacy_resources(account,region,kind,id) ON DELETE CASCADE) WITHOUT ROWID",
        "CREATE TABLE IF NOT EXISTS legacy_observations(account TEXT NOT NULL,region TEXT NOT NULL,kind TEXT NOT NULL,id TEXT NOT NULL,scope_key BLOB NOT NULL,sweep_id TEXT NOT NULL,PRIMARY KEY(account,region,kind,id,scope_key,sweep_id),FOREIGN KEY(account,region,kind,id) REFERENCES legacy_resources(account,region,kind,id) ON DELETE CASCADE) WITHOUT ROWID",
    ] { sqlx::query(statement).execute(&mut **tx).await?; }
    let columns = sqlx::query("PRAGMA table_info(legacy_authorities)").fetch_all(&mut **tx).await?;
    for (name, statement) in [
        ("v3_misses", "ALTER TABLE legacy_authorities ADD COLUMN v3_misses INTEGER NOT NULL DEFAULT 0 CHECK(v3_misses BETWEEN 0 AND 2)"),
        ("v3_completed_sweep", "ALTER TABLE legacy_authorities ADD COLUMN v3_completed_sweep TEXT NOT NULL DEFAULT ''"),
    ] {
        let present = columns.iter().try_fold(false, |found, row| {
            Ok::<_, sqlx::Error>(found || row.try_get::<&str, _>("name")? == name)
        })?;
        if !present { sqlx::query(statement).execute(&mut **tx).await?; }
    }
    Ok(())
}

pub(super) async fn adopt(tx: &mut Transaction<'_, Sqlite>) -> Result<(), StoreError> {
    // Keep the existing canonical typed row, name and relationships as the explicit
    // unattributed baseline; never guess which historical source owned a field.
    sqlx::query("INSERT OR IGNORE INTO legacy_resources SELECT account,region,kind,id FROM resources").execute(&mut **tx).await?;
    sqlx::query("INSERT OR IGNORE INTO legacy_authorities(account,region,kind,id,scope_key) SELECT account,region,kind,id,scope_key FROM resource_scopes WHERE missing_complete<2").execute(&mut **tx).await?;
    Ok(())
}

pub(super) async fn demote_facets(
    tx: &mut Transaction<'_, Sqlite>, attributed: bool,
) -> Result<(), StoreError> {
    for statement in [
        "CREATE TABLE IF NOT EXISTS legacy_authority_history_v2 AS SELECT * FROM legacy_authorities WHERE 0",
        "INSERT INTO legacy_authority_history_v2 SELECT * FROM legacy_authorities",
        "CREATE TABLE IF NOT EXISTS legacy_unknown_authority_history_v2 AS SELECT * FROM legacy_unknown_authorities WHERE 0",
        "INSERT INTO legacy_unknown_authority_history_v2 SELECT * FROM legacy_unknown_authorities",
        "CREATE TABLE IF NOT EXISTS legacy_observation_history_v2 AS SELECT * FROM legacy_observations WHERE 0",
        "INSERT INTO legacy_observation_history_v2 SELECT * FROM legacy_observations",
        "CREATE TABLE IF NOT EXISTS legacy_work_authority_history_v2 AS SELECT sweep_id,scope_key,generation,authority_generation,authority_complete FROM work_queue WHERE 0",
        "INSERT INTO legacy_work_authority_history_v2 SELECT sweep_id,scope_key,generation,authority_generation,authority_complete FROM work_queue",
        "CREATE TABLE IF NOT EXISTS legacy_refresh_history_v2 AS SELECT * FROM legacy_refreshes WHERE 0",
        "INSERT INTO legacy_refresh_history_v2 SELECT * FROM legacy_refreshes",
    ] { sqlx::query(statement).execute(&mut **tx).await?; }
    for table in ["legacy_authorities", "legacy_unknown_authorities"] {
        let mut query = sqlx::QueryBuilder::<Sqlite>::new("DELETE FROM ");
        query.push(table).push(" AS pending WHERE EXISTS(SELECT 1 FROM resource_scopes owner WHERE owner.account=pending.account AND owner.region=pending.region AND owner.kind=pending.kind AND owner.id=pending.id AND owner.scope_key=pending.scope_key AND owner.missing_complete>=2)");
        query.build().execute(&mut **tx).await?;
    }
    adopt(tx).await?;
    if attributed {
        // A facet with no surviving owner row has no recoverable execution proof.
        // Keep its exact origin opaque rather than substituting another request.
        for table in ["legacy_authorities", "legacy_unknown_authorities"] {
            let mut query = sqlx::QueryBuilder::<Sqlite>::new("INSERT OR IGNORE INTO ");
            query.push(table).push("(account,region,kind,id,scope_key) SELECT facet.account,facet.region,facet.kind,facet.id,facet.source_key FROM resource_facets facet WHERE NOT EXISTS(SELECT 1 FROM resource_scopes owner WHERE owner.account=facet.account AND owner.region=facet.region AND owner.kind=facet.kind AND owner.id=facet.id AND owner.scope_key=facet.source_key)");
            query.build().execute(&mut **tx).await?;
        }
    }
    sqlx::query("INSERT OR IGNORE INTO legacy_unknown_authorities(account,region,kind,id,scope_key) SELECT pending.account,pending.region,pending.kind,pending.id,pending.scope_key FROM legacy_authorities pending WHERE NOT EXISTS(SELECT 1 FROM resource_scopes owner WHERE owner.account=pending.account AND owner.region=pending.region AND owner.kind=pending.kind AND owner.id=pending.id AND owner.scope_key=pending.scope_key)")
        .execute(&mut **tx).await?;
    // Old Complete outcomes and active dependencies are history, not V3 proofs.
    sqlx::query("DELETE FROM legacy_refreshes").execute(&mut **tx).await?;
    sqlx::query("DELETE FROM legacy_observations").execute(&mut **tx).await?;
    sqlx::query("UPDATE work_queue SET authority_generation=0,authority_complete=0").execute(&mut **tx).await?;
    sqlx::query("UPDATE legacy_authorities SET v3_misses=0,v3_completed_sweep=''").execute(&mut **tx).await?;
    let statements = if attributed {
        [
            "CREATE TABLE IF NOT EXISTS legacy_facet_history_v2 AS SELECT * FROM resource_facets WHERE 0",
            "INSERT INTO legacy_facet_history_v2 SELECT * FROM resource_facets",
        ]
    } else {
        [
            "CREATE TABLE IF NOT EXISTS legacy_facet_history_unattributed AS SELECT * FROM resource_facets WHERE 0",
            "INSERT INTO legacy_facet_history_unattributed SELECT * FROM resource_facets",
        ]
    };
    for statement in statements { sqlx::query(statement).execute(&mut **tx).await?; }
    Ok(())
}

pub(super) async fn observe_source(
    tx: &mut Transaction<'_, Sqlite>, key: &ResourceKey, source: &[u8], sweep: &SweepId,
) -> Result<(), StoreError> {
    sqlx::query("UPDATE legacy_authorities SET v3_misses=0,v3_completed_sweep='' WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 AND scope_key=?5")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .bind(source).execute(&mut **tx).await?;
    sqlx::query("INSERT OR IGNORE INTO legacy_observations(account,region,kind,id,scope_key,sweep_id) SELECT account,region,kind,id,?5,?6 FROM legacy_resources WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .bind(source).bind(sweep.as_str()).execute(&mut **tx).await?;
    Ok(())
}

pub(super) async fn observe_miss(
    tx: &mut Transaction<'_, Sqlite>, key: &ResourceKey, source: &[u8], sweep: &SweepId,
) -> Result<Option<bool>, StoreError> {
    let opaque: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM legacy_unknown_authorities WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 AND scope_key=?5)")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .bind(source).fetch_one(&mut **tx).await?;
    if opaque {
        sqlx::query("UPDATE legacy_authorities SET v3_completed_sweep=?6 WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 AND scope_key=?5")
            .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
            .bind(source).bind(sweep.as_str()).execute(&mut **tx).await?;
        return Ok(Some(false));
    }
    sqlx::query("UPDATE legacy_authorities SET v3_misses=min(v3_misses+1,2),v3_completed_sweep=?6 WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 AND scope_key=?5 AND v3_completed_sweep<>?6")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .bind(source).bind(sweep.as_str()).execute(&mut **tx).await?;
    let misses: Option<i64> = sqlx::query_scalar("SELECT v3_misses FROM legacy_authorities WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 AND scope_key=?5")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .bind(source).fetch_optional(&mut **tx).await?;
    Ok(misses.map(|misses| misses >= 2))
}

pub(super) async fn contains(tx: &mut Transaction<'_, Sqlite>, key: &ResourceKey) -> Result<bool, StoreError> {
    Ok(sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM legacy_resources WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4)")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .fetch_one(&mut **tx).await?)
}

pub(super) async fn complete_source(
    tx: &mut Transaction<'_, Sqlite>, scope: &[u8], sweep: &SweepId,
) -> Result<(), StoreError> {
    sqlx::query("INSERT OR IGNORE INTO legacy_refreshes SELECT account,region,kind,id,scope_key FROM legacy_observations WHERE scope_key=?1 AND sweep_id=?2")
        .bind(scope).bind(sweep.as_str()).execute(&mut **tx).await?;
    sqlx::query("DELETE FROM legacy_authorities AS pending WHERE scope_key=?1 AND (EXISTS(SELECT 1 FROM legacy_observations fresh WHERE fresh.account=pending.account AND fresh.region=pending.region AND fresh.kind=pending.kind AND fresh.id=pending.id AND fresh.scope_key=pending.scope_key AND fresh.sweep_id=?2) OR (pending.v3_misses>=2 AND EXISTS(SELECT 1 FROM resource_scopes current WHERE current.account=pending.account AND current.region=pending.region AND current.kind=pending.kind AND current.id=pending.id AND current.scope_key=pending.scope_key AND current.missing_complete>=2)))")
        .bind(scope).bind(sweep.as_str()).execute(&mut **tx).await?;
    Ok(())
}

pub(super) async fn retire_source(
    tx: &mut Transaction<'_, Sqlite>, key: &ResourceKey, source: &[u8],
) -> Result<(), StoreError> {
    sqlx::query("DELETE FROM legacy_authorities WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 AND scope_key=?5")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .bind(source).execute(&mut **tx).await?;
    // A proven loss of this exact parent origin retires even an opaque request
    // authority; unrelated fresh requests still cannot certify or remove it.
    sqlx::query("DELETE FROM legacy_unknown_authorities WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 AND scope_key=?5")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .bind(source).execute(&mut **tx).await?;
    sqlx::query("DELETE FROM legacy_refreshes WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 AND scope_key=?5")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .bind(source).execute(&mut **tx).await?;
    Ok(())
}

pub(super) async fn publish_ready(
    store: &BinaryStore, tx: &mut Transaction<'_, Sqlite>, at: i64,
) -> Result<ChangeStats, StoreError> {
    // An orphan baseline also needs a completed attributed facet: unrelated
    // complete work cannot certify its buffered partial observations.
    let rows = sqlx::query("SELECT account,region,kind,id FROM legacy_resources baseline WHERE NOT EXISTS(SELECT 1 FROM legacy_unknown_authorities unknown WHERE unknown.account=baseline.account AND unknown.region=baseline.region AND unknown.kind=baseline.kind AND unknown.id=baseline.id) AND NOT EXISTS(SELECT 1 FROM legacy_authorities pending WHERE pending.account=baseline.account AND pending.region=baseline.region AND pending.kind=baseline.kind AND pending.id=baseline.id) AND EXISTS(SELECT 1 FROM resource_facets facet JOIN legacy_refreshes refreshed ON refreshed.account=facet.account AND refreshed.region=facet.region AND refreshed.kind=facet.kind AND refreshed.id=facet.id AND refreshed.scope_key=facet.source_key WHERE facet.account=baseline.account AND facet.region=baseline.region AND facet.kind=baseline.kind AND facet.id=baseline.id)")
        .fetch_all(&mut **tx).await?;
    let mut stats = ChangeStats::default();
    for row in rows {
        let key = ResourceKey { account: AccountId::parse(&row.try_get::<String, _>("account")?)?,
            region: Region::parse(&row.try_get::<String, _>("region")?)?, kind: Kind::parse(&row.try_get::<String, _>("kind")?)?,
            id: ResourceId::parse(&row.try_get::<String, _>("id")?)? };
        let aggregate = facets::load(&store.registry, tx, &key).await?;
        sqlx::query("DELETE FROM legacy_resources WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4")
            .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str()).execute(&mut **tx).await?;
        let published = super::commit::publish_aggregate(store, tx, &aggregate, at).await?;
        stats.resources_changed = stats.resources_changed.checked_add(published.resources_changed).ok_or(StoreError::Invalid("legacy resource change count overflow"))?;
        stats.edges_changed = stats.edges_changed.checked_add(published.edges_changed).ok_or(StoreError::Invalid("legacy edge count overflow"))?;
    }
    Ok(stats)
}

pub(super) async fn count(pool: &SqlitePool) -> Result<u64, StoreError> {
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM legacy_resources").fetch_one(pool).await?;
    u64::try_from(count).map_err(|_| StoreError::Invalid("unattributed resource count exceeds bounds"))
}
