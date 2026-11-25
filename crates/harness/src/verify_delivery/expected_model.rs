//! Expected delivery model for one finalized scan, read from its SQLite snapshot.

use serde_json::{Value, json};
use sqlx::Row;

use super::expected_rows;
use crate::HarnessError;

pub async fn build_expected(pool: &sqlx::SqlitePool, scan_id: &str) -> Result<Value, HarnessError> {
    let (resources, edges) = fetch_resource_and_edge_rows(pool, scan_id).await?;

    let scan = sqlx::query("SELECT status, started_at, finished_at FROM scans WHERE scan_id=?")
        .bind(scan_id)
        .fetch_one(pool)
        .await
        .map_err(|e| HarnessError::invalid(format!("scan query failed: {e}")))?;

    let status = scan.try_get::<String, _>(0)?;
    if !["complete", "partial", "cancelled"].contains(&status.as_str()) {
        return Err(HarnessError::invalid("expected a finalized snapshot"));
    }

    let (completed, failed) = count_scope_checkpoints(pool, scan_id).await?;

    let resource_ids: Vec<String> = resources
        .iter()
        .map(|row| row.try_get::<String, _>(0).map(|s| s.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    let graph_ids = collect_graph_ids(&resource_ids, &edges)?;
    let resource_json_by_id = index_resource_json(scan_id, &resources)?;
    let node_rows = expected_node_rows(scan_id, &graph_ids, &resource_json_by_id)?;
    let edge_rows = expected_edge_rows(scan_id, &edges)?;

    Ok(json!({
        "scan": {
            "scan_id": scan_id,
            "status": status,
            "started_at": scan.try_get::<i64, _>(1).ok(),
            "finished_at": scan.try_get::<i64, _>(2).ok(),
        },
        "counts": {
            "resources": resource_ids.len(),
            "edges": edges.len(),
            "completed": completed,
            "failed": failed,
        },
        "observations": {
            "failure": failed,
        },
        "node_rows": node_rows,
        "edge_rows": edge_rows,
        "tag_rows": [],
    }))
}

async fn fetch_resource_and_edge_rows(
    pool: &sqlx::SqlitePool,
    scan_id: &str,
) -> Result<(Vec<sqlx::sqlite::SqliteRow>, Vec<sqlx::sqlite::SqliteRow>), HarnessError> {
    let resources =
        sqlx::query("SELECT id, resource_json FROM scan_resources WHERE scan_id=? ORDER BY id")
            .bind(scan_id)
            .fetch_all(pool)
            .await
            .map_err(|e| HarnessError::invalid(format!("query failed: {e}")))?;

    let edges = sqlx::query(
        "SELECT from_id, to_id, relation, source, edge_json FROM scan_edges WHERE scan_id=? ORDER BY from_id,to_id,relation,source",
    )
    .bind(scan_id)
    .fetch_all(pool)
    .await
    .map_err(|e| HarnessError::invalid(format!("query failed: {e}")))?;

    Ok((resources, edges))
}

async fn count_scope_checkpoints(
    pool: &sqlx::SqlitePool,
    scan_id: &str,
) -> Result<(i64, i64), HarnessError> {
    let completed: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM scope_checkpoints WHERE scan_id=? AND complete=1 AND failed=0",
    )
    .bind(scan_id)
    .fetch_one(pool)
    .await
    .map_err(|e| HarnessError::invalid(format!("scope query failed: {e}")))?;

    let failed: i64 =
        sqlx::query_scalar("SELECT count(*) FROM scope_checkpoints WHERE scan_id=? AND failed=1")
            .bind(scan_id)
            .fetch_one(pool)
            .await
            .map_err(|e| HarnessError::invalid(format!("scope query failed: {e}")))?;

    Ok((completed, failed))
}

fn collect_graph_ids(
    resource_ids: &[String],
    edges: &[sqlx::sqlite::SqliteRow],
) -> Result<Vec<String>, HarnessError> {
    let mut graph_ids = Vec::new();
    for id in resource_ids {
        graph_ids.push(id.clone());
    }
    for row in edges {
        graph_ids.push(row.try_get::<String, _>(0)?.to_string());
        graph_ids.push(row.try_get::<String, _>(1)?.to_string());
    }
    graph_ids.sort();
    graph_ids.dedup();
    Ok(graph_ids)
}

fn index_resource_json(
    scan_id: &str,
    resources: &[sqlx::sqlite::SqliteRow],
) -> Result<std::collections::BTreeMap<String, String>, HarnessError> {
    let mut resource_json_by_id = std::collections::BTreeMap::new();
    for row in resources {
        let id = row.try_get::<String, _>(0)?;
        let json = row.try_get::<String, _>(1)?;
        if resource_json_by_id.insert(id.clone(), json).is_some() {
            return Err(HarnessError::invalid(format!(
                "scan {scan_id} has duplicate resource id {id}"
            )));
        }
    }
    Ok(resource_json_by_id)
}

fn expected_node_rows(
    scan_id: &str,
    graph_ids: &[String],
    resource_json_by_id: &std::collections::BTreeMap<String, String>,
) -> Result<Vec<Value>, HarnessError> {
    graph_ids
        .iter()
        .map(|id| match resource_json_by_id.get(id) {
            Some(json) => expected_rows::observed_node(scan_id, id, json),
            None => Ok(expected_rows::unresolved_node(scan_id, id)),
        })
        .collect::<Result<Vec<_>, HarnessError>>()
}

fn expected_edge_rows(
    scan_id: &str,
    edges: &[sqlx::sqlite::SqliteRow],
) -> Result<Vec<Value>, HarnessError> {
    edges
        .iter()
        .map(|row| {
            let relation = row.try_get::<String, _>(2)?;
            let source = row.try_get::<String, _>(3)?;
            let edge_json = row.try_get::<String, _>(4)?;
            expected_rows::edge_row(scan_id, &relation, &source, &edge_json)
        })
        .collect::<Result<Vec<_>, HarnessError>>()
}
