use super::commit::{enqueue, quote};
use super::work::{ChangeStats, WorkLease};
use super::{BinaryStore, StoreError};
use inventory_core::inventory::ResourceKey;
use sqlx::{QueryBuilder, Row, Sqlite, Transaction};

pub(super) async fn complete_scope(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    lease: &WorkLease,
    at: i64,
) -> Result<ChangeStats, StoreError> {
    let scope = super::work::scope_bytes(lease.scope())?;
    let rows = sqlx::query("SELECT account,region,kind,id,missing_complete FROM resource_scopes WHERE scope_key=?1 AND observed_sweep<>?2")
        .bind(&scope).bind(lease.sweep().as_str()).fetch_all(&mut **tx).await?;
    let mut stats = ChangeStats::default();
    for row in rows {
        let account: String = row.try_get("account")?;
        let region: String = row.try_get("region")?;
        let kind: String = row.try_get("kind")?;
        let id: String = row.try_get("id")?;
        let misses: i64 = row.try_get("missing_complete")?;
        sqlx::query("UPDATE resource_scopes SET missing_complete=missing_complete+1,last_seen=?5 WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 AND scope_key=?6")
            .bind(&account).bind(&region).bind(&kind).bind(&id).bind(at).bind(&scope).execute(&mut **tx).await?;
        let remaining: i64 = sqlx::query("SELECT COUNT(*) FROM resource_scopes WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 AND missing_complete<2")
            .bind(&account).bind(&region).bind(&kind).bind(&id).fetch_one(&mut **tx).await?.try_get(0)?;
        if misses >= 1 && remaining == 0 {
            let key = ResourceKey {
                account: inventory_core::inventory::AccountId::parse(&account)?,
                region: inventory_core::inventory::Region::parse(&region)?,
                kind: inventory_core::inventory::Kind::parse(&kind)?,
                id: inventory_core::inventory::ResourceId::parse(&id)?,
            };
            let deleted = delete_tree(store, tx, &key).await?;
            stats.resources_deleted = stats
                .resources_deleted
                .checked_add(deleted)
                .ok_or(StoreError::Invalid("deletion count overflow"))?;
        }
    }
    Ok(stats)
}

async fn delete_tree(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    root: &ResourceKey,
) -> Result<u64, StoreError> {
    let rows = sqlx::query("WITH RECURSIVE tree(account,region,kind,id) AS (SELECT account,region,kind,id FROM resources WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 UNION SELECT parent.child_account,parent.child_region,parent.child_kind,parent.child_id FROM resource_parents parent JOIN tree ON parent.parent_account=tree.account AND parent.parent_region=tree.region AND parent.parent_kind=tree.kind AND parent.parent_id=tree.id) SELECT account,region,kind,id FROM tree")
        .bind(root.account.as_str()).bind(root.region.as_str()).bind(root.kind.as_str()).bind(root.id.as_str()).fetch_all(&mut **tx).await?;
    let keys = rows
        .into_iter()
        .map(resource_key_from_row)
        .collect::<Result<Vec<_>, StoreError>>()?;
    for key in &keys {
        enqueue(tx, key, true).await?;
    }
    for key in &keys {
        delete_resource_rows(store, tx, key).await?;
    }
    u64::try_from(keys.len()).map_err(|_| StoreError::Invalid("resource deletion count overflow"))
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
