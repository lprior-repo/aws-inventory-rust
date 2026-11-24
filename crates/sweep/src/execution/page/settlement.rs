use super::*;

pub(super) async fn finish_error(
    store: &impl InventoryStore,
    lease: WorkLease,
    error: SweepError,
) -> Result<SweepSummary, SweepError> {
    let outcome = match &error {
        SweepError::Absent(reason) => {
            tracing::debug!(reason, "inventory scope is empty");
            StoredOutcome::Complete
        }
        SweepError::Api(inventory_engine::limiter::ApiError::Denied(_)) => {
            StoredOutcome::Inaccessible
        }
        SweepError::Api(inventory_engine::limiter::ApiError::UnsupportedType(_)) => {
            StoredOutcome::Unsupported
        }
        SweepError::Api(inventory_engine::limiter::ApiError::Cancelled) => {
            release(store, lease).await?;
            return Err(error);
        }
        _ => failed(&lease, &error),
    };
    finalize(store, lease, outcome).await
}

fn failed(lease: &WorkLease, error: &SweepError) -> StoredOutcome {
    let detail = crate::error_detail::detail(error, None);
    tracing::warn!(
        service = lease.scope().service().as_str(),
        operation = lease.scope().operation().as_str(),
        region = lease.scope().region().as_str(),
        detail = %detail,
        %error,
        "inventory scope failed"
    );
    StoredOutcome::Failed
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
) -> Result<SweepSummary, SweepError> {
    match store.finish_scope(lease, outcome, now_ms()?).await {
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
