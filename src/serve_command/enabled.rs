use anyhow::{Context, Result};
use inventory_aws::model_credentials::ProfileCredentials;
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_engine::cadence::{Environment, Millis, RunOutcome, Wake, serve};
use inventory_engine::inventory::{BinaryStore, SchemaRegistry};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{error, info};

use crate::cli::ServeArgs;
use crate::sweep_runtime::{self, SweepConfig};

struct ProfileConfig {
    name: String,
    credentials: ProfileCredentials,
    account_id: String,
}

pub(super) async fn run(options: ServeArgs) -> Result<ExitCode> {
    let interval = parse_interval(&options.interval)?;
    let catalog_bytes = read_catalog(&options.catalog)?;
    let store = open_store(&options.database, &catalog_bytes).await?;

    let mut profiles = load_profiles(&options).await?;
    if profiles.is_empty() {
        anyhow::bail!("at least one AWS profile is required; pass --profile NAME");
    }

    // Discover account IDs so we can query last-completion times from the store
    // on restart. This enables the stalest-first scheduling across restarts.
    for profile in profiles.iter_mut() {
        match discover_account(&options, &profile.name).await {
            Ok(account) => profile.account_id = account,
            Err(e) => {
                error!(
                    profile = profile.name,
                    error = %e,
                    "discovering account; will treat as stale on this interval"
                );
            }
        }
    }

    info!(
        profiles = profiles.len(),
        interval_ms = interval.as_millis(),
        "starting cadence scheduler"
    );

    let sweep_config = SweepConfig {
        region: options.region.clone(),
        max_pages: options.max_pages,
        concurrency: options.concurrency,
        resume: true,
        endpoint: options.endpoint.clone(),
        ca_bundle: options.ca_bundle.clone(),
    };

    let env = ServeEnv {
        catalog_bytes,
        store,
        profiles,
        sweep_config,
        stop: Arc::new(AtomicBool::new(false)),
    };

    let result = serve(&env, interval, env.profiles.len()).await;
    match result {
        Ok(report) => {
            info!(
                passes = report.passes,
                completed = report.completed,
                failed = report.failed,
                not_owner = report.not_owner,
                interrupted = report.interrupted,
                "cadence scheduler finished"
            );
            if report.failed > 0 {
                return Ok(ExitCode::from(2));
            }
            Ok(ExitCode::SUCCESS)
        }
        Err(e) => {
            error!(error = %e, "cadence scheduler error");
            anyhow::bail!("cadence scheduler failed: {e}")
        }
    }
}

pub fn parse_interval(input: &str) -> Result<Duration> {
    let input = input.trim();
    let (value, unit) = split_interval(input)?;
    let numeric = value.parse::<u64>().with_context(|| {
        format!("interval value '{value}' is not a number")
    })?;
    let multiplier = match unit {
        "s" => 1,
        "m" => 60,
        "h" => 3600,
        "d" => 86400,
        _ => anyhow::bail!("unknown interval unit '{unit}'"),
    };
    let total_seconds = numeric
        .checked_mul(multiplier)
        .ok_or_else(|| anyhow::anyhow!("interval exceeds u64 range"))?;
    if total_seconds == 0 {
        anyhow::bail!("interval must be at least one second");
    }
    Ok(Duration::from_secs(total_seconds))
}

fn split_interval(input: &str) -> Result<(&str, &str)> {
    if input.is_empty() {
        anyhow::bail!("interval is empty");
    }
    let digits_end = input
        .chars()
        .enumerate()
        .find(|(_, c)| !c.is_ascii_digit())
        .map(|(i, _)| i)
        .unwrap_or(input.len());
    if digits_end == 0 {
        anyhow::bail!("interval must start with a number");
    }
    let (value, unit) = input.split_at(digits_end);
    if unit.is_empty() {
        anyhow::bail!("interval must have a unit");
    }
    Ok((value, unit))
}

