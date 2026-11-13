//! Strict bounded graph plan validation and replay.
use super::*;

pub(super) struct PlanReader {
    pub(super) reader: BufReader<File>,
    pub(super) identity: Option<(String, String)>,
    pub(super) sequence: u64,
    pub(super) stage: u8,
    pub(super) ended: bool,
}

impl PlanReader {
    pub(super) fn new(file: File) -> Self {
        Self {
            reader: BufReader::new(file),
            identity: None,
            sequence: 0,
            stage: 0,
            ended: false,
        }
    }

    pub(super) fn accept(&mut self, batch: &GraphBatch) -> Result<(), GraphError> {
        validate_name(&batch.graph)?;
        validate_name(&batch.scan_id)?;
        let pair = (batch.graph.clone(), batch.scan_id.clone());
        if self.identity.as_ref().is_some_and(|old| old != &pair)
            || self.sequence != batch.sequence
            || self.ended
        {
            return Err(GraphError::Invalid(
                "mixed identities, noncontiguous sequence, or data after end marker".into(),
            ));
        }
        if batch.version != 1
            || batch.query.len() > MAX_QUERY_BYTES
            || self.sequence >= MAX_BATCHES
            || batch.rows > 256
        {
            return Err(GraphError::Invalid(
                "unsupported or oversized projection batch".into(),
            ));
        }
        let next = match batch.stage.as_str() {
            "index" => 0,
            "begin" => 1,
            "resources" => 2,
            "edges" => 3,
            "end" => 4,
            _ => return Err(GraphError::Invalid("invalid projection stage".into())),
        };
        let allowed = match self.sequence {
            0 => next == 0,
            1 => next == 1,
            _ => next >= 2 && next >= self.stage,
        };
        if !allowed {
            return Err(GraphError::Invalid("projection stages out of order".into()));
        }
        if (next < 2 || next == 4) && batch.rows != 0 {
            return Err(GraphError::Invalid(
                "metadata batch contains row count".into(),
            ));
        }
        self.identity = Some(pair);
        self.stage = next;
        self.ended = next == 4;
        self.sequence = self.sequence.saturating_add(1);
        Ok(())
    }
}

#[tracing::instrument(skip_all)]
pub(super) async fn next_batch(
    mut plan: PlanReader,
) -> Result<Option<(GraphBatch, PlanReader)>, GraphError> {
    let bound = u64::try_from(MAX_QUERY_BYTES.saturating_mul(2).saturating_add(8192))
        .map_err(|_| GraphError::Invalid("line bound".into()))?;
    let mut bytes = Vec::new();
    (&mut plan.reader)
        .take(bound)
        .read_until(b'\n', &mut bytes)
        .await?;
    if bytes.is_empty() {
        if !plan.ended {
            return Err(GraphError::Invalid(
                "projection is empty or lacks a final end marker".into(),
            ));
        }
        return Ok(None);
    }
    if u64::try_from(bytes.len()).map_err(|_| GraphError::Invalid("line size".into()))? >= bound {
        return Err(GraphError::Invalid("projection line exceeds limit".into()));
    }
    let batch: GraphBatch = serde_json::from_slice(&bytes)?;
    plan.accept(&batch)?;
    Ok(Some((batch, plan)))
}
