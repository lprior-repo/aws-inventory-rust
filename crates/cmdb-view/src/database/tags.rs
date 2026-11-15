use super::{ResourceTag, StoredTagObservation, ViewError, MAX_PAGE_LIMIT};
use inventory_core::inventory::TagCoverage;
use sqlx::Row;

pub(super) struct StoredTags {
    pub tags: Vec<ResourceTag>,
    pub truncated: bool,
    pub observation: Option<StoredTagObservation>,
}

// Uses the caller's retained transaction: tags and record cannot mix revisions.
pub(super) async fn read(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    [account, region, kind, id]: [&str; 4],
) -> Result<StoredTags, ViewError> {
    let limit = i64::try_from(MAX_PAGE_LIMIT.checked_add(1).ok_or(ViewError::PageLimit)?)
        .map_err(|_| ViewError::PageLimit)?;
    let mut rows = sqlx::query("SELECT tag_key,tag_value FROM tags WHERE account=? AND region=? AND kind=? AND id=? ORDER BY tag_key LIMIT ?")
        .bind(account).bind(region).bind(kind).bind(id).bind(limit)
        .fetch_all(&mut **tx).await?;
    let truncated = rows.len() > MAX_PAGE_LIMIT;
    rows.truncate(MAX_PAGE_LIMIT);
    let tags = rows.into_iter().map(|row| {
        Ok(ResourceTag { key: row.try_get("tag_key")?, value: row.try_get("tag_value")? })
    }).collect::<Result<Vec<_>, ViewError>>()?;
    let observation = observation(tx, [account, region, kind, id]).await?;
    Ok(StoredTags { tags, truncated, observation })
}

async fn observation(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    [account, region, kind, id]: [&str; 4],
) -> Result<Option<StoredTagObservation>, ViewError> {
    let row = sqlx::query("SELECT coverage,observed_at,last_complete_at FROM tag_observations WHERE account=? AND region=? AND kind=? AND id=?")
        .bind(account).bind(region).bind(kind).bind(id)
        .fetch_optional(&mut **tx).await?;
    row.map(|row| {
        let coverage = match row.try_get::<&str, _>("coverage")? {
            "complete" => TagCoverage::Complete,
            "partial" => TagCoverage::Partial,
            "failed" => TagCoverage::Failed,
            "unobserved" => TagCoverage::Unobserved,
            _ => return Err(ViewError::StoredRow),
        };
        Ok(StoredTagObservation { coverage, observed_at_ms: row.try_get("observed_at")?,
            last_complete_at_ms: row.try_get("last_complete_at")? })
    }).transpose()
}