fn read_catalog(path: &Path) -> Result<Vec<u8>> {
    let limit = inventory_core::catalog::MAX_CATALOG_BYTES;
    let file = File::open(path).with_context(|| format!("opening catalog {}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(u64::try_from(limit).context("limit exceeds u64")?.saturating_add(1))
        .read_to_end(&mut bytes)?;
    anyhow::ensure!(bytes.len() <= limit, "runtime catalog exceeds {limit} bytes");
    Ok(bytes)
}

async fn open_store(db_path: &Path, catalog_bytes: &[u8]) -> Result<BinaryStore> {
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

async fn load_profiles(options: &ServeArgs) -> Result<Vec<ProfileConfig>> {
    let profile_names = if let Some(name) = &options.profile {
        name.split(',')
            .map(|p| p.trim().to_owned())
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>()
    } else {
        vec!["default".to_owned()]
    };

    let mut profiles = Vec::with_capacity(profile_names.len());
    for name in profile_names {
        info!(profile = name, "loading credentials");
        let credentials = ProfileCredentials::load(Some(&name), options.region.as_deref())
            .await
            .map_err(|e| anyhow::anyhow!("loading credentials for profile {name}: {e}"))?;
        profiles.push(ProfileConfig {
            name,
            credentials,
            account_id: String::new(),
        });
    }
    Ok(profiles)
}

async fn discover_account(options: &ServeArgs, profile_name: &str) -> Result<String> {
    let region = options
        .region
        .clone()
        .unwrap_or_else(|| aws_types::region::Region::from_static("us-east-1").to_string());

    let loader = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .profile_name(profile_name)
        .region(aws_types::region::Region::new(region));
    let config = loader.load().await;

    let sts = aws_sdk_sts::Client::from_conf(aws_sdk_sts::config::Builder::from(&config).build());

    let response = sts
        .get_caller_identity()
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("STS GetCallerIdentity failed: {e}"))?;

    response
        .account()
        .map(|s| s.to_owned())
        .ok_or_else(|| anyhow::anyhow!("GetCallerIdentity response missing account"))
}

struct ServeEnv {
    catalog_bytes: Vec<u8>,
    store: BinaryStore,
    profiles: Vec<ProfileConfig>,
    sweep_config: SweepConfig,
    stop: Arc<AtomicBool>,
}

impl ServeEnv {
    fn catalog(&self) -> RuntimeCatalog<'_> {
        RuntimeCatalog::from_bytes(&self.catalog_bytes)
            .expect("catalog_bytes is valid (validated at construction)")
    }
}

impl Environment for ServeEnv {
    fn now(&self) -> Millis {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::ZERO);
        let secs = elapsed.as_secs();
        let nanos = elapsed.subsec_nanos();
        let millis = (secs * 1_000).saturating_add((nanos / 1_000_000) as u64);
        Millis::new(millis)
    }

    async fn last_completed(&self, count: usize) -> Vec<Option<Millis>> {
        let mut results = Vec::with_capacity(count);
        for profile in &self.profiles {
            let account = &profile.account_id;
            if account.is_empty() {
                results.push(None);
                continue;
            }
            match inventory_engine::inventory::last_completed_sweep(&self.store, account).await {
                Ok(Some(sweep)) => {
                    let millis = Millis::new(sweep.ended_at_ms as u64);
                    results.push(Some(millis));
                }
                Ok(None) => results.push(None),
                Err(e) => {
                    error!(
                        profile = profile.name,
                        account = account,
                        error = %e,
                        "querying last completed sweep"
                    );
                    results.push(None);
                }
            }
        }
        results
    }

    fn stop_requested(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }

    async fn sleep_until(&self, until: Millis) -> Wake {
        let target_millis = until.get();
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::ZERO);
        let now_millis = (elapsed.as_secs() * 1_000)
            .saturating_add((elapsed.subsec_nanos() / 1_000_000) as u64);
        let sleep_duration =
            Duration::from_millis(target_millis.saturating_sub(now_millis));
        tokio::select! {
            _ = tokio::time::sleep(sleep_duration) => Wake::Elapsed,
            _ = tokio::signal::ctrl_c() => Wake::Stop,
        }
    }

    async fn run_profile(&self, profile_index: usize) -> RunOutcome {
        let profile = self.profiles.get(profile_index).ok_or(());
        let profile = match profile {
            Ok(p) => p,
            Err(_) => return RunOutcome::Failed,
        };
        info!(profile = profile.name, "starting sweep");
        let catalog = self.catalog();
        let result = sweep_runtime::run(
            &catalog,
            &self.store,
            self.sweep_config.clone(),
            &profile.credentials,
        )
        .await;
        match result {
            Ok(summary) => {
                if summary.outcomes.failed > 0
                    || summary.outcomes.partial > 0
                    || summary.outcomes.inaccessible > 0
                    || summary.outcomes.unsupported > 0
                {
                    error!(
                        profile = profile.name,
                        failed = summary.outcomes.failed,
                        partial = summary.outcomes.partial,
                        inaccessible = summary.outcomes.inaccessible,
                        unsupported = summary.outcomes.unsupported,
                        "sweep completed with issues"
                    );
                    RunOutcome::Failed
                } else {
                    info!(profile = profile.name, "sweep completed successfully");
                    RunOutcome::Completed
                }
            }
            Err(e) => {
                error!(profile = profile.name, error = %e, "sweep failed");
                RunOutcome::Failed
            }
        }
    }
}