use anyhow::Result;
use inventory_engine::store::Store;
use serde_json::Value;
use std::path::PathBuf;

#[tracing::instrument(skip_all)]
pub async fn report(path: PathBuf, scan_id: String) -> Result<Value> {
    super::require_database(&path)?;
    let store = Store::open(path).await?;
    let result = report_open(&store, &scan_id).await;
    store.close().await;
    result
}

#[tracing::instrument(skip_all)]
async fn report_open(store: &Store, scan_id: &str) -> Result<Value> {
    let status = store.scan_status(scan_id).await?;
    let summary = store.summary(scan_id).await?;
    let (counts, coverage, failures) = detail_queries(store, scan_id).await?;
    Ok(
        serde_json::json!({"status":status.as_str(),"summary":summary,"resources_by_scope":counts,
        "coverage":coverage,"failures":failures,"semantics":"point-in-time observations; not a simultaneous AWS snapshot; no absence-based deletion"}),
    )
}

#[tracing::instrument(skip_all)]
async fn detail_queries(store: &Store, scan_id: &str) -> Result<(Value, Value, Value)> {
    let escaped = scan_id.replace('\'', "''");
    let counts = store.query(&format!("SELECT account_id, region, kind, COUNT(*) AS resources FROM scan_resources WHERE scan_id='{escaped}' GROUP BY account_id,region,kind ORDER BY account_id,region,kind"), 10000).await?;
    let coverage = store.query(&format!("SELECT account_id,region,collector,parent_id,source,coverage_status FROM scan_coverage WHERE scan_id='{escaped}' ORDER BY account_id,region,collector,parent_id"), 10000).await?;
    let failures = store.query(&format!("SELECT scope_key,category,message FROM collection_failures WHERE scan_id='{escaped}'"), 10000).await?;
    Ok((counts, coverage, failures))
}
