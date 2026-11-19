#[path = "database/queries.rs"]
mod queries;
#[path = "database/snapshot.rs"]
mod snapshot;
#[path = "database/tags.rs"]
mod tags;
use crate::ViewError;
use crate::graph::{GraphEdge, GraphNode};
use inventory_core::inventory::{AccountId, Kind, Region, ResourceId, ResourceKey, SqlName};
use inventory_core::record::{ContentHash, Record};
use queries::*;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::path::Path;

pub const MAX_PAGE_LIMIT: usize = 1000;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResourceFilter {
    pub account: Option<String>,
    pub region: Option<String>,
    pub kind: Option<String>,
    pub text: Option<String>,
    pub tag: Option<TagFilter>,
}

/// Exact tag key, optionally restricted to an exact value (including empty).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagFilter {
    pub key: String,
    pub value: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceTag {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoredTagObservation {
    pub coverage: inventory_core::inventory::TagCoverage,
    pub observed_at_ms: i64,
    pub last_complete_at_ms: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredResource {
    pub key: ResourceKey,
    pub name: Option<String>,
    pub content_hash: ContentHash,
    pub record: Record,
    /// Last known persisted pairs; an empty set does not imply a complete scan.
    pub tags: Vec<ResourceTag>,
    /// The persisted tag set exceeded the reader's `MAX_PAGE_LIMIT` cap.
    pub tags_truncated: bool,
    /// Latest tag observation, independent of the retained last-known pairs.
    /// `None` means no persisted observation; it does not mean complete-empty.
    pub tag_observation: Option<StoredTagObservation>,
}

/// Lightweight row for table pages; omits the record payload to avoid loading
/// gigabytes of unused data for bounded pages. Payload is loaded lazily via
/// `Database::detail` when the user opens a specific resource.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceRow {
    pub key: ResourceKey,
    pub name: Option<String>,
    pub content_hash: ContentHash,
    /// Byte size of the persisted record payload, loaded without the payload.
    pub record_bytes: usize,
    /// Last known persisted pairs; an empty set does not imply a complete scan.
    pub tags: Vec<ResourceTag>,
    /// The persisted tag set exceeded the reader's `MAX_PAGE_LIMIT` cap.
    pub tags_truncated: bool,
    /// Latest tag observation, independent of the retained last-known pairs.
    /// `None` means no persisted observation; it does not mean complete-empty.
    pub tag_observation: Option<StoredTagObservation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageCursor {
    pub account: String,
    pub region: String,
    pub kind: String,
    pub id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageRequest {
    pub limit: usize,
    pub after: Option<PageCursor>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourcePage {
    pub resources: Vec<StoredResource>,
    pub next: Option<PageCursor>,
    pub total_matching: u64,
}

/// Lightweight page for table views; rows omit the record payload to bound
/// memory on large inventories. Payload is loaded on demand via `detail`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceRowPage {
    pub rows: Vec<ResourceRow>,
    pub next: Option<PageCursor>,
    pub total_matching: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DatabaseCounts {
    pub accounts: u64,
    pub regions: u64,
    pub services: u64,
    pub resources: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EdgeCursor {
    pub edge: GraphEdge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EdgePageRequest {
    pub limit: usize,
    pub after: Option<EdgeCursor>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EdgePage {
    pub edges: Vec<GraphEdge>,
    pub next: Option<EdgeCursor>,
    pub total_matching: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LodSummary {
    pub resources: u64,
    pub edges: u64,
    pub connected_resources: u64,
    pub isolated_resources: u64,
}

pub struct Database {
    pool: SqlitePool,
}

pub struct DatabaseSnapshot {
    tx: sqlx::Transaction<'static, sqlx::Sqlite>,
}

impl Database {
    pub async fn open(path: &Path) -> Result<Self, ViewError> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .read_only(true)
            .pragma("query_only", "ON");
        let pool = SqlitePoolOptions::new()
            .max_connections(4)
            .connect_with(options)
            .await?;
        let version = sqlx::query_scalar::<_, i64>("PRAGMA user_version")
            .fetch_one(&pool)
            .await?;
        if !matches!(version, 15 | 16) {
            return Err(ViewError::Version(version));
        }
        Ok(Self { pool })
    }

    pub async fn snapshot(&self) -> Result<DatabaseSnapshot, ViewError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query_scalar::<_, i64>("SELECT EXISTS(SELECT 1 FROM resources)")
            .fetch_one(&mut *tx)
            .await?;
        Ok(DatabaseSnapshot { tx })
    }

    pub async fn counts(&self) -> Result<DatabaseCounts, ViewError> {
        let mut snapshot = self.snapshot().await?;
        let counts = snapshot.counts().await?;
        snapshot.finish().await?;
        Ok(counts)
    }

    pub async fn page(
        &self,
        filter: &ResourceFilter,
        request: PageRequest,
    ) -> Result<ResourcePage, ViewError> {
        let mut snapshot = self.snapshot().await?;
        let page = snapshot.page(filter, request).await?;
        snapshot.finish().await?;
        Ok(page)
    }

    /// Returns lightweight rows without record payloads for table views.
    /// The record payload is loaded lazily via `detail` when needed.
    pub async fn page_rows(
        &self,
        filter: &ResourceFilter,
        request: PageRequest,
    ) -> Result<ResourceRowPage, ViewError> {
        let mut snapshot = self.snapshot().await?;
        let page = snapshot.page_rows(filter, request).await?;
        snapshot.finish().await?;
        Ok(page)
    }

    pub async fn edges_page(
        &self,
        node: &GraphNode,
        request: EdgePageRequest,
    ) -> Result<EdgePage, ViewError> {
        let mut snapshot = self.snapshot().await?;
        let page = snapshot.edges_page(node, request).await?;
        snapshot.finish().await?;
        Ok(page)
    }

    pub async fn lod_summary(&self) -> Result<LodSummary, ViewError> {
        let mut snapshot = self.snapshot().await?;
        let summary = snapshot.lod_summary().await?;
        snapshot.finish().await?;
        Ok(summary)
    }

    pub async fn detail(&self, node: &GraphNode) -> Result<Option<StoredResource>, ViewError> {
        let mut snapshot = self.snapshot().await?;
        let resource = snapshot.detail(node).await?;
        snapshot.finish().await?;
        Ok(resource)
    }
}

async fn decode_row(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    row: sqlx::sqlite::SqliteRow,
) -> Result<StoredResource, ViewError> {
    let account: String = row.try_get("account")?;
    let region: String = row.try_get("region")?;
    let kind: String = row.try_get("kind")?;
    let id: String = row.try_get("id")?;
    let expected: Vec<u8> = row.try_get("content_hash")?;
    let table = table_for(tx, &kind).await?;
    let detail = read_detail(tx, &table, &account, &region, &id).await?;
    let tags = tags::read(tx, [&account, &region, &kind, &id]).await?;
    Ok(StoredResource {
        key: resource_key([&account, &region, &kind, &id])?,
        name: row.try_get("name")?,
        content_hash: ContentHash::from_bytes(stored_hash(&expected)?),
        record: verified_record(&detail, &expected)?,
        tags: tags.tags,
        tags_truncated: tags.truncated,
        tag_observation: tags.observation,
    })
}

/// Lightweight decode for table pages: skips loading the record payload,
/// reading only its byte length. Tags are still loaded since they appear
/// in the table view.
async fn decode_row_lite(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    row: sqlx::sqlite::SqliteRow,
) -> Result<ResourceRow, ViewError> {
    let account: String = row.try_get("account")?;
    let region: String = row.try_get("region")?;
    let kind: String = row.try_get("kind")?;
    let id: String = row.try_get("id")?;
    let expected: Vec<u8> = row.try_get("content_hash")?;
    let table = table_for(tx, &kind).await?;
    let record_bytes = record_byte_length(tx, &table, &account, &region, &id).await?;
    let tags = tags::read(tx, [&account, &region, &kind, &id]).await?;
    Ok(ResourceRow {
        key: resource_key([&account, &region, &kind, &id])?,
        name: row.try_get("name")?,
        content_hash: ContentHash::from_bytes(stored_hash(&expected)?),
        record_bytes,
        tags: tags.tags,
        tags_truncated: tags.truncated,
        tag_observation: tags.observation,
    })
}

fn stored_hash(expected: &[u8]) -> Result<[u8; 32], ViewError> {
    expected.try_into().map_err(|_| ViewError::StoredRow)
}

fn verified_record(detail: &sqlx::sqlite::SqliteRow, expected: &[u8]) -> Result<Record, ViewError> {
    let stored_hash: Vec<u8> = detail.try_get("content_hash")?;
    if stored_hash != expected {
        return Err(ViewError::StoredRow);
    }
    Ok(Record::from_bytes(detail.try_get::<Vec<u8>, _>("record")?)?)
}

fn resource_key([account, region, kind, id]: [&str; 4]) -> Result<ResourceKey, ViewError> {
    Ok(ResourceKey {
        account: AccountId::parse(account)?,
        region: Region::parse(region)?,
        kind: Kind::parse(kind)?,
        id: ResourceId::parse(id)?,
    })
}
fn validate_page_limit(limit: usize) -> Result<(), ViewError> {
    if limit == 0 || limit > MAX_PAGE_LIMIT {
        return Err(ViewError::PageLimit);
    }
    Ok(())
}

async fn decode_resources(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    filter: &ResourceFilter,
    request: &PageRequest,
) -> Result<(Vec<StoredResource>, bool), ViewError> {
    let mut rows = fetch_page(tx, filter, request).await?;
    let has_more = rows.len() > request.limit;
    rows.truncate(request.limit);
    let mut resources = Vec::with_capacity(rows.len());
    for row in rows {
        resources.push(decode_row(tx, row).await?);
    }
    Ok((resources, has_more))
}

async fn decode_rows(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    filter: &ResourceFilter,
    request: &PageRequest,
) -> Result<(Vec<ResourceRow>, bool), ViewError> {
    let mut rows = fetch_page(tx, filter, request).await?;
    let has_more = rows.len() > request.limit;
    rows.truncate(request.limit);
    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        result.push(decode_row_lite(tx, row).await?);
    }
    Ok((result, has_more))
}

fn next_resource_cursor(resources: &[StoredResource], has_more: bool) -> Option<PageCursor> {
    if has_more {
        resources.last().map(cursor_of)
    } else {
        None
    }
}

fn next_row_cursor(rows: &[ResourceRow], has_more: bool) -> Option<PageCursor> {
    if has_more {
        rows.last().map(cursor_of_row)
    } else {
        None
    }
}

fn cursor_of_row(row: &ResourceRow) -> PageCursor {
    let key = &row.key;
    PageCursor {
        account: key.account.as_str().to_owned(),
        region: key.region.as_str().to_owned(),
        kind: key.kind.as_str().to_owned(),
        id: key.id.as_str().to_owned(),
    }
}

async fn decode_edges(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    node: &GraphNode,
    request: &EdgePageRequest,
) -> Result<(Vec<GraphEdge>, bool), ViewError> {
    let mut rows = fetch_node_edges(tx, node, request).await?;
    let has_more = rows.len() > request.limit;
    rows.truncate(request.limit);
    let edges = rows
        .into_iter()
        .map(decode_edge)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((edges, has_more))
}

fn next_edge_cursor(edges: &[GraphEdge], has_more: bool) -> Option<EdgeCursor> {
    if has_more {
        edges.last().map(edge_cursor)
    } else {
        None
    }
}

async fn table_for(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    kind: &str,
) -> Result<SqlName, ViewError> {
    let table: String = sqlx::query_scalar("SELECT table_name FROM kinds WHERE kind=?")
        .bind(kind)
        .fetch_one(&mut **tx)
        .await?;
    SqlName::parse(&table).map_err(|_| ViewError::SqlName)
}
