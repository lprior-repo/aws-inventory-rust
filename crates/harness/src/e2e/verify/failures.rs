use super::metric_value;
use crate::HarnessError;
use sqlx::{Connection, postgres::PgConnection};
use std::collections::BTreeSet;

pub async fn failed_operations(database: &str, summary: &str) -> Result<(), HarnessError> {
    if metric_value(summary, "failed")? == 0 {
        return Ok(());
    }
    let mut connection = PgConnection::connect(database).await?;
    let result = load_names(&mut connection).await;
    let closed = connection.close().await.map_err(HarnessError::from);
    let names = result?;
    closed?;
    if names.is_empty() {
        return Err(HarnessError::invalid(
            "failed scope count has no persisted operation names",
        ));
    }
    println!(
        "collector failed_operations={}",
        names.into_iter().collect::<Vec<_>>().join(",")
    );
    Ok(())
}

async fn load_names(connection: &mut PgConnection) -> Result<BTreeSet<String>, HarnessError> {
    let scopes: Vec<Vec<u8>> = sqlx::query_scalar("SELECT scope_key FROM inventory.scope_outcomes WHERE outcome='failed' AND sweep_id=(SELECT sweep_id FROM inventory.sweeps ORDER BY started_at DESC,sweep_id DESC LIMIT 1) ORDER BY scope_key LIMIT 10001")
        .fetch_all(connection).await?;
    if scopes.len() > 10_000 {
        return Err(HarnessError::invalid(
            "failed operation report exceeds 10000 scopes",
        ));
    }
    scopes
        .iter()
        .map(|bytes| {
            let scope = inventory_engine::inventory::decode_scope_key(bytes).map_err(|error| {
                HarnessError::invalid(format!("persisted failed scope: {error}"))
            })?;
            Ok(format!(
                "{}:{}",
                scope.service().as_str(),
                scope.operation().as_str()
            ))
        })
        .collect()
}
