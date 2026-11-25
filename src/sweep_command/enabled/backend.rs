#[cfg(test)]
mod tests;

use super::{SweepArgs, finish};
use anyhow::{Context, Result, ensure};
use inventory_aws::model_credentials::ProfileCredentials;
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_engine::inventory::{
    BinaryStore, DatabaseUrl, InventoryStore, PgStore, PgWriteConfig, SchemaRegistry,
};
use inventory_sweep::SweepConfig;
use std::{fs::File, io::Read, path::Path};

pub(super) async fn run(options: &SweepArgs) -> Result<std::process::ExitCode> {
    let bytes = read_catalog(&options.catalog)?;
    let catalog = match RuntimeCatalog::from_bytes(&bytes)?.select_services(&options.services) {
        Ok(catalog) => catalog,
        Err(error) => {
            eprintln!("invalid --service: {error}");
            return Ok(std::process::ExitCode::from(2));
        }
    };
    if options.database.starts_with("postgres://") || options.database.starts_with("postgresql://")
    {
        postgres(options, &catalog).await
    } else {
        file(options, &catalog).await
    }
}

async fn postgres(
    options: &SweepArgs,
    catalog: &RuntimeCatalog<'_>,
) -> Result<std::process::ExitCode> {
    let store = open_postgres(options, catalog).await?;
    let result = collect(&store, options, catalog, 0).await;
    store.close().await;
    result
}

async fn file(options: &SweepArgs, catalog: &RuntimeCatalog<'_>) -> Result<std::process::ExitCode> {
    let store = open_file(options, catalog).await?;
    let unattributed = store.unattributed_authority_count().await?;
    collect(&store, options, catalog, unattributed).await
}

async fn collect<S: InventoryStore>(
    store: &S,
    options: &SweepArgs,
    catalog: &RuntimeCatalog<'_>,
    unattributed: u64,
) -> Result<std::process::ExitCode> {
    let credentials =
        ProfileCredentials::load(options.profile.as_deref(), options.region.as_deref()).await?;
    let summary =
        crate::sweep_command::control::run(catalog, store, config(options), &credentials).await?;
    finish(store, options, summary, unattributed).await
}

async fn open_postgres(options: &SweepArgs, catalog: &RuntimeCatalog<'_>) -> Result<PgStore> {
    let url = DatabaseUrl::parse(&options.database)?;
    let config = postgres_config()?;
    if PgStore::exists(&url, config).await? {
        return open_current(&url, config, catalog.raw_bytes()).await;
    }
    ensure!(!options.resume, "cannot resume a missing inventory schema");
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    Ok(PgStore::create(&url, registry, config).await?)
}

fn postgres_config() -> Result<PgWriteConfig> {
    postgres_config_value(
        std::env::var("INVENTORY_POSTGRES_COPY_MIN_ROWS").ok(),
        direct_copy_environment()?,
    )
}

fn direct_copy_environment() -> Result<Option<String>> {
    const NAME: &str = "INVENTORY_POSTGRES_COPY_DIRECT";
    match std::env::var(NAME) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => {
            anyhow::bail!("{NAME} must be 1 or 0")
        }
    }
}

fn postgres_config_value(
    min_rows: Option<String>,
    direct: Option<String>,
) -> Result<PgWriteConfig> {
    const MIN_ROWS: &str = "INVENTORY_POSTGRES_COPY_MIN_ROWS";
    const DIRECT: &str = "INVENTORY_POSTGRES_COPY_DIRECT";
    let config = match min_rows {
        Some(value) => {
            let threshold = value
                .parse::<std::num::NonZeroUsize>()
                .with_context(|| format!("{MIN_ROWS} must be a positive integer"))?;
            PgWriteConfig::default().with_copy_min_rows(threshold)
        }
        None => PgWriteConfig::default(),
    };
    match direct.as_deref() {
        None | Some("0") => Ok(config),
        Some("1") => config.with_direct_copy().context(DIRECT),
        Some(_) => anyhow::bail!("{DIRECT} must be 1 or 0"),
    }
}

async fn open_current(url: &DatabaseUrl, config: PgWriteConfig, bytes: &[u8]) -> Result<PgStore> {
    let store = PgStore::open(url, config).await?;
    if let Err(error) = store.validate_catalog(bytes).await {
        store.close().await;
        return Err(error.into());
    }
    Ok(store)
}

async fn open_file(options: &SweepArgs, catalog: &RuntimeCatalog<'_>) -> Result<BinaryStore> {
    let path = Path::new(&options.database);
    if std::fs::metadata(&options.database).is_ok_and(|metadata| metadata.len() > 0) {
        return Ok(BinaryStore::open_with_catalog(path, catalog.raw_bytes()).await?);
    }
    ensure!(!options.resume, "cannot resume a missing database");
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    Ok(BinaryStore::create(path, registry).await?)
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

fn config(options: &SweepArgs) -> SweepConfig {
    SweepConfig {
        region: options.region.clone(),
        max_pages: options.max_pages,
        concurrency: options.concurrency,
        resume: options.resume,
        endpoint: options.endpoint.clone(),
        ca_bundle: options.ca_bundle.clone(),
        idle_timeout: options
            .idle_timeout_ms
            .map(std::time::Duration::from_millis),
        pool_idle_per_host: options.pool_idle_per_host,
        slice_timeout_secs: (options.slice_timeout > 0).then_some(options.slice_timeout),
        max_rps: options
            .max_rps_milli
            .filter(|rate| rate.is_multiple_of(1_000))
            .map(|rate| rate / 1_000),
        max_rps_milli: options
            .max_rps_milli
            .filter(|rate| !rate.is_multiple_of(1_000)),
        max_page_size: options.max_page_size,
    }
}
