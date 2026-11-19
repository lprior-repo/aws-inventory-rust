#[path = "inventory_store/support.rs"]
mod support;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, AssertSqlSafe};
use support::*;

#[tokio::test]
async fn work_queue_claim_uses_ready_work_index() -> Result<(), Box<dyn std::error::Error>> {
    let (directory, _store, _catalog) = fixture().await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='index' AND name='work_queue_claim'",
    )
    .fetch_optional(&pool)
    .await?;
    if row.is_none() {
        pool.close().await;
        return Err("work_queue_claim index does not exist".into());
    }

    // Test that the index supports the state-filter portion of the claim query
    // (the key optimization for ES-09: avoiding scans of completed rows)
    let plan = explain_plan(&pool,
        "SELECT scope_key FROM work_queue WHERE state=0 ORDER BY scope_key LIMIT 1")
        .await?;

    pool.close().await;

    if !plan.contains("INDEX work_queue_claim") {
        return Err(format!("state-filter query plan does not use work_queue_claim index: {}", plan).into());
    }
    Ok(())
}

#[tokio::test]
async fn resource_scopes_completeness_uses_scope_key_index() -> Result<(), Box<dyn std::error::Error>> {
    let (directory, _store, _catalog) = fixture().await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='index' AND name='resource_scopes_key'",
    )
    .fetch_optional(&pool)
    .await?;
    if row.is_none() {
        pool.close().await;
        return Err("resource_scopes_key index does not exist".into());
    }

    let plan = explain_plan(&pool,
        "SELECT current.account,current.region,current.kind,current.id,current.missing_complete FROM resource_scopes current WHERE current.scope_key=X'00'")
        .await?;

    pool.close().await;

    if !plan.contains("INDEX resource_scopes_key") {
        return Err(format!("resource_scopes scope_key query plan does not use index: {}", plan).into());
    }
    Ok(())
}

#[tokio::test]
async fn unresolved_refs_exact_kind_uses_target_index() -> Result<(), Box<dyn std::error::Error>> {
    let (directory, _store, _catalog) = fixture().await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='index' AND name='unresolved_refs_target'",
    )
    .fetch_optional(&pool)
    .await?;
    if row.is_none() {
        pool.close().await;
        return Err("unresolved_refs_target index does not exist".into());
    }

    let plan = explain_plan(&pool,
        "SELECT from_account,from_region,from_kind,from_id,target_kind,target_kinds,target_path,target_value,relation,source FROM unresolved_refs WHERE target_scope=X'00' AND target_kind='AWS::Example::Thing'")
        .await?;

    pool.close().await;

    if !plan.contains("INDEX unresolved_refs_target") {
        return Err(format!("exact kind query plan does not use unresolved_refs_target index: {}", plan).into());
    }
    Ok(())
}

#[tokio::test]
async fn unresolved_refs_blob_kind_uses_target_index() -> Result<(), Box<dyn std::error::Error>> {
    let (directory, _store, _catalog) = fixture().await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;

    let plan = explain_plan(&pool,
        "SELECT from_account,from_region,from_kind,from_id,target_kind,target_kinds,target_path,target_value,relation,source FROM unresolved_refs WHERE target_scope=X'00' AND target_kind<>'AWS::Example::Thing' AND target_kinds<>X''")
        .await?;

    pool.close().await;

    if !plan.contains("INDEX unresolved_refs_target") {
        return Err(format!("blob kind query plan does not use unresolved_refs_target index: {}", plan).into());
    }
    Ok(())
}

