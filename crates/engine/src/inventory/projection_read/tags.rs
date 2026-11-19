use super::{ResourceKey, Row, Sqlite, StoreError, Transaction, validate_limit};
use inventory_core::inventory::{MAX_TAG_KEY_CHARS, MAX_TAG_VALUE_CHARS};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionTag {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionTagCursor(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionTagPage {
    pub tags: Vec<ProjectionTag>,
    pub next: Option<ProjectionTagCursor>,
}

pub(super) async fn read(
    tx: &mut Transaction<'_, Sqlite>,
    key: &ResourceKey,
    after: Option<ProjectionTagCursor>,
    limit: u16,
) -> Result<ProjectionTagPage, StoreError> {
    validate_limit(limit)?;
    let key_bound = i64::try_from(MAX_TAG_KEY_CHARS)
        .map_err(|_| StoreError::Invalid("projection tag key bound overflow"))?;
    let value_bound = i64::try_from(MAX_TAG_VALUE_CHARS)
        .map_err(|_| StoreError::Invalid("projection tag value bound overflow"))?;
    // Bound character length inside SQLite, before allocating legacy data in the reader.
    // SQLite length() on TEXT returns the character count, not byte count.
    // Exclude rows containing NUL bytes which length() would truncate.
    let rows = sqlx::query("SELECT tag_key,tag_value FROM tags WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 AND (?5 IS NULL OR tag_key>?5) AND length(tag_key) BETWEEN 1 AND ?7 AND length(tag_value)<=?8 AND instr(tag_key,char(0))=0 AND instr(tag_value,char(0))=0 ORDER BY tags.tag_key LIMIT ?6")
        .bind(key.account.as_str())
        .bind(key.region.as_str())
        .bind(key.kind.as_str())
        .bind(key.id.as_str())
        .bind(after.as_ref().map(|cursor| cursor.0.as_str()))
        .bind(i64::from(limit).saturating_add(1))
        .bind(key_bound)
        .bind(value_bound)
        .fetch_all(&mut **tx)
        .await?;
    let more = rows.len() > usize::from(limit);
    let tags = rows
        .into_iter()
        .take(usize::from(limit))
        .map(tag_row)
        .collect::<Result<Vec<_>, StoreError>>()?;
    let next = if more {
        tags.last().map(|tag| ProjectionTagCursor(tag.key.clone()))
    } else {
        None
    };
    Ok(ProjectionTagPage { tags, next })
}

fn tag_row(row: sqlx::sqlite::SqliteRow) -> Result<ProjectionTag, StoreError> {
    let key = row.try_get::<String, _>("tag_key")?;
    let value = row
        .try_get::<Option<String>, _>("tag_value")?
        .unwrap_or_default();
    if key.chars().any(char::is_control) || value.contains('\0') {
        return Err(StoreError::Invalid(
            "persisted tag contains forbidden controls",
        ));
    }
    Ok(ProjectionTag { key, value })
}
