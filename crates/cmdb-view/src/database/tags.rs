use super::{MAX_PAGE_LIMIT, ResourceTag, StoredTagObservation, ViewError};
use inventory_core::inventory::TagCoverage;
use sqlx::Row;

/// Batched read of tags and observations for multiple resources in a single
/// database round-trip each. Returns maps keyed by the full resource key
/// tuple (account,region,kind,id).
pub(super) async fn read_many(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    resources: &[(String, String, String, String)],
) -> Result<PageTags, ViewError> {
    let (tags, tags_truncated) = batch_tags(tx, resources).await?;
    let observations = batch_observations(tx, resources).await?;
    Ok(PageTags {
        tags,
        observations,
        tags_truncated,
    })
}

pub(super) struct PageTags {
    pub tags: TagMap,
    pub observations: ObsMap,
    pub tags_truncated: bool,
}

pub(super) type ResKey = (String, String, String, String);
pub(super) type TagMap = std::collections::HashMap<ResKey, (Vec<ResourceTag>, bool)>;
pub(super) type ObsMap = std::collections::HashMap<ResKey, Option<StoredTagObservation>>;

async fn batch_tags(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    resources: &[(String, String, String, String)],
) -> Result<(TagMap, bool), ViewError> {
    if resources.is_empty() {
        return Ok((std::collections::HashMap::new(), false));
    }
    let mut builder = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "SELECT account,region,kind,id,tag_key,tag_value FROM tags WHERE (account,region,kind,id) IN (",
    );
    for (index, (account, region, kind, id)) in resources.iter().enumerate() {
        if index > 0 {
            builder.push(",");
        }
        builder.push("(");
        builder.push_bind(account);
        builder.push(",");
        builder.push_bind(region);
        builder.push(",");
        builder.push_bind(kind);
        builder.push(",");
        builder.push_bind(id);
        builder.push(")");
    }
    builder.push(")").push(" ORDER BY tag_key");
    let query = builder.build();
    let rows = query.fetch_all(&mut **tx).await?;
    let mut tags = std::collections::HashMap::new();
    for row in rows {
        let account: String = row.try_get("account")?;
        let region: String = row.try_get("region")?;
        let kind: String = row.try_get("kind")?;
        let id: String = row.try_get("id")?;
        let tag_key: String = row.try_get("tag_key")?;
        let tag_value: String = row.try_get("tag_value")?;
        tags.entry((account, region, kind, id))
            .or_insert_with(|| (Vec::new(), false))
            .0
            .push(ResourceTag {
                key: tag_key,
                value: tag_value,
            });
    }
    let mut any_truncated = false;
    tags.values_mut().for_each(|(tag_list, truncated)| {
        let was_truncated = tag_list.len() > MAX_PAGE_LIMIT;
        if was_truncated {
            any_truncated = true;
        }
        tag_list.truncate(MAX_PAGE_LIMIT);
        *truncated = was_truncated;
    });
    Ok((tags, any_truncated))
}

async fn batch_observations(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    resources: &[(String, String, String, String)],
) -> Result<ObsMap, ViewError> {
    if resources.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let mut builder = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "SELECT account,region,kind,id,coverage,observed_at,last_complete_at FROM tag_observations WHERE (account,region,kind,id) IN (",
    );
    for (index, (account, region, kind, id)) in resources.iter().enumerate() {
        if index > 0 {
            builder.push(",");
        }
        builder.push("(");
        builder.push_bind(account);
        builder.push(",");
        builder.push_bind(region);
        builder.push(",");
        builder.push_bind(kind);
        builder.push(",");
        builder.push_bind(id);
        builder.push(")");
    }
    builder.push(")");
    let query = builder.build();
    let rows = query.fetch_all(&mut **tx).await?;
    let mut observations = std::collections::HashMap::new();
    for row in rows {
        let account: String = row.try_get("account")?;
        let region: String = row.try_get("region")?;
        let kind: String = row.try_get("kind")?;
        let id: String = row.try_get("id")?;
        let coverage = match row.try_get::<&str, _>("coverage")? {
            "complete" => TagCoverage::Complete,
            "partial" => TagCoverage::Partial,
            "failed" => TagCoverage::Failed,
            "unobserved" => TagCoverage::Unobserved,
            _ => return Err(ViewError::StoredRow),
        };
        observations.insert(
            (account, region, kind, id),
            Some(StoredTagObservation {
                coverage,
                observed_at_ms: row.try_get("observed_at")?,
                last_complete_at_ms: row.try_get("last_complete_at")?,
            }),
        );
    }
    Ok(observations)
}

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
    let tags = rows
        .into_iter()
        .map(|row| {
            Ok(ResourceTag {
                key: row.try_get("tag_key")?,
                value: row.try_get("tag_value")?,
            })
        })
        .collect::<Result<Vec<_>, ViewError>>()?;
    let observation = observation(tx, [account, region, kind, id]).await?;
    Ok(StoredTags {
        tags,
        truncated,
        observation,
    })
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
        Ok(StoredTagObservation {
            coverage,
            observed_at_ms: row.try_get("observed_at")?,
            last_complete_at_ms: row.try_get("last_complete_at")?,
        })
    })
    .transpose()
}
