use super::*;

pub(super) fn validate_graph_key(
    scan_id: &str,
    target: &str,
    snapshot_id: &str,
) -> Result<(), StoreError> {
    validate_identifier(scan_id, "scan id")?;
    validate_identifier(target, "graph target")?;
    validate_identifier(snapshot_id, "graph snapshot id")?;
    Ok(())
}

pub(super) fn graph_checkpoint_from_row(
    row: &SqliteRow,
) -> Result<GraphExportCheckpoint, StoreError> {
    Ok(GraphExportCheckpoint {
        scan_id: row.try_get("scan_id")?,
        target: row.try_get("target")?,
        snapshot_id: row.try_get("snapshot_id")?,
        resources_exported: row_u64(row, "resources_exported")?,
        edges_exported: row_u64(row, "edges_exported")?,
        complete: row.try_get("complete")?,
    })
}

pub(super) async fn persist_graph_checkpoint(
    connection: &mut SqliteConnection,
    checkpoint: &GraphExportCheckpoint,
) -> Result<(), StoreError> {
    let resources = i64::try_from(checkpoint.resources_exported)
        .map_err(|_| StoreError::Limit("graph resource count"))?;
    let edges = i64::try_from(checkpoint.edges_exported)
        .map_err(|_| StoreError::Limit("graph edge count"))?;
    let changed = sqlx::query("INSERT INTO graph_export_checkpoints
        (scan_id,target,snapshot_id,resources_exported,edges_exported,complete) VALUES (?1,?2,?3,?4,?5,?6)
        ON CONFLICT(scan_id,target,snapshot_id) DO UPDATE SET resources_exported=excluded.resources_exported,
          edges_exported=excluded.edges_exported,complete=excluded.complete
        WHERE excluded.resources_exported >= resources_exported AND excluded.edges_exported >= edges_exported
          AND excluded.complete >= complete")
        .bind(&checkpoint.scan_id).bind(&checkpoint.target).bind(&checkpoint.snapshot_id)
        .bind(resources).bind(edges).bind(checkpoint.complete).execute(connection).await?.rows_affected();
    if changed != 1 {
        return Err(StoreError::Conflict(
            "graph checkpoint would move backwards",
        ));
    }
    Ok(())
}

pub(super) fn row_u64(row: &SqliteRow, column: &str) -> Result<u64, StoreError> {
    u64::try_from(row.try_get::<i64, _>(column)?)
        .map_err(|_| StoreError::Conflict("negative persisted count"))
}
