use super::*;

impl Store {
    pub async fn graph_checkpoint(
        &self,
        scan_id: &str,
        target: &str,
        snapshot_id: &str,
    ) -> Result<Option<GraphExportCheckpoint>, StoreError> {
        validate_graph_key(scan_id, target, snapshot_id)?;
        let mut connection = self.reader.acquire().await?;
        reset_progress(&mut connection).await?;
        let row = sqlx::query(
            "SELECT scan_id,target,snapshot_id,resources_exported,edges_exported,complete
            FROM graph_export_checkpoints WHERE scan_id=?1 AND target=?2 AND snapshot_id=?3",
        )
        .bind(scan_id)
        .bind(target)
        .bind(snapshot_id)
        .fetch_optional(&mut *connection)
        .await?;
        row.as_ref().map(graph_checkpoint_from_row).transpose()
    }

    pub async fn save_graph_checkpoint(
        &self,
        checkpoint: &GraphExportCheckpoint,
    ) -> Result<(), StoreError> {
        validate_graph_key(
            &checkpoint.scan_id,
            &checkpoint.target,
            &checkpoint.snapshot_id,
        )?;
        let mut tx = self.writer.begin_with("BEGIN IMMEDIATE").await?;
        if matches!(
            scan_state(&mut tx, &checkpoint.scan_id).await?,
            PersistedScan::Running { .. }
        ) {
            return Err(StoreError::InvalidInput(
                "graph export requires a finalized scan",
            ));
        }
        persist_graph_checkpoint(&mut tx, checkpoint).await?;
        tx.commit().await?;
        Ok(())
    }
}
