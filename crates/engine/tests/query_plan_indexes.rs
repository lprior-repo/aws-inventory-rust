#[path = "inventory_store/support.rs"]
#[allow(
    dead_code,
    reason = "shared via #[path]; each test target uses a subset of the helpers"
)]
mod support;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{AssertSqlSafe, Row};
use support::*;

#[tokio::test]
async fn work_queue_claim_ready_branch_uses_index() -> Result<(), Box<dyn std::error::Error>> {
    let (directory, _store, _catalog) = fixture().await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='index' AND name='work_queue_ready'",
    )
    .fetch_optional(&pool)
    .await?;
    if row.is_none() {
        pool.close().await;
        return Err("work_queue_ready index does not exist".into());
    }

    // Populate table so optimizer has statistics
    seed_work_queue(&pool).await?;

    // Test the ready branch of the claim query (state=0)
    let plan = explain_plan(&pool,
        "SELECT scope_key,token,bindings,generation,authority_complete,context FROM work_queue WHERE sweep_id='s1' AND state=0 ORDER BY scope_key LIMIT 1")
        .await?;

    pool.close().await;

    // Must use the work_queue_ready index with SEARCH, not PK or SCAN
    if plan.contains("PRIMARY KEY") {
        return Err(format!("ready branch uses PK, not work_queue_ready index: {}", plan).into());
    }
    if !plan.contains("SEARCH") || !plan.contains("work_queue_ready") {
        return Err(format!("ready branch does not use work_queue_ready index: {}", plan).into());
    }
    if !plan.contains("sweep_id=? AND state=?") {
        return Err(format!(
            "ready branch does not filter on sweep_id and state: {}",
            plan
        )
        .into());
    }
    Ok(())
}

#[tokio::test]
async fn work_queue_claim_expired_branch_is_bounded() -> Result<(), Box<dyn std::error::Error>> {
    let (directory, _store, _catalog) = fixture().await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;

    // Populate table so optimizer has statistics
    seed_work_queue(&pool).await?;

    // Test the expired lease branch of the claim query (state=1 AND lease_until<=now)
    let plan = explain_plan(&pool,
        "SELECT scope_key,token,bindings,generation,authority_complete,context FROM work_queue WHERE sweep_id='s1' AND state=1 AND lease_until<=1000 ORDER BY scope_key LIMIT 1")
        .await?;

    pool.close().await;

    // Must use work_queue_ready with SEARCH, not PK or SCAN
    // SQLite chooses work_queue_ready over work_queue_expired because it can
    // satisfy ORDER BY scope_key without a temp B-tree.
    if plan.contains("PRIMARY KEY") {
        return Err(format!("expired branch uses PK, not work_queue_ready: {}", plan).into());
    }
    if !plan.contains("SEARCH") || !plan.contains("work_queue_ready") {
        return Err(format!("expired branch does not use work_queue_ready: {}", plan).into());
    }
    if !plan.contains("sweep_id=? AND state=?") {
        return Err(format!(
            "expired branch does not filter on sweep_id and state: {}",
            plan
        )
        .into());
    }
    Ok(())
}

#[tokio::test]
async fn es09_claim_split_result_set_equivalence() -> Result<(), Box<dyn std::error::Error>> {
    let (directory, _store, _catalog) = fixture().await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;

    // Insert work queue rows: completed, pending, ready, expired lease
    // state=2 is completed, state=1 is leased, state=0 is ready
    sqlx::query(
        "INSERT INTO work_queue(sweep_id,scope_key,state,generation,lease_until,bindings) VALUES('s1',X'01',2,1,0,X'')"
    ).execute(&pool).await?; // completed
    sqlx::query(
        "INSERT INTO work_queue(sweep_id,scope_key,state,generation,lease_until,bindings) VALUES('s1',X'02',1,1,999999,X'')"
    ).execute(&pool).await?; // leased, not expired
    sqlx::query(
        "INSERT INTO work_queue(sweep_id,scope_key,state,generation,lease_until,bindings) VALUES('s1',X'03',0,1,0,X'')"
    ).execute(&pool).await?; // ready
    sqlx::query(
        "INSERT INTO work_queue(sweep_id,scope_key,state,generation,lease_until,bindings) VALUES('s1',X'04',1,1,500,X'')"
    ).execute(&pool).await?; // expired lease (now=1000)
    sqlx::query(
        "INSERT INTO work_queue(sweep_id,scope_key,state,generation,lease_until,bindings) VALUES('s1',X'05',0,1,0,X'')"
    ).execute(&pool).await?; // ready (higher scope_key)

    let now = 1000_i64;

    // Original OR query
    let original = sqlx::query(
        "SELECT scope_key FROM work_queue WHERE sweep_id='s1' AND (state=0 OR (state=1 AND lease_until<=?)) ORDER BY scope_key LIMIT 1"
    )
    .bind(now)
    .fetch_optional(&pool)
    .await?;

    // Split queries
    let ready = sqlx::query(
        "SELECT scope_key FROM work_queue WHERE sweep_id='s1' AND state=0 ORDER BY scope_key LIMIT 1"
    )
    .fetch_optional(&pool)
    .await?;

    let expired = sqlx::query(
        "SELECT scope_key FROM work_queue WHERE sweep_id='s1' AND state=1 AND lease_until<=? ORDER BY scope_key LIMIT 1"
    )
    .bind(now)
    .fetch_optional(&pool)
    .await?;

    pool.close().await;

    // Compute split result
    let split = match (ready, expired) {
        (Some(r), Some(e)) => {
            let r_key: Vec<u8> = r.try_get("scope_key")?;
            let e_key: Vec<u8> = e.try_get("scope_key")?;
            if r_key < e_key { r } else { e }
        }
        (Some(r), None) => r,
        (None, Some(e)) => e,
        (None, None) => return Err("split returned no row".into()),
    };

    // Compare
    let original_key: Vec<u8> = original
        .ok_or("original returned no row")?
        .try_get("scope_key")?;
    let split_key: Vec<u8> = split.try_get("scope_key")?;

    if original_key != split_key {
        return Err(format!(
            "claim split mismatch: original={:?}, split={:?}",
            original_key, split_key
        )
        .into());
    }

    Ok(())
}