#[tokio::test]
async fn es10_split_query_result_set_equivalence() -> Result<(), Box<dyn std::error::Error>> {
    let (directory, _store, _catalog) = fixture().await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;

    let scope = reference_scope_bytes("123456789012", "us-east-1")?;

    // Insert source resources
    for id in ["f1", "f2", "f3"] {
        sqlx::query(
            "INSERT INTO resources(account,region,kind,id,name,content_hash,first_seen,last_changed) VALUES('a','r','k',?1,'n',X'00',0,0)"
        )
        .bind(id)
        .execute(&pool)
        .await?;
    }

    // Insert test data: one exact match, one blob match, one neither
    sqlx::query(
        "INSERT INTO unresolved_refs(from_account,from_region,from_kind,from_id,target_scope,target_kind,target_path,target_value,relation,source) VALUES('a','r','k','f1',?1,'AWS::Example::Thing','p','v1','references','s')"
    )
    .bind(&scope)
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT INTO unresolved_refs(from_account,from_region,from_kind,from_id,target_scope,target_kind,target_path,target_value,relation,source,target_kinds) VALUES('a','r','k','f2',?1,'AWS::Other::Kind','p','v2','references','s','blob')"
    )
    .bind(&scope)
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT INTO unresolved_refs(from_account,from_region,from_kind,from_id,target_scope,target_kind,target_path,target_value,relation,source) VALUES('a','r','k','f3',?1,'AWS::Other::Kind','p','v3','references','s')"
    )
    .bind(&scope)
    .execute(&pool)
    .await?;

    // Original query with OR
    let original_rows = sqlx::query(
        "SELECT from_id FROM unresolved_refs WHERE target_scope=? AND (target_kind=? OR target_kinds<>X'') ORDER BY from_id",
    )
    .bind(&scope)
    .bind("AWS::Example::Thing")
    .fetch_all(&pool)
    .await?;

    // Split queries
    let exact_rows = sqlx::query(
        "SELECT from_id FROM unresolved_refs WHERE target_scope=? AND target_kind=? ORDER BY from_id",
    )
    .bind(&scope)
    .bind("AWS::Example::Thing")
    .fetch_all(&pool)
    .await?;

    let blob_rows = sqlx::query(
        "SELECT from_id FROM unresolved_refs WHERE target_scope=? AND target_kind<>? AND target_kinds<>X'' ORDER BY from_id",
    )
    .bind(&scope)
    .bind("AWS::Example::Thing")
    .fetch_all(&pool)
    .await?;

    pool.close().await;

    if original_rows.len() != exact_rows.len() + blob_rows.len() {
        return Err(format!(
            "row count mismatch: original={}, split={}",
            original_rows.len(),
            exact_rows.len() + blob_rows.len()
        ).into());
    }

    let mut original_ids = Vec::new();
    for row in &original_rows {
        let id: String = row.try_get("from_id").map_err(|e: sqlx::Error| format!("try_get failed: {}", e))?;
        original_ids.push(id);
    }

    let mut split_ids = Vec::new();
    for row in &exact_rows {
        let id: String = row.try_get("from_id").map_err(|e: sqlx::Error| format!("try_get failed: {}", e))?;
        split_ids.push(id);
    }
    for row in &blob_rows {
        let id: String = row.try_get("from_id").map_err(|e: sqlx::Error| format!("try_get failed: {}", e))?;
        split_ids.push(id);
    }

    if original_ids != split_ids {
        return Err(format!(
            "row content mismatch: original={:?}, split={:?}",
            original_ids, split_ids
        ).into());
    }

    Ok(())
}

async fn explain_plan(pool: &sqlx::SqlitePool, query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let plan_rows = sqlx::query(AssertSqlSafe(format!("EXPLAIN QUERY PLAN {}", query)))
        .fetch_all(pool)
        .await?;

    let mut plan = String::new();
    for row in &plan_rows {
        let detail: String = row.try_get("detail").map_err(|e: sqlx::Error| format!("try_get detail failed: {}", e))?;
        if !plan.is_empty() {
            plan.push('\n');
        }
        plan.push_str(&detail);
    }
    Ok(plan)
}

fn reference_scope_bytes(account: &str, region: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    let account_len = u32::try_from(account.len()).map_err(|e| format!("account len conversion failed: {}", e))?;
    bytes.extend_from_slice(&account_len.to_be_bytes());
    bytes.extend_from_slice(account.as_bytes());
    let region_len = u32::try_from(region.len()).map_err(|e| format!("region len conversion failed: {}", e))?;
    bytes.extend_from_slice(&region_len.to_be_bytes());
    bytes.extend_from_slice(region.as_bytes());
    Ok(bytes)
}