use super::*;

pub(super) struct BatchWriter {
    pub(super) writer: BufWriter<File>,
    pub(super) graph: String,
    pub(super) scan_id: String,
    pub(super) sequence: u64,
    pub(super) rows: Vec<String>,
    pub(super) bytes: usize,
}

impl BatchWriter {
    #[tracing::instrument(skip_all, fields(stage))]
    pub(super) async fn emit(
        &mut self,
        stage: &str,
        query: String,
        rows: u64,
    ) -> Result<(), GraphError> {
        if query.len() > MAX_QUERY_BYTES || self.sequence >= MAX_BATCHES {
            return Err(GraphError::Invalid("graph export bound exceeded".into()));
        }
        let batch = GraphBatch {
            version: 1,
            graph: self.graph.clone(),
            scan_id: self.scan_id.clone(),
            sequence: self.sequence,
            stage: stage.into(),
            rows,
            query,
        };
        let mut bytes = serde_json::to_vec(&batch)?;
        bytes.push(b'\n');
        self.writer.write_all(&bytes).await?;
        self.sequence = self.sequence.saturating_add(1);
        Ok(())
    }

    #[tracing::instrument(skip_all, fields(stage))]
    pub(super) async fn push(&mut self, row: String, stage: &str) -> Result<(), GraphError> {
        let next_bytes = self
            .bytes
            .checked_add(row.len())
            .ok_or_else(|| GraphError::Invalid("batch size overflow".into()))?;
        if !self.rows.is_empty()
            && (self.rows.len() >= MAX_BATCH_ROWS
                || next_bytes > MAX_QUERY_BYTES.saturating_sub(4096))
        {
            self.flush_rows(stage).await?;
        }
        if row.len() > MAX_QUERY_BYTES.saturating_sub(4096) {
            return Err(GraphError::Invalid(
                "single graph row exceeds byte limit".into(),
            ));
        }
        self.bytes = self.bytes.saturating_add(row.len());
        self.rows.push(row);
        Ok(())
    }

    #[tracing::instrument(skip_all, fields(stage))]
    pub(super) async fn flush_rows(&mut self, stage: &str) -> Result<(), GraphError> {
        if self.rows.is_empty() {
            return Ok(());
        }
        let template = if stage == "resources" {
            RESOURCE_QUERY
        } else {
            EDGE_QUERY
        };
        let query = format!("CYPHER rows=[{}] {template}", self.rows.join(","));
        let count = u64::try_from(self.rows.len())
            .map_err(|_| GraphError::Invalid("row count overflow".into()))?;
        self.emit(stage, query, count).await?;
        self.rows.clear();
        self.bytes = 0;
        Ok(())
    }
}