#[tokio::test]
async fn es09_claim_skips_completed_rows() -> Result<(), Box<dyn std::error::Error>> {
    let (directory, _store, _catalog) = fixture().await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;

    // Insert many completed rows with lower scope_keys than the ready row
    for i in 0..10u8 {
        sqlx::query(
            "INSERT INTO work_queue(sweep_id,scope_key,state,generation,lease_until,bindings) VALUES('s1',?,2,1,0,X'')"
        )
        .bind([i])
        .execute(&pool)
        .await?;
    }

    // One ready row with higher scope_key
    sqlx::query(
        "INSERT INTO work_queue(sweep_id,scope_key,state,generation,lease_until,bindings) VALUES('s1',X'FF',0,1,0,X'')"
    )
    .execute(&pool)
    .await?;

    // Split queries
    let ready = sqlx::query(
        "SELECT scope_key FROM work_queue WHERE sweep_id='s1' AND state=0 ORDER BY scope_key LIMIT 1"
    )
    .fetch_optional(&pool)
    .await?;

    let expired = sqlx::query(
        "SELECT scope_key FROM work_queue WHERE sweep_id='s1' AND state=1 AND lease_until<=1000 ORDER BY scope_key LIMIT 1"
    )
    .bind(1000_i64)
    .fetch_optional(&pool)
    .await?;

    pool.close().await;

    // The ready query should find the ready row, not any completed row
    let ready_key: Vec<u8> = ready.ok_or("ready returned no row")?.try_get("scope_key")?;
    assert!(expired.is_none(), "expired should return no row");
    if ready_key != [0xFF] {
        return Err(format!(
            "claim returned wrong row: expected [0xFF], got {:?}",
            ready_key
        )
        .into());
    }

    Ok(())
}

#[tokio::test]
async fn resource_scopes_completeness_uses_scope_key_index()
-> Result<(), Box<dyn std::error::Error>> {
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
        return Err(format!(
            "resource_scopes scope_key query plan does not use index: {}",
            plan
        )
        .into());
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
        return Err(format!(
            "exact kind query plan does not use unresolved_refs_target index: {}",
            plan
        )
        .into());
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
        return Err(format!(
            "blob kind query plan does not use unresolved_refs_target index: {}",
            plan
        )
        .into());
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
        )
        .into());
    }

    let mut original_ids = Vec::new();
    for row in &original_rows {
        let id: String = row
            .try_get("from_id")
            .map_err(|e: sqlx::Error| format!("try_get failed: {}", e))?;
        original_ids.push(id);
    }

    let mut split_ids = Vec::new();
    for row in &exact_rows {
        let id: String = row
            .try_get("from_id")
            .map_err(|e: sqlx::Error| format!("try_get failed: {}", e))?;
        split_ids.push(id);
    }
    for row in &blob_rows {
        let id: String = row
            .try_get("from_id")
            .map_err(|e: sqlx::Error| format!("try_get failed: {}", e))?;
        split_ids.push(id);
    }

    if original_ids != split_ids {
        return Err(format!(
            "row content mismatch: original={:?}, split={:?}",
            original_ids, split_ids
        )
        .into());
    }

    Ok(())
}

async fn explain_plan(
    pool: &sqlx::SqlitePool,
    query: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let plan_rows = sqlx::query(AssertSqlSafe(format!("EXPLAIN QUERY PLAN {}", query)))
        .fetch_all(pool)
        .await?;

    let mut plan = String::new();
    for row in &plan_rows {
        let detail: String = row
            .try_get("detail")
            .map_err(|e: sqlx::Error| format!("try_get detail failed: {}", e))?;
        if !plan.is_empty() {
            plan.push('\n');
        }
        plan.push_str(&detail);
    }
    Ok(plan)
}

async fn seed_work_queue(pool: &sqlx::SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    // Insert many completed rows so optimizer has statistics
    for i in 0..100u8 {
        sqlx::query(
            "INSERT INTO work_queue(sweep_id,scope_key,state,generation,lease_until,bindings) VALUES('s1',?,2,1,0,X'')"
        )
        .bind([i])
        .execute(pool)
        .await?;
    }
    for i in 100..102u8 {
        sqlx::query(
            "INSERT INTO work_queue(sweep_id,scope_key,state,generation,lease_until,bindings) VALUES('s1',?,0,1,0,X'')"
        )
        .bind([i])
        .execute(pool)
        .await?;
    }
    sqlx::query("ANALYZE").execute(pool).await?;
    Ok(())
}

fn reference_scope_bytes(
    account: &str,
    region: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    let account_len = u32::try_from(account.len())
        .map_err(|e| format!("account len conversion failed: {}", e))?;
    bytes.extend_from_slice(&account_len.to_be_bytes());
    bytes.extend_from_slice(account.as_bytes());
    let region_len =
        u32::try_from(region.len()).map_err(|e| format!("region len conversion failed: {}", e))?;
    bytes.extend_from_slice(&region_len.to_be_bytes());
    bytes.extend_from_slice(region.as_bytes());
    Ok(bytes)
}
