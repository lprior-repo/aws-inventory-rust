use super::{StoreError, work};
use inventory_core::inventory::{AccountId, Kind, Region, ResourceId, ResourceKey};
use sqlx::{QueryBuilder, Row, Sqlite, Transaction};

pub(super) async fn migrate(tx: &mut Transaction<'_, Sqlite>) -> Result<(), StoreError> {
    let columns = sqlx::query("PRAGMA table_info(resource_scopes)")
        .fetch_all(&mut **tx)
        .await?;
    let has_completed = columns.iter().try_fold(false, |found, row| {
        Ok::<_, sqlx::Error>(found || row.try_get::<String, _>("name")? == "completed_sweep")
    })?;
    if !has_completed {
        sqlx::query(
            "ALTER TABLE resource_scopes ADD COLUMN completed_sweep TEXT NOT NULL DEFAULT ''",
        )
        .execute(&mut **tx)
        .await?;
    }
    let done: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM metadata WHERE key='semantic_authority_v1')",
    )
    .fetch_one(&mut **tx)
    .await?;
    if done && has_completed {
        return Ok(());
    }
    let rows = sqlx::query("SELECT owner.account,owner.region,owner.kind,owner.id,owner.scope_key,owner.missing_complete,(SELECT work.bindings FROM work_queue work WHERE work.scope_key=owner.scope_key AND work.sweep_id=owner.observed_sweep) AS bindings FROM resource_scopes owner")
        .fetch_all(&mut **tx).await?;
    for row in rows {
        let old: Vec<u8> = row.try_get("scope_key")?;
        let scope = work::persisted_scope(&old)?;
        if scope.context().is_none() && scope.parent().is_none() {
            continue;
        }
        let key = ResourceKey {
            account: AccountId::parse(&row.try_get::<String, _>("account")?)?,
            region: Region::parse(&row.try_get::<String, _>("region")?)?,
            kind: Kind::parse(&row.try_get::<String, _>("kind")?)?,
            id: ResourceId::parse(&row.try_get::<String, _>("id")?)?,
        };
        let Some(bindings) = row.try_get::<Option<Vec<u8>>, _>("bindings")? else {
            if row.try_get::<i64, _>("missing_complete")? < 2 {
                retain_unknown(tx, &key, &old).await?;
            }
            continue;
        };
        let new = work::ownership_scope_bytes(&scope, &bindings)?;
        if old != new {
            replace(tx, &key, &old, &new).await?;
        }
    }
    mark(tx).await
}

pub(super) async fn mark(tx: &mut Transaction<'_, Sqlite>) -> Result<(), StoreError> {
    sqlx::query("INSERT OR IGNORE INTO metadata(key,value) VALUES('semantic_authority_v1',X'01')")
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn retain_unknown(
    tx: &mut Transaction<'_, Sqlite>,
    key: &ResourceKey,
    scope: &[u8],
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT OR IGNORE INTO legacy_resources(account,region,kind,id) VALUES(?1,?2,?3,?4)",
    )
    .bind(key.account.as_str())
    .bind(key.region.as_str())
    .bind(key.kind.as_str())
    .bind(key.id.as_str())
    .execute(&mut **tx)
    .await?;
    for table in ["legacy_authorities", "legacy_unknown_authorities"] {
        let mut query = QueryBuilder::<Sqlite>::new("INSERT OR IGNORE INTO ");
        query
            .push(table)
            .push("(account,region,kind,id,scope_key) VALUES(")
            .push_bind(key.account.as_str())
            .push(",")
            .push_bind(key.region.as_str())
            .push(",")
            .push_bind(key.kind.as_str())
            .push(",")
            .push_bind(key.id.as_str())
            .push(",")
            .push_bind(scope)
            .push(")")
            .build()
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

async fn replace(
    tx: &mut Transaction<'_, Sqlite>,
    key: &ResourceKey,
    old: &[u8],
    new: &[u8],
) -> Result<(), StoreError> {
    sqlx::query("INSERT INTO resource_scopes(account,region,kind,id,scope_key,last_seen,missing_complete,observed_sweep,completed_sweep) SELECT account,region,kind,id,?2,last_seen,missing_complete,observed_sweep,completed_sweep FROM resource_scopes WHERE scope_key=?1 AND account=?3 AND region=?4 AND kind=?5 AND id=?6 ON CONFLICT(account,region,kind,id,scope_key) DO UPDATE SET missing_complete=min(resource_scopes.missing_complete,excluded.missing_complete),observed_sweep=CASE WHEN excluded.missing_complete<resource_scopes.missing_complete OR (excluded.missing_complete=resource_scopes.missing_complete AND excluded.last_seen>resource_scopes.last_seen) THEN excluded.observed_sweep ELSE resource_scopes.observed_sweep END,completed_sweep=CASE WHEN excluded.last_seen>resource_scopes.last_seen THEN excluded.completed_sweep ELSE resource_scopes.completed_sweep END,last_seen=max(resource_scopes.last_seen,excluded.last_seen)")
        .bind(old).bind(new).bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .execute(&mut **tx).await?;
    sqlx::query("INSERT OR IGNORE INTO resource_parents SELECT parent_account,parent_region,parent_kind,parent_id,child_account,child_region,child_kind,child_id,?2 FROM resource_parents WHERE scope_key=?1 AND child_account=?3 AND child_region=?4 AND child_kind=?5 AND child_id=?6")
        .bind(old).bind(new).bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .execute(&mut **tx).await?;
    let values = [
        key.account.as_str(),
        key.region.as_str(),
        key.kind.as_str(),
        key.id.as_str(),
    ];
    for (table, columns) in [
        (
            "resource_parents",
            ["child_account", "child_region", "child_kind", "child_id"],
        ),
        ("resource_scopes", ["account", "region", "kind", "id"]),
    ] {
        let mut query = QueryBuilder::<Sqlite>::new("DELETE FROM ");
        query.push(table).push(" WHERE scope_key=").push_bind(old);
        for (column, value) in columns.into_iter().zip(values) {
            query.push(" AND ").push(column).push("=").push_bind(value);
        }
        query.build().execute(&mut **tx).await?;
    }
    Ok(())
}
