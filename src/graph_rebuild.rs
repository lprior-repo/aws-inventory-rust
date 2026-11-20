//! Rebuilds the FalkorDB projection from an existing SQLite inventory.
//!
//! Makes no AWS calls: it re-projects the stored resources and edges into the
//! graph named by `--graph-name`, replacing whatever that graph holds.

use anyhow::{Context, Result, ensure};
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_engine::inventory::{BinaryStore, ProjectionReport, run_projection};
use serde_json::{Value, json};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

const PROJECTION_BATCH: u16 = 1000;

pub async fn run(
    database: PathBuf,
    catalog: PathBuf,
    graph_url: String,
    graph_name: String,
) -> Result<Value> {
    ensure!(
        database.is_file(),
        "database does not exist: {}",
        database.display()
    );
    let bytes = read_catalog(&catalog)?;
    RuntimeCatalog::from_bytes(&bytes).context("validating runtime catalog")?;
    let store = BinaryStore::open_with_catalog(&database, &bytes).await?;
    store.request_graph_rebuild().await?;
    let last = deliver_until_settled(&store, &graph_url, &graph_name).await?;
    ensure!(
        !last.offline,
        "FalkorDB at {graph_url} is offline; graph {graph_name} was not rebuilt"
    );
    let state = store.projection_state().await?;
    ensure!(
        state.pending == 0 && !state.rebuild_required,
        "graph {graph_name} rebuild incomplete: pending={} rebuild_required={}",
        state.pending,
        state.rebuild_required
    );
    Ok(json!({
        "status": "rebuilt",
        "graph": graph_name,
        "pending": state.pending,
    }))
}

fn read_catalog(path: &Path) -> Result<Vec<u8>> {
    let limit = inventory_core::catalog::MAX_CATALOG_BYTES;
    let file = File::open(path).with_context(|| format!("opening catalog {}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(u64::try_from(limit)?.saturating_add(1))
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() <= limit,
        "runtime catalog exceeds {limit} bytes"
    );
    Ok(bytes)
}

async fn deliver_until_settled(
    store: &BinaryStore,
    endpoint: &str,
    graph: &str,
) -> Result<ProjectionReport> {
    loop {
        let report = run_projection(store, endpoint, graph, PROJECTION_BATCH).await?;
        if report.offline
            || report.rebuild_incomplete
            || report.pending == 0
            || report.acknowledged == 0
        {
            return Ok(report);
        }
    }
}
