use super::*;
use inventory_engine::inventory::WorkLease;

pub(in crate::sweep_runtime::execution) async fn claim_batch<F>(
    store: &BinaryStore,
    executor: &ApiExecutor,
    concurrency: usize,
    leases: &mut Vec<WorkLease>,
    mut claim_next: impl FnMut() -> F,
) -> Result<(), SweepError>
where
    F: Future<Output = Result<Option<WorkLease>, SweepError>>,
{
    // Capacity failure must precede every durable claim, not strand a lease
    // after its transaction has already committed.
    leases.try_reserve(concurrency)?;
    while !executor.is_cancelled() && leases.len() < concurrency {
        match claim_next().await {
            Ok(Some(lease)) => leases.push(lease),
            Ok(None) => break,
            Err(error) => {
                // This also drains a claim's queued rollback before returning
                // its original error, whether or not cancellation has arrived.
                super::settle_owned(store, leases.drain(..)).await?;
                return Err(error);
            }
        }
    }
    Ok(())
}
