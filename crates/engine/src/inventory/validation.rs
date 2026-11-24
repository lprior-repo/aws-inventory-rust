use super::work::SweepId;
use super::{
    BinaryStore, MAX_OUTCOME_DETAIL_BYTES, MAX_TOKEN_BYTES, PageCommit, StoreError, WorkSpec,
};
use sqlx::{Row, Sqlite, Transaction};

pub(super) async fn validate_active_sweep(
    tx: &mut Transaction<'_, Sqlite>,
    expected: Option<&SweepId>,
) -> Result<(), StoreError> {
    let rows = sqlx::query(
        "SELECT DISTINCT sweep_id FROM work_queue WHERE state IN(0,1) ORDER BY sweep_id LIMIT 2",
    )
    .fetch_all(&mut **tx)
    .await?;
    if rows.len() > 1 {
        return Err(StoreError::MultipleActiveSweeps);
    }
    let active = rows
        .into_iter()
        .next()
        .map(|row| row.try_get::<String, _>("sweep_id"))
        .transpose()?;
    if let Some(active) = active {
        let matches = match expected {
            Some(sweep) => active == sweep.as_str(),
            None => false,
        };
        if !matches {
            return Err(StoreError::ActiveSweepConflict);
        }
    }
    Ok(())
}

/// A stored outcome reason is bounded so one failing scope cannot grow the table.
pub(super) fn validate_detail(detail: Option<&str>) -> Result<(), StoreError> {
    match detail {
        Some(text) if text.len() > MAX_OUTCOME_DETAIL_BYTES => {
            Err(StoreError::Invalid("scope outcome detail exceeds bounds"))
        }
        _ => Ok(()),
    }
}

pub(super) fn validate_work(store: &BinaryStore, work: &WorkSpec) -> Result<(), StoreError> {
    work.validate_context()?;
    if !store.registry.allows(work.scope()) {
        return Err(StoreError::Invalid(
            "work scope is not present in the bound catalog",
        ));
    }
    Ok(())
}

pub(super) fn validate_page(store: &BinaryStore, page: &PageCommit) -> Result<(), StoreError> {
    let token_limit = MAX_TOKEN_BYTES;
    let scope = page.lease.scope();
    if page.observed_at_ms < 0
        || page.observations.len() > 100_000
        || page.edges.len() > 10_000
        || page.unresolved.len() > 10_000
        || page.tags.len() > 50_000
        || page.children.len() > 10_000
    {
        return Err(StoreError::Invalid("page exceeds storage bounds"));
    }
    if page.done != page.continuation_digest.is_none()
        || page.done != page.next_token.is_none()
        || page
            .next_token
            .as_ref()
            .is_some_and(|token| token.len() > token_limit)
    {
        return Err(StoreError::Invalid(
            "page continuation token and digest are inconsistent or out of bounds",
        ));
    }
    if page
        .children
        .iter()
        .any(|child| child.sweep() != page.lease.sweep() || !store.registry.allows(child.scope()))
    {
        return Err(StoreError::Invalid(
            "child work is not in this sweep or bound catalog",
        ));
    }
    if page.observations.iter().any(|item| {
        item.key().account.as_str() != scope.account().as_str()
            || item.key().region.as_str() != scope.region().as_str()
    }) || page.edges.iter().any(|edge| {
        edge.from.account.as_str() != scope.account().as_str()
            || edge.from.region.as_str() != scope.region().as_str()
    }) || page.unresolved.iter().any(|reference| {
        reference.from.account.as_str() != scope.account().as_str()
            || reference.from.region.as_str() != scope.region().as_str()
    }) || page.tags.iter().any(|tag| {
        tag.resource().account.as_str() != scope.account().as_str()
            || tag.resource().region.as_str() != scope.region().as_str()
    }) {
        return Err(StoreError::Invalid(
            "page contains data outside its leased account and region",
        ));
    }
    validate_tags(page)?;
    store.registry.permissions.validate_page(page)?;
    Ok(())
}

fn validate_tags(page: &PageCommit) -> Result<(), StoreError> {
    let unique = page
        .tags
        .iter()
        .map(|tags| tags.resource())
        .collect::<std::collections::BTreeSet<_>>();
    if unique.len() != page.tags.len() {
        return Err(StoreError::Invalid(
            "page contains duplicate resource tag observations",
        ));
    }
    let count = page.tags.iter().try_fold(0_usize, |count, tags| {
        count
            .checked_add(tags.tags().len())
            .ok_or(StoreError::Invalid("tag count overflow"))
    })?;
    if count > 50_000 {
        return Err(StoreError::Invalid("page exceeds tag pair storage bound"));
    }
    Ok(())
}
