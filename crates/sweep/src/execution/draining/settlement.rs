use super::*;
use inventory_engine::inventory::{StoreError, WorkLease};

pub(in crate::execution) async fn settle_batch(
    store: &impl InventoryStore,
    executor: &ApiExecutor,
    owned: Vec<WorkLease>,
    work: impl Future<Output = Result<SweepSummary, SweepError>>,
) -> Result<SweepSummary, SweepError> {
    let result = drain(executor, work).await;
    if !executor.is_cancelled() {
        return result;
    }
    settle_owned(store, owned.into_iter()).await?;
    match result {
        Ok(_) | Err(SweepError::Api(inventory_engine::limiter::ApiError::Cancelled)) => {
            Err(inventory_engine::limiter::ApiError::Cancelled.into())
        }
        Err(error) => Err(error),
    }
}

async fn drain(
    executor: &ApiExecutor,
    work: impl Future<Output = Result<SweepSummary, SweepError>>,
) -> Result<SweepSummary, SweepError> {
    tokio::pin!(work);
    tokio::select! {
        result = &mut work => result,
        () = executor.cancelled() => {
            tokio::time::timeout(std::time::Duration::from_millis(LEASE_MARGIN_MS), &mut work)
                .await.map_err(|_| inventory_engine::limiter::ApiError::Exhausted(
                    "cancelled page drain deadline exceeded; owned leases released".to_owned()))?
        }
    }
}

pub(in crate::execution) async fn settle_owned(
    store: &impl InventoryStore,
    owned: impl ExactSizeIterator<Item = WorkLease>,
) -> Result<(), SweepError> {
    let admitted = owned.len();
    let result = stream::iter(owned)
        .then(|lease| async move {
            match store.release_lease(lease).await {
                Ok(()) | Err(StoreError::StaleLease) => Ok(()),
                Err(error) => Err(SweepError::Store(error)),
            }
        })
        .fold(Ok(()), |prior, next| async move { prior.and(next) })
        .await;
    tracing::info!(
        admitted,
        settled = result.is_ok(),
        "cancelled batch durable release completed"
    );
    result
}
