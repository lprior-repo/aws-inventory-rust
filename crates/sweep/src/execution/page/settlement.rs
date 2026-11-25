use super::*;

pub(super) async fn finish_error(
    store: &impl InventoryStore,
    lease: WorkLease,
    error: SweepError,
    output_shape: Option<&str>,
) -> Result<SweepSummary, SweepError> {
    let outcome = match &error {
        SweepError::Absent(reason) => {
            tracing::debug!(reason, "inventory scope is empty");
            return finalize(store, lease, StoredOutcome::Complete, None).await;
        }
        SweepError::Api(inventory_engine::limiter::ApiError::Cancelled) => {
            release(store, lease).await?;
            return Err(error);
        }
        SweepError::Api(inventory_engine::limiter::ApiError::Denied(_)) => {
            StoredOutcome::Inaccessible
        }
        SweepError::Api(inventory_engine::limiter::ApiError::UnsupportedType(_)) => {
            StoredOutcome::Unsupported
        }
        _ => StoredOutcome::Failed,
    };
    let detail = crate::error_detail::detail(&error, output_shape);
    if matches!(outcome, StoredOutcome::Failed) {
        warn_failed(&lease, &error, &detail);
    }
    finalize(store, lease, outcome, Some(&detail)).await
}

fn warn_failed(lease: &WorkLease, error: &SweepError, detail: &str) {
    tracing::warn!(
        service = lease.scope().service().as_str(),
        operation = lease.scope().operation().as_str(),
        region = lease.scope().region().as_str(),
        detail,
        %error,
        "inventory scope failed"
    );
}

async fn release(store: &impl InventoryStore, lease: WorkLease) -> Result<(), SweepError> {
    match store.release_lease(lease).await {
        Ok(()) | Err(StoreError::StaleLease) => Ok(()),
        Err(error) => Err(SweepError::Store(error)),
    }
}

pub(super) async fn finalize(
    store: &impl InventoryStore,
    lease: WorkLease,
    outcome: StoredOutcome,
    detail: Option<&str>,
) -> Result<SweepSummary, SweepError> {
    match store.finish_scope(lease, outcome, now_ms()?, detail).await {
        Ok(stats) => Ok(summary(outcome, 0, stats.resources_deleted)),
        Err(StoreError::StaleLease) => Ok(SweepSummary::default()),
        Err(error) => Err(SweepError::Store(error)),
    }
}

pub(super) fn summary(outcome: StoredOutcome, rows: u64, writes: u64) -> SweepSummary {
    let mut outcomes = ScopeOutcomes::default();
    outcomes.record(outcome);
    SweepSummary {
        resource_writes: writes,
        rows,
        scopes: 1,
        outcomes,
        ..SweepSummary::default()
    }
}

#[cfg(test)]
mod tests;
