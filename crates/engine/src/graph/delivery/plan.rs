//! Immutable graph plan hashing and binding.
use super::*;

#[derive(Default)]
pub(super) struct FrozenPlan {
    pub(super) graph: String,
    pub(super) scan_id: String,
    pub(super) hashes: Vec<[u8; 32]>,
    pub(super) hasher: Sha256,
}

pub(super) fn batch_hash(batch: &GraphBatch) -> Result<[u8; 32], GraphError> {
    Ok(Sha256::digest(serde_json::to_vec(batch)?).into())
}

pub(super) fn hex(hash: &[u8]) -> String {
    hash.iter().map(|byte| format!("{byte:02x}")).collect()
}

impl FrozenPlan {
    pub(super) fn digest(&self) -> String {
        format!(
            "{:x}",
            base16ct::HexDisplay(&self.hasher.clone().finalize())
        )
    }

    pub(super) fn hash(&self, sequence: u64) -> Result<&[u8; 32], GraphError> {
        let index = usize::try_from(sequence)
            .map_err(|_| GraphError::Invalid("sequence overflow".into()))?;
        self.hashes.get(index).ok_or_else(|| {
            GraphError::Invalid("projection differs from the validated frozen plan".into())
        })
    }
}

pub(super) async fn freeze_plan(input: &Path) -> Result<FrozenPlan, GraphError> {
    let file = File::open(input).await?;
    stream::try_unfold(PlanReader::new(file), next_batch)
        .try_fold(FrozenPlan::default(), |mut plan, batch| async move {
            let hash = batch_hash(&batch)?;
            if plan.hashes.is_empty() {
                plan.graph = batch.graph.clone();
                plan.scan_id = batch.scan_id.clone();
            }
            plan.hashes.try_reserve(1).map_err(|_| {
                GraphError::Invalid("projection fingerprint allocation failed".into())
            })?;
            plan.hashes.push(hash);
            plan.hasher.update(hash);
            Ok(plan)
        })
        .await
}

pub(super) async fn bind_plan(log: &ProjectionLog, plan: &FrozenPlan) -> Result<(), GraphError> {
    sqlx::query("CREATE TABLE IF NOT EXISTS projection_plan (destination TEXT NOT NULL, graph TEXT NOT NULL, scan_id TEXT NOT NULL, digest TEXT NOT NULL, PRIMARY KEY(destination,graph,scan_id))")
        .execute(&log.pool).await?;
    let expected = plan.digest();
    let found: Option<String> = sqlx::query_scalar(
        "SELECT digest FROM projection_plan WHERE destination=?1 AND graph=?2 AND scan_id=?3",
    )
    .bind(&log.destination)
    .bind(&plan.graph)
    .bind(&plan.scan_id)
    .fetch_optional(&log.pool)
    .await?;
    if found.as_ref().is_some_and(|old| old != &expected) {
        return Err(GraphError::Invalid(
            "projection plan changed after checkpoint; use a new checkpoint or scan ID".into(),
        ));
    }
    // Also validate legacy checkpoints in full before any new server write.
    let limit = i64::try_from(crate::graph::MAX_BATCHES.saturating_add(1))
        .map_err(|_| GraphError::Invalid("checkpoint row bound overflow".into()))?;
    sqlx::query_as::<_, (i64, String)>("SELECT sequence,digest FROM projection_ack WHERE destination=?1 AND graph=?2 AND scan_id=?3 ORDER BY sequence LIMIT ?4")
        .bind(&log.destination).bind(&plan.graph).bind(&plan.scan_id).bind(limit).fetch(&log.pool)
        .map_err(GraphError::Sql)
        .try_for_each(|(sequence, old)| async move {
            let sequence = u64::try_from(sequence).map_err(|_| GraphError::Invalid("negative checkpoint sequence".into()))?;
            if old != hex(plan.hash(sequence)?) {
                return Err(GraphError::Invalid("acknowledged batch differs from the frozen plan".into()));
            }
            Ok(())
        }).await?;
    sqlx::query("INSERT INTO projection_plan(destination,graph,scan_id,digest) VALUES (?1,?2,?3,?4) ON CONFLICT(destination,graph,scan_id) DO NOTHING")
        .bind(&log.destination).bind(&plan.graph).bind(&plan.scan_id).bind(&expected).execute(&log.pool).await?;
    let bound: String = sqlx::query_scalar(
        "SELECT digest FROM projection_plan WHERE destination=?1 AND graph=?2 AND scan_id=?3",
    )
    .bind(&log.destination)
    .bind(&plan.graph)
    .bind(&plan.scan_id)
    .fetch_one(&log.pool)
    .await?;
    if bound != expected {
        return Err(GraphError::Invalid(
            "another writer bound a different graph plan".into(),
        ));
    }
    Ok(())
}

pub(super) struct Projection {
    pub(super) client: redis::Client,
    pub(super) connection: Option<MultiplexedConnection>,
    pub(super) log: ProjectionLog,
    pub(super) plan: FrozenPlan,
    pub(super) options: PushOptions,
    pub(super) summary: PushSummary,
    pub(super) connections: AtomicU64,
}
