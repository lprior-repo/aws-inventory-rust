use crate::postgres_store_config::write_config_from_env;
use anyhow::{Context, Result};
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_engine::inventory::{DatabaseUrl, PgStore, PgWriteConfig, SchemaRegistry};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn read_catalog(path: &Path) -> Result<Vec<u8>> {
    let limit = inventory_core::catalog::MAX_CATALOG_BYTES;
    let file = File::open(path).with_context(|| format!("opening catalog {}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(
        u64::try_from(limit)
            .context("limit exceeds u64")?
            .saturating_add(1),
    )
    .read_to_end(&mut bytes)?;
    anyhow::ensure!(
        bytes.len() <= limit,
        "runtime catalog exceeds {limit} bytes"
    );
    Ok(bytes)
}

pub async fn open_store(database: &str, catalog_bytes: &[u8]) -> Result<PgStore> {
    let url = DatabaseUrl::parse(database)?;
    let config = write_config_from_env()?;
    if PgStore::is_ready(&url, config).await? {
        return open_current(&url, config, catalog_bytes).await;
    }
    let catalog = RuntimeCatalog::from_bytes(catalog_bytes)
        .map_err(|e| anyhow::anyhow!("building runtime catalog: {e}"))?;
    let registry = SchemaRegistry::new(catalog_bytes, catalog.schemas().to_vec())
        .map_err(|e| anyhow::anyhow!("building schema registry: {e}"))?;
    Ok(PgStore::create(&url, registry, config).await?)
}

async fn open_current(url: &DatabaseUrl, config: PgWriteConfig, catalog: &[u8]) -> Result<PgStore> {
    let store = PgStore::open(url, config).await?;
    if let Err(error) = store.validate_catalog(catalog).await {
        store.close().await;
        return Err(error.into());
    }
    Ok(store)
}

#[cfg(test)]
mod tests {
    use super::*;
    use inventory_engine::inventory::StoreError;

    fn is_invalid_url(error: Option<&anyhow::Error>) -> bool {
        error
            .and_then(|e| e.downcast_ref::<StoreError>())
            .is_some_and(|e| matches!(e, StoreError::Invalid(_)))
    }

    #[tokio::test]
    async fn given_a_file_path_when_opening_then_the_url_is_rejected_before_any_store_access() {
        let error = open_store("inventory.bin", &[]).await.err();
        assert!(is_invalid_url(error.as_ref()));
    }

    #[tokio::test]
    async fn given_a_non_postgres_scheme_when_opening_then_the_url_is_rejected_before_any_store_access() {
        let error = open_store("sqlite:///var/lib/inventory.db", &[]).await.err();
        assert!(is_invalid_url(error.as_ref()));
    }

    #[tokio::test]
    async fn given_an_invalid_url_with_a_secret_when_opening_then_the_error_omits_the_secret() {
        let error = open_store("postgres://reader:hunter2@db.example:notaport/inventory", &[])
            .await
            .err();
        assert!(is_invalid_url(error.as_ref()));
        let text = error.map(|e| format!("{e:#}")).unwrap_or_default();
        assert!(!text.contains("hunter2"), "error leaked the secret: {text}");
        assert!(!text.contains("reader"), "error leaked the user: {text}");
    }
}
