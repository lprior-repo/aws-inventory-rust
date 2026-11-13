//! Independent persisted-row and schema snapshot oracle.
use super::*;

pub(super) type StoredResource = (
    String,
    i64,
    String,
    String,
    String,
    String,
    Option<String>,
    String,
    i64,
    String,
);
pub(super) type StoredEdge = (String, i64, String, String, String, String, i64, String);
pub(super) type StoredPage = (String, i64, Vec<u8>, Option<String>, i64, i64);
pub(super) type StoredScope = (
    String,
    String,
    String,
    String,
    Option<String>,
    String,
    i64,
    Option<String>,
    i64,
    i64,
);
pub(super) type SchemaObject = (String, String, String, Option<String>);

#[derive(Debug, PartialEq, Eq)]
pub(super) struct InventorySnapshot {
    pub(super) scan: (i64, Option<i64>, String),
    pub(super) resources: Vec<StoredResource>,
    pub(super) edges: Vec<StoredEdge>,
    pub(super) pages: Vec<StoredPage>,
    pub(super) scopes: Vec<StoredScope>,
    pub(super) failures: Vec<(String, String, String)>,
    pub(super) graph: Vec<(String, String, i64, i64, i64)>,
}

pub(super) async fn inventory_snapshot(
    connection: &mut SqliteConnection,
) -> Result<InventorySnapshot, TestError> {
    Ok(InventorySnapshot {
        scan: sqlx::query_as("SELECT started_at,finished_at,status FROM scans WHERE scan_id='populated'")
            .fetch_one(&mut *connection).await?,
        resources: sqlx::query_as("SELECT scope_key,page_number,resource_id,account_id,region,kind,name,source,observed_at,resource_json
            FROM resource_observations WHERE scan_id='populated' ORDER BY scope_key,resource_id")
            .fetch_all(&mut *connection).await?,
        edges: sqlx::query_as("SELECT scope_key,page_number,from_id,to_id,relation,source,observed_at,edge_json
            FROM edge_observations WHERE scan_id='populated' ORDER BY scope_key,from_id,to_id,relation")
            .fetch_all(&mut *connection).await?,
        pages: sqlx::query_as("SELECT scope_key,page_number,fingerprint,next_token,resource_count,edge_count
            FROM collection_pages WHERE scan_id='populated' ORDER BY scope_key,page_number")
            .fetch_all(&mut *connection).await?,
        scopes: sqlx::query_as("SELECT scope_key,account_id,region,collector,parent_id,source,last_page,next_token,complete,failed
            FROM scope_checkpoints WHERE scan_id='populated' ORDER BY scope_key")
            .fetch_all(&mut *connection).await?,
        failures: sqlx::query_as("SELECT scope_key,category,message FROM collection_failures WHERE scan_id='populated'
            ORDER BY scope_key,category,message").fetch_all(&mut *connection).await?,
        graph: sqlx::query_as("SELECT target,snapshot_id,resources_exported,edges_exported,complete
            FROM graph_export_checkpoints WHERE scan_id='populated' ORDER BY target,snapshot_id")
            .fetch_all(&mut *connection).await?,
    })
}

pub(super) async fn schema_snapshot(
    connection: &mut SqliteConnection,
) -> Result<Vec<SchemaObject>, sqlx::Error> {
    sqlx::query_as("SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name")
        .fetch_all(connection)
        .await
}
