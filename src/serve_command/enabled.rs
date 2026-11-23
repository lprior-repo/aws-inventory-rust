use anyhow::Result;
use inventory_aws::model_credentials::ProfileCredentials;
use inventory_engine::cadence::{Environment, Millis, RunOutcome, Wake, serve};
use inventory_engine::inventory::BinaryStore;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{error, info};

use super::config::ServeConfig;
use super::store::{open_store, read_catalog};
use crate::cli::ServeArgs;
use inventory_sweep::SweepConfig;

struct ProfileConfig {
    name: String,
    credentials: ProfileCredentials,
    account_id: Arc<tokio::sync::Mutex<Option<String>>>,
}

pub(super) async fn run(options: ServeArgs) -> Result<ExitCode> {
    let interval = ServeConfig::parse_interval(&options.interval)?;
    let catalog_bytes = read_catalog(&options.catalog)?;
    let store = open_store(&options.database, &catalog_bytes).await?;

    let profiles = load_profiles(&options).await?;
    if profiles.is_empty() {
        anyhow::bail!("at least one AWS profile is required; pass --profile NAME");
    }

    info!(
        profiles = profiles.len(),
        interval_ms = interval.as_millis(),
        "starting cadence scheduler"
    );

    let sweep_config = sweep_config(&options);

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
        let credentials =
            match ProfileCredentials::load(Some(&name), options.region.as_deref()).await {
                Ok(c) => c,
                Err(e) => anyhow::bail!("loading credentials for profile {name}: {e}"),
            };
        profiles.push(ProfileConfig {
            name,
            credentials,
            account_id: Arc::new(tokio::sync::Mutex::new(None)),
        });
    }
    Ok(profiles)
}

fn sweep_config(options: &ServeArgs) -> SweepConfig {
    SweepConfig {
        region: options.region.clone(),
        max_pages: options.max_pages,
        concurrency: options.concurrency,
        resume: true,
        endpoint: options.endpoint.clone(),
        ca_bundle: options.ca_bundle.clone(),
        slice_timeout_secs: None,
    }
}

struct ServeEnv {
    catalog_bytes: Vec<u8>,
    store: BinaryStore,
    profiles: Vec<ProfileConfig>,
    sweep_config: SweepConfig,
    stop: Arc<AtomicBool>,
}

impl ServeEnv {
    fn catalog(&self) -> inventory_core::catalog::runtime::RuntimeCatalog<'_> {
        inventory_core::catalog::runtime::RuntimeCatalog::from_bytes(&self.catalog_bytes)
            .unwrap_or_else(|_| {
                // Catalog was validated at construction; this is internal state.
                std::process::exit(1);
            })
    }

    async fn update_account_id(&self, profile_index: usize, account: &str) {
        let Some(profile) = self.profiles.get(profile_index) else {
            return;
        };
        let mut account_id = profile.account_id.lock().await;
        *account_id = Some(account.to_owned());
    }
}

impl Environment for ServeEnv {
    fn now(&self) -> Millis {
        use std::time::{SystemTime, UNIX_EPOCH};
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(std::time::Duration::ZERO);
        let millis = elapsed.as_millis();
        let millis_u64 = u64::try_from(millis).unwrap_or(u64::MAX);
        Millis::new(millis_u64)
    }

    async fn last_completed(&self, count: usize) -> Vec<Option<Millis>> {
        let mut results = Vec::with_capacity(count);
        for profile in &self.profiles {
            let account_id = profile.account_id.lock().await;
            let Some(account) = account_id.as_ref() else {
                results.push(None);
                continue;
            };
            match inventory_engine::inventory::last_completed_sweep(&self.store, account).await {
                Ok(Some(sweep)) => {
                    let millis = sweep.ended_at_ms.try_into().unwrap_or(0);
                    results.push(Some(Millis::new(millis)));
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
        use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let target_millis = until.get();
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);
        let now_millis = u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX);
        let remaining = target_millis.saturating_sub(now_millis);
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(remaining)) => Wake::Elapsed,
            _ = tokio::signal::ctrl_c() => Wake::Stop,
        }
    }

    async fn run_profile(&self, profile_index: usize) -> RunOutcome {
        let profile = self.profiles.get(profile_index);
        let Some(profile) = profile else {
            return RunOutcome::Failed;
        };
        info!(profile = profile.name, "starting sweep");
        let catalog = self.catalog();
        let result = crate::sweep_command::control::run(
            &catalog,
            &self.store,
            self.sweep_config.clone(),
            &profile.credentials,
        )
        .await;
        match result {
            Ok(summary) => {
                // Discover the account from the completed sweep
                let account = match self.store.discover_active_sweep().await {
                    Ok(Some(id)) => match self.store.sweep_manifest_account(&id).await {
                        Ok(Some(acc)) => Some(acc),
                        Ok(None) => None,
                        Err(e) => {
                            error!(error = %e, "reading sweep manifest account");
                            None
                        }
                    },
                    Ok(None) => None,
                    Err(e) => {
                        error!(error = %e, "discovering active sweep");
                        None
                    }
                };
                if let Some(acc) = account {
                    self.update_account_id(profile_index, &acc).await;
                }
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
