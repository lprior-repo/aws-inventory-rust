use super::{
    Postgres, ProjectionTagCursor, ProjectionTagPage, ResourceKey, StoreError, Transaction,
    validate_limit,
};
use crate::inventory::ProjectionTag;
use inventory_core::inventory::{MAX_TAG_KEY_CHARS, MAX_TAG_VALUE_CHARS};
use sqlx::{Row, postgres::PgRow};

pub(super) async fn read(
    tx: &mut Transaction<'_, Postgres>,
    key: &ResourceKey,
    after: Option<ProjectionTagCursor>,
    limit: u16,
) -> Result<ProjectionTagPage, StoreError> {
    validate_limit(limit)?;
    let key_bound = i64::try_from(MAX_TAG_KEY_CHARS)
        .map_err(|_| StoreError::Invalid("projection tag key bound overflow"))?;
    let value_bound = i64::try_from(MAX_TAG_VALUE_CHARS)
        .map_err(|_| StoreError::Invalid("projection tag value bound overflow"))?;
    let rows = sqlx::query("SELECT tag_key,tag_value FROM tags WHERE account=$1 AND region=$2 AND kind=$3 AND id=$4 AND ($5::text IS NULL OR tag_key>$5) AND char_length(tag_key) BETWEEN 1 AND $7 AND char_length(tag_value)<=$8 ORDER BY tag_key LIMIT $6")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .bind(after.as_ref().map(|cursor| cursor.0.as_str())).bind(i64::from(limit).saturating_add(1))
        .bind(key_bound).bind(value_bound).fetch_all(&mut **tx).await?;
    page(rows, limit)
}

fn page(rows: Vec<PgRow>, limit: u16) -> Result<ProjectionTagPage, StoreError> {
    let more = rows.len() > usize::from(limit);
    let tags = rows
        .into_iter()
        .take(usize::from(limit))
        .map(decode)
        .collect::<Result<Vec<_>, _>>()?;
    let next = if more {
        tags.last().map(|tag| ProjectionTagCursor(tag.key.clone()))
    } else {
        None
    };
    Ok(ProjectionTagPage { tags, next })
}

fn decode(row: PgRow) -> Result<ProjectionTag, StoreError> {
    let key: String = row.try_get("tag_key")?;
    let value: String = row.try_get("tag_value")?;
    if key.chars().any(char::is_control) || value.contains('\0') {
        return Err(StoreError::Invalid(
            "persisted tag contains forbidden controls",
        ));
    }
    Ok(ProjectionTag { key, value })
}
