mod projection;

use super::PgStore;
use crate::inventory::{StoreError, SweepId};
use sqlx::{Postgres, QueryBuilder, Transaction};

pub(super) use projection::project;

const TREE: &str = concat!(
    "CREATE TEMP TABLE inventory_retire_keys ON COMMIT DROP AS WITH RECURSIVE candidates(account,region,kind,id) AS (",
    "SELECT r.account,r.region,r.kind,r.id FROM resources r WHERE NOT EXISTS(SELECT 1 FROM resource_scopes s WHERE (s.account,s.region,s.kind,s.id)=(r.account,r.region,r.kind,r.id) AND s.missing_complete<2) UNION ",
    "SELECT p.child_account,p.child_region,p.child_kind,p.child_id FROM resource_parents p JOIN candidates c ON (p.parent_account,p.parent_region,p.parent_kind,p.parent_id)=(c.account,c.region,c.kind,c.id)), ",
    "survivors(account,region,kind,id) AS (SELECT c.* FROM candidates c JOIN resource_scopes s USING(account,region,kind,id) WHERE s.missing_complete<2 AND (NOT EXISTS(SELECT 1 FROM resource_parents p WHERE (p.child_account,p.child_region,p.child_kind,p.child_id,p.scope_key)=(c.account,c.region,c.kind,c.id,s.scope_key)) OR EXISTS(SELECT 1 FROM resource_parents p WHERE (p.child_account,p.child_region,p.child_kind,p.child_id,p.scope_key)=(c.account,c.region,c.kind,c.id,s.scope_key) AND NOT EXISTS(SELECT 1 FROM candidates parent WHERE (parent.account,parent.region,parent.kind,parent.id)=(p.parent_account,p.parent_region,p.parent_kind,p.parent_id)))) UNION ",
    "SELECT c.* FROM survivors parent JOIN resource_parents p ON (p.parent_account,p.parent_region,p.parent_kind,p.parent_id)=(parent.account,parent.region,parent.kind,parent.id) JOIN candidates c ON (c.account,c.region,c.kind,c.id)=(p.child_account,p.child_region,p.child_kind,p.child_id) JOIN resource_scopes s ON (s.account,s.region,s.kind,s.id,s.scope_key)=(c.account,c.region,c.kind,c.id,p.scope_key) WHERE s.missing_complete<2) SELECT * FROM candidates EXCEPT SELECT * FROM survivors"
);
const SOURCES: &str = concat!(
    "CREATE TEMP TABLE inventory_retire_sources ON COMMIT DROP AS SELECT s.account,s.region,s.kind,s.id,s.scope_key FROM resource_scopes s JOIN resource_facets f ON (f.account,f.region,f.kind,f.id,f.source_key)=(s.account,s.region,s.kind,s.id,s.scope_key) WHERE s.missing_complete>=2 AND s.completed_sweep=$1 AND NOT EXISTS(SELECT 1 FROM inventory_retire_keys d WHERE (d.account,d.region,d.kind,d.id)=(s.account,s.region,s.kind,s.id)) UNION ",
    "SELECT p.child_account,p.child_region,p.child_kind,p.child_id,p.scope_key FROM resource_parents p JOIN inventory_retire_keys d ON (d.account,d.region,d.kind,d.id)=(p.parent_account,p.parent_region,p.parent_kind,p.parent_id) WHERE NOT EXISTS(SELECT 1 FROM inventory_retire_keys child WHERE (child.account,child.region,child.kind,child.id)=(p.child_account,p.child_region,p.child_kind,p.child_id))"
);

pub(super) async fn stage(
    tx: &mut Transaction<'_, Postgres>,
    sweep: &SweepId,
) -> Result<(), StoreError> {
    sqlx::query(TREE).execute(&mut **tx).await?;
    sqlx::query("ALTER TABLE inventory_retire_keys ADD PRIMARY KEY(account,region,kind,id)")
        .execute(&mut **tx)
        .await?;
    sqlx::query(SOURCES)
        .bind(sweep.as_str())
        .execute(&mut **tx)
        .await?;
    sqlx::query(
        "ALTER TABLE inventory_retire_sources ADD PRIMARY KEY(account,region,kind,id,scope_key)",
    )
    .execute(&mut **tx)
    .await?;
    sqlx::query("UPDATE resource_scopes s SET missing_complete=greatest(s.missing_complete,2) FROM inventory_retire_sources r WHERE (s.account,s.region,s.kind,s.id,s.scope_key)=(r.account,r.region,r.kind,r.id,r.scope_key) AND s.missing_complete<2")
        .execute(&mut **tx).await?;
    projection::stage(tx).await
}

pub(super) async fn remove(
    store: &PgStore,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<u64, StoreError> {
    let kinds: Vec<String> =
        sqlx::query_scalar("SELECT DISTINCT kind FROM inventory_retire_keys ORDER BY kind")
            .fetch_all(&mut **tx)
            .await?;
    for kind in kinds {
        remove_kind(store, tx, kind).await?;
    }
    Ok(sqlx::query("DELETE FROM resources r USING inventory_retire_keys d WHERE (r.account,r.region,r.kind,r.id)=(d.account,d.region,d.kind,d.id)")
        .execute(&mut **tx).await?.rows_affected())
}

async fn remove_kind(
    store: &PgStore,
    tx: &mut Transaction<'_, Postgres>,
    kind: String,
) -> Result<(), StoreError> {
    let schema = store
        .registry
        .schemas()
        .iter()
        .find(|schema| schema.kind().as_str() == kind)
        .ok_or(StoreError::Invalid("retired resource kind has no schema"))?;
    let mut query = QueryBuilder::new("DELETE FROM \"");
    query
        .push(schema.table().as_str())
        .push("\" p USING inventory_retire_keys d WHERE d.kind=")
        .push_bind(kind)
        .push(" AND (p.account,p.region,p.id)=(d.account,d.region,d.id)");
    query.build().execute(&mut **tx).await?;
    Ok(())
}
