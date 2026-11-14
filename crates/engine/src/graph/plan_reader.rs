use super::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum PlanStage {
    Index,
    Begin,
    Resources,
    Edges,
    End,
}

impl PlanStage {
    fn parse(value: &str) -> Result<Self, GraphError> {
        match value {
            "index" => Ok(Self::Index),
            "begin" => Ok(Self::Begin),
            "resources" => Ok(Self::Resources),
            "edges" => Ok(Self::Edges),
            "end" => Ok(Self::End),
            _ => Err(GraphError::Invalid("invalid projection stage".into())),
        }
    }

    fn is_metadata(self) -> bool {
        matches!(self, Self::Index | Self::Begin | Self::End)
    }
}

pub(super) struct PlanReader {
    reader: BufReader<File>,
    identity: Option<(String, String)>,
    sequence: u64,
    stage: PlanStage,
}

impl PlanReader {
    pub(super) fn new(file: File) -> Self {
        Self {
            reader: BufReader::new(file),
            identity: None,
            sequence: 0,
            stage: PlanStage::Index,
        }
    }

    pub(super) fn accept(&mut self, batch: &GraphBatch) -> Result<(), GraphError> {
        validate_name(&batch.graph)?;
        validate_name(&batch.scan_id)?;
        let pair = (batch.graph.clone(), batch.scan_id.clone());
        self.validate_identity(batch, &pair)?;
        self.validate_bounds(batch)?;
        let next = self.next_stage(batch)?;
        self.identity = Some(pair);
        self.stage = next;
        self.sequence = self.sequence.saturating_add(1);
        Ok(())
    }

    fn validate_identity(
        &self,
        batch: &GraphBatch,
        pair: &(String, String),
    ) -> Result<(), GraphError> {
        if self.identity.as_ref().is_some_and(|old| old != pair)
            || self.sequence != batch.sequence
            || self.stage == PlanStage::End
        {
            return Err(GraphError::Invalid(
                "mixed identities, noncontiguous sequence, or data after end marker".into(),
            ));
        }
        Ok(())
    }

    fn validate_bounds(&self, batch: &GraphBatch) -> Result<(), GraphError> {
        if batch.version != 1
            || batch.query.len() > MAX_QUERY_BYTES
            || self.sequence >= MAX_BATCHES
            || batch.rows > 256
        {
            return Err(GraphError::Invalid(
                "unsupported or oversized projection batch".into(),
            ));
        }
        Ok(())
    }

    fn next_stage(&self, batch: &GraphBatch) -> Result<PlanStage, GraphError> {
        let next = PlanStage::parse(&batch.stage)?;
        let allowed = match self.sequence {
            0 => next == PlanStage::Index,
            1 => next == PlanStage::Begin,
            _ => next >= PlanStage::Resources && next >= self.stage,
        };
        if !allowed {
            return Err(GraphError::Invalid("projection stages out of order".into()));
        }
        if next.is_metadata() && batch.rows != 0 {
            return Err(GraphError::Invalid(
                "metadata batch contains row count".into(),
            ));
        }
        Ok(next)
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
        if plan.stage != PlanStage::End {
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

#[cfg(test)]
mod tests;
