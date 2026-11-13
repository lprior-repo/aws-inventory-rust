//! Bounded, observation-based analysis of finalized local inventory snapshots.
//!
//! The shell owns SQLite reads. Comparison and traversal decisions have no AWS
//! effects, and absence never becomes evidence of resource deletion.

mod diff;
mod findings;
mod graph;

#[cfg(test)]
mod tests;

use anyhow::{Context, Result};
use inventory_core::{ScanStatus, validate_identifier};
use inventory_engine::store::Store;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::io::{self, Write};
use std::path::PathBuf;

pub const MAX_DETAIL_ROWS: usize = 1_000;
pub const MAX_DIFF_ITEMS: usize = 1_000_000;
pub const MAX_TRAVERSAL_DEPTH: usize = 16;
pub const MAX_TRAVERSAL_NODES: usize = 1_000;
pub const MAX_TRAVERSAL_EDGES: usize = 4_000;
pub const MAX_ANALYSIS_OUTPUT_BYTES: usize = 16_777_216;
const DETAIL_BYTES: usize = 6_291_456;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Outgoing,
    Incoming,
    Both,
}

impl fmt::Display for Direction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Outgoing => "outgoing",
            Self::Incoming => "incoming",
            Self::Both => "both",
        })
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct TraversalOptions {
    pub direction: Direction,
    pub max_depth: usize,
    pub max_nodes: usize,
    pub max_edges: usize,
}

impl TraversalOptions {
    fn validate(self) -> Result<()> {
        anyhow::ensure!(
            self.max_depth <= MAX_TRAVERSAL_DEPTH,
            "depth must be 0..=16"
        );
        anyhow::ensure!(
            (1..=MAX_TRAVERSAL_NODES).contains(&self.max_nodes),
            "node limit must be 1..=1000"
        );
        anyhow::ensure!(
            (1..=MAX_TRAVERSAL_EDGES).contains(&self.max_edges),
            "edge limit must be 1..=4000"
        );
        Ok(())
    }
}

/// Compares selected observations using two bounded readers. Store deliberately
/// has one connection per reader pool: two live streams from one Store would
/// wait on each other. Both handles open before either stream starts.
#[tracing::instrument(skip_all)]
pub async fn diff(
    db: PathBuf,
    before_scan: String,
    after_scan: String,
    limit: usize,
    max_items: usize,
) -> Result<Value> {
    validate_diff(&before_scan, &after_scan, limit, max_items)?;
    super::require_database(&db)?;
    let before = Store::open(&db).await?;
    let after = match Store::open(db).await {
        Ok(store) => store,
        Err(error) => {
            before.close().await;
            return Err(error.into());
        }
    };
    let result = diff::report(&before, &after, &before_scan, &after_scan, limit, max_items).await;
    before.close().await;
    after.close().await;
    result.and_then(bounded_output)
}

#[tracing::instrument(skip_all)]
pub async fn neighborhood(
    db: PathBuf,
    scan_id: String,
    resource_id: String,
    options: TraversalOptions,
) -> Result<Value> {
    validate_traversal(&scan_id, &resource_id, options)?;
    super::require_database(&db)?;
    let store = Store::open(db).await?;
    let result = graph::report(&store, &scan_id, &resource_id, None, options).await;
    store.close().await;
    result.and_then(bounded_output)
}

/// Finds one deterministic breadth-first path in the requested direction.
/// A non-result is scoped to the selected snapshot, depth and work budgets.
#[tracing::instrument(skip_all)]
pub async fn path(
    db: PathBuf,
    scan_id: String,
    from_id: String,
    to_id: String,
    options: TraversalOptions,
) -> Result<Value> {
    validate_traversal(&scan_id, &from_id, options)?;
    validate_identifier(&to_id, "path target")?;
    super::require_database(&db)?;
    let store = Store::open(db).await?;
    let result = graph::report(&store, &scan_id, &from_id, Some(&to_id), options).await;
    store.close().await;
    result.and_then(bounded_output)
}

#[tracing::instrument(skip_all)]
pub async fn findings(db: PathBuf, scan_id: String, limit: usize) -> Result<Value> {
    validate_identifier(&scan_id, "scan id")?;
    validate_limit(limit)?;
    super::require_database(&db)?;
    let store = Store::open(db).await?;
    let result = findings::report(&store, &scan_id, limit).await;
    store.close().await;
    result.and_then(bounded_output)
}

fn validate_diff(before: &str, after: &str, limit: usize, max_items: usize) -> Result<()> {
    validate_identifier(before, "before scan id")?;
    validate_identifier(after, "after scan id")?;
    validate_limit(limit)?;
    anyhow::ensure!(
        (1..=MAX_DIFF_ITEMS).contains(&max_items),
        "comparison item limit must be 1..=1000000"
    );
    Ok(())
}

fn validate_traversal(scan: &str, resource: &str, options: TraversalOptions) -> Result<()> {
    validate_identifier(scan, "scan id")?;
    validate_identifier(resource, "start resource")?;
    options.validate()
}

fn validate_limit(limit: usize) -> Result<()> {
    anyhow::ensure!(
        (1..=MAX_DETAIL_ROWS).contains(&limit),
        "row limit must be 1..=1000"
    );
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn finalized(store: &Store, scan: &str) -> Result<ScanStatus> {
    validate_identifier(scan, "scan id")?;
    let status = store.scan_status(scan).await?;
    anyhow::ensure!(
        status != ScanStatus::Running,
        "analysis requires finalized scans"
    );
    Ok(status)
}

fn literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[tracing::instrument(skip_all)]
async fn query_rows(store: &Store, sql: &str, limit: usize) -> Result<Vec<Value>> {
    match store.query(sql, limit).await? {
        Value::Array(rows) => Ok(rows),
        _ => anyhow::bail!("SQLite result is not a row array"),
    }
}

#[derive(Debug, Serialize)]
struct Rows {
    rows: Vec<Value>,
    truncated: bool,
}

#[tracing::instrument(skip_all)]
async fn limited_rows(store: &Store, sql: &str, limit: usize) -> Result<Rows> {
    let probe = limit.checked_add(1).context("row probe overflow")?;
    let mut rows = query_rows(store, &format!("{sql} LIMIT {probe}"), probe).await?;
    let truncated = rows.len() > limit;
    rows.truncate(limit);
    Ok(Rows { rows, truncated })
}

fn bounded_output(value: Value) -> Result<Value> {
    encoded_len(&value)?;
    Ok(value)
}

/// Counts encoded bytes without allocating a second serialized report buffer.
fn encoded_len(value: &impl Serialize) -> Result<usize> {
    let mut counter = ByteCounter(0);
    serde_json::to_writer(&mut counter, value).context("analysis output byte bound")?;
    Ok(counter.0)
}

struct ByteCounter(usize);

impl Write for ByteCounter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let next = self
            .0
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("analysis output byte count overflow"))?;
        if next > MAX_ANALYSIS_OUTPUT_BYTES {
            return Err(io::Error::other("analysis output exceeds 16 MiB"));
        }
        self.0 = next;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
