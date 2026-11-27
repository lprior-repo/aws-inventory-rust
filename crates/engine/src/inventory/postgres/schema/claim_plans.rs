use super::super::lease::CLAIM;
use super::shared;
use inventory_harness::postgres::{TestDatabase, admin_url};
use serde_json::Value;
use sqlx::{AssertSqlSafe, Connection, PgConnection, Row};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn sparse_ready_work_uses_claim_index() -> TestResult {
    indexed_claim(0).await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn sparse_expired_leases_use_claim_index() -> TestResult {
    indexed_claim(1).await
}

async fn indexed_claim(eligible_state: i64) -> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let plan = explain(database.url(), eligible_state).await;
    database.finish().await?;
    let plan = plan?;
    let candidate = plan.pointer("/0/Plan/Plans").and_then(Value::as_array)
        .and_then(|nodes| nodes.iter().find(|node| node["Subplan Name"] == "CTE candidate"))
        .ok_or("missing claim candidate subplan")?;
    let mut nodes = vec![candidate];
    let mut indexed = false;
    while let Some(node) = nodes.pop() {
        assert_ne!(node["Node Type"], "Seq Scan", "{candidate}");
        indexed |= node["Index Name"] == "work_queue_ready";
        if let Some(children) = node["Plans"].as_array() {
            nodes.extend(children);
        }
    }
    assert!(indexed, "{candidate}");
    Ok(())
}

async fn explain(url: &str, eligible_state: i64) -> TestResult<Value> {
    let mut connection = PgConnection::connect(url).await?;
    let mut tx = connection.begin().await?;
    sqlx::query("CREATE SCHEMA inventory").execute(&mut *tx).await?;
    shared::create(&mut tx).await?;
    sqlx::query("SET LOCAL search_path = inventory").execute(&mut *tx).await?;
    sqlx::query(
        "INSERT INTO work_queue(sweep_id,scope_key,bindings,state,generation,lease_until,last_complete) \
         SELECT 'sweep',convert_to(i::text,'UTF8'),''::bytea,\
         CASE WHEN i % 200 = 0 THEN $1::bigint ELSE 2 END,0,10,i FROM generate_series(1,2000) g(i)",
    )
    .bind(eligible_state)
    .execute(&mut *tx)
    .await?;
    sqlx::query("ANALYZE work_queue").execute(&mut *tx).await?;
    let row = sqlx::query(AssertSqlSafe(format!("EXPLAIN (FORMAT JSON) {CLAIM}")))
        .bind("sweep")
        .bind(1000_i64)
        .bind(2000_i64)
        .bind(10_i64)
        .fetch_one(&mut *tx)
        .await?;
    tx.rollback().await?;
    connection.close().await?;
    Ok(serde_json::from_slice(row.try_get_raw(0)?.as_bytes()?)?)
}
