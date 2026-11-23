use super::*;
use inventory_engine::inventory::WorkLease;

pub(in crate::execution) async fn claim_batch<F>(
    store: &BinaryStore,
    executor: &ApiExecutor,
    concurrency: usize,
    leases: &mut Vec<WorkLease>,
    progress: &crate::control::Progress,
    mut claim_next: impl FnMut() -> F,
) -> Result<(), SweepError>
where
    F: Future<Output = Result<Option<WorkLease>, SweepError>>,
{
    leases.try_reserve(concurrency)?;
    while !executor.is_cancelled() && leases.len() < concurrency {
        if !admit(store, leases, progress, claim_next().await).await? {
            break;
        }
    }
    Ok(())
}

async fn admit(
    store: &BinaryStore,
    leases: &mut Vec<WorkLease>,
    progress: &crate::control::Progress,
    result: Result<Option<WorkLease>, SweepError>,
) -> Result<bool, SweepError> {
    match result {
        Ok(Some(lease)) => {
            let pending = lease.pending_work();
            leases.push(lease);
            progress.queue(pending, leases.len());
            Ok(true)
        }
        Ok(None) => Ok(false),
        Err(error) => {
            super::settle_owned(store, leases.drain(..)).await?;
            Err(error)
        }
    }
}

pub(super) async fn claim_next(
    context: &RuntimeContext<'_, '_, impl crate::CredentialSource>,
    sweep: &SweepId,
) -> Result<Option<inventory_engine::inventory::WorkLease>, SweepError> {
    let now = now_ms()?;
    let lease = if context.progress.enabled() {
        context
            .store
            .claim_with_depth(sweep, now, context.lease_ms)
            .await?
    } else {
        context.store.claim(sweep, now, context.lease_ms).await?
    };
    Ok(lease)
}
