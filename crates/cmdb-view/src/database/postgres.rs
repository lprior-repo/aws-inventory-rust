mod freshness;
#[cfg(test)]
mod probes;
mod queries;
mod records;
mod snapshot;
#[cfg(test)]
mod tests;

use super::sqlite::{
    next_edge_cursor, next_resource_cursor, next_row_cursor, resource_key, stored_hash,
    validate_page_limit,
};
use super::*;
use crate::GraphEdge;
use futures_util::{TryStreamExt, stream};
use inventory_core::inventory::{ResourceKey, SqlName, TagCoverage};
use inventory_core::record::{ContentHash, Record};
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{PgPool, Postgres, Row, Transaction};
use std::collections::BTreeMap;

pub(super) struct Database {
    pool: PgPool,
}

pub(super) struct Snapshot {
    tx: Transaction<'static, Postgres>,
    tables: BTreeMap<String, SqlName>,
}

impl Database {
    pub(super) async fn open(target: &str) -> Result<Self, ViewError> {
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .after_connect(|connection, _| {
                Box::pin(async move {
                    sqlx::query("SET search_path = inventory")
                        .execute(&mut *connection)
                        .await?;
                    sqlx::query("SET default_transaction_read_only = on")
                        .execute(connection)
                        .await?;
                    Ok(())
                })
            })
            .connect(target)
            .await
            .map_err(open_error)?;
        #[cfg(test)]
        probes::record_pool(&pool);
        let version: Option<Vec<u8>> = match sqlx::query_scalar(
            "SELECT value FROM inventory.metadata WHERE key='schema_version'",
        )
        .fetch_optional(&pool)
        .await
        {
            Ok(version) => version,
            Err(error) => {
                pool.close().await;
                return Err(open_error(error));
            }
        };
        if !matches!(
            version.as_deref(),
            Some(value) if value == b"14" || value == b"15"
        ) {
            pool.close().await;
            return Err(ViewError::PostgresVersion);
        }
        Ok(Self { pool })
    }

    pub(super) async fn snapshot(&self) -> Result<Snapshot, ViewError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .execute(&mut *tx)
            .await?;
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM inventory.resources)")
            .fetch_one(&mut *tx)
            .await?;
        Ok(Snapshot {
            tx,
            tables: BTreeMap::new(),
        })
    }
}

fn open_error(error: sqlx::Error) -> ViewError {
    match error
        .as_database_error()
        .and_then(|error| error.code())
        .as_deref()
    {
        Some("28P01" | "28000") => ViewError::PostgresAuthentication,
        Some("3D000") => ViewError::PostgresDatabase,
        Some("42P01" | "3F000") => ViewError::PostgresSchema,
        _ => ViewError::PostgresConnect,
    }
}

fn row_key(row: &PgRow) -> Result<ResourceKey, ViewError> {
    resource_key([
        row.try_get("account")?,
        row.try_get("region")?,
        row.try_get("kind")?,
        row.try_get("id")?,
    ])
}

fn row_tags(
    row: &PgRow,
) -> Result<(Vec<ResourceTag>, bool, Option<StoredTagObservation>), ViewError> {
    let pairs: Vec<(String, String)> = row.try_get("tag_pairs")?;
    let truncated = pairs.len() > MAX_PAGE_LIMIT;
    let tags = pairs
        .into_iter()
        .take(MAX_PAGE_LIMIT)
        .map(|(key, value)| ResourceTag { key, value })
        .collect();
    let observation = row
        .try_get::<Option<&str>, _>("coverage")?
        .map(|coverage| {
            let coverage = match coverage {
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
        .transpose()?;
    Ok((tags, truncated, observation))
}

fn decode_resource(
    key: ResourceKey,
    row: PgRow,
    detail: PgRow,
) -> Result<StoredResource, ViewError> {
    let expected: Vec<u8> = row.try_get("content_hash")?;
    let actual: Vec<u8> = detail.try_get("content_hash")?;
    if expected != actual {
        return Err(ViewError::StoredRow);
    }
    let (tags, tags_truncated, tag_observation) = row_tags(&row)?;
    Ok(StoredResource {
        key,
        name: row.try_get("name")?,
        content_hash: ContentHash::from_bytes(stored_hash(&expected)?),
        record: Record::from_bytes(detail.try_get::<Vec<u8>, _>("record")?)?,
        tags,
        tags_truncated,
        tag_observation,
    })
}

async fn decode_resources(
    snapshot: &mut Snapshot,
    rows: Vec<PgRow>,
) -> Result<Vec<StoredResource>, ViewError> {
    let details = records::read_details(snapshot, &rows).await?;
    rows.into_iter()
        .zip(details)
        .map(|(row, detail)| {
            let key = row_key(&row)?;
            decode_resource(key, row, detail.ok_or(ViewError::StoredRow)?)
        })
        .collect()
}

fn decode_rows(
    rows: Vec<PgRow>,
    lengths: &BTreeMap<ResourceKey, usize>,
) -> Result<(Vec<ResourceRow>, bool), ViewError> {
    let (rows, _, truncated) = rows.into_iter().try_fold(
        (Vec::new(), 0_usize, false),
        |(mut result, bytes, truncated), row| {
            if truncated {
                return Ok((result, bytes, true));
            }
            let key = row_key(&row)?;
            let record_bytes = *lengths.get(&key).ok_or(ViewError::StoredRow)?;
            let (tags, tags_truncated, tag_observation) = row_tags(&row)?;
            let row_bytes = tags.iter().fold(record_bytes, |sum, tag| {
                sum.saturating_add(tag.key.len())
                    .saturating_add(tag.value.len())
            });
            let total = bytes.saturating_add(row_bytes);
            if total > 10_485_760 {
                if result.is_empty() {
                    return Err(ViewError::PageBudget);
                }
                return Ok((result, bytes, true));
            }
            let expected: Vec<u8> = row.try_get("content_hash")?;
            result.push(ResourceRow {
                key,
                name: row.try_get("name")?,
                content_hash: ContentHash::from_bytes(stored_hash(&expected)?),
                record_bytes,
                tags,
                tags_truncated,
                tag_observation,
            });
            Ok::<_, ViewError>((result, total, false))
        },
    )?;
    Ok((rows, truncated))
}
