//! Catalog and store setup for the serve command.

use anyhow::{Context, Result};
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_engine::inventory::{BinaryStore, SchemaRegistry};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn read_catalog(path: &Path) -> Result<Vec<u8>> {
    let limit = inventory_core::catalog::MAX_CATALOG_BYTES;
    let file = File::open(path).with_context(|| format!("opening catalog {}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(u64::try_from(limit).context("limit exceeds u64")?.saturating_add(1))
        .read_to_end(&mut bytes)?;
    anyhow::ensure!(bytes.len() <= limit, "runtime catalog exceeds {limit} bytes");
    Ok(bytes)
}

pub async fn open_store(db_path: &Path, catalog_bytes: &[u8]) -> Result<BinaryStore> {
    if db_path.exists() {
        BinaryStore::open_with_catalog(db_path, catalog_bytes)
            .await
            .map_err(|e| anyhow::anyhow!("opening existing store: {e}"))
    } else {
        let catalog = RuntimeCatalog::from_bytes(catalog_bytes)
            .map_err(|e| anyhow::anyhow!("building runtime catalog: {e}"))?;
        let registry = SchemaRegistry::new(catalog_bytes, catalog.schemas().to_vec())
            .map_err(|e| anyhow::anyhow!("building schema registry: {e}"))?;
        BinaryStore::create(db_path, registry)
            .await
            .map_err(|e| anyhow::anyhow!("creating new store: {e}"))
    }
}