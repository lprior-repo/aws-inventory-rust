use anyhow::Result;
use inventory_aws::model_credentials::ProfileCredentials;
use inventory_core::inventory::AccountId;
use inventory_engine::cadence::{Environment, Millis, RunOutcome, Shutdown, Wake, serve};
use inventory_engine::inventory::{BinaryStore, RunStart, last_completed_sweep, plan_run};
use inventory_sweep::{SweepConfig, SweepControl, SweepError, SweepSummary};
use std::process::ExitCode;
use tracing::{error, info};

use super::config::ServeConfig;
use super::store::{open_store, read_catalog};
use crate::cli::ServeArgs;

const EXIT_FAILED: u8 = 2;
const EXIT_INTERRUPTED: u8 = 130;

struct ProfileConfig {
    name: String,
    credentials: ProfileCredentials,
    account: tokio::sync::Mutex<Option<AccountId>>,
}

pub(super) async fn run(options: ServeArgs) -> Result<ExitCode> {
    let interval = ServeConfig::parse_interval(&options.interval)?;
    let sweep_config = sweep_config(&options);
    sweep_config.validate_transport_overrides()?;
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

    let shutdown = Shutdown::new();
    let signal = tokio::spawn({
        let shutdown = shutdown.clone();
        async move {
            match tokio::signal::ctrl_c().await {
                Ok(()) => shutdown.request(),
                Err(e) => error!(error = %e, "Ctrl-C handler unavailable"),
            }
        }
    });
    let env = ServeEnv {
        catalog_bytes,
        store,
        profiles,
        sweep_config,
        shutdown,
    };

    let result = serve(&env, interval, env.profiles.len()).await;
    signal.abort();
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
            Ok(exit_code(report.interrupted, report.failed))
        }
        Err(e) => {
            error!(error = %e, "cadence scheduler error");
            anyhow::bail!("cadence scheduler failed: {e}")
        }
    }
}

fn exit_code(interrupted: u64, failed: u64) -> ExitCode {
    if interrupted > 0 {
        ExitCode::from(EXIT_INTERRUPTED)
    } else if failed > 0 {
        ExitCode::from(EXIT_FAILED)
    } else {
        ExitCode::SUCCESS
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
            account: tokio::sync::Mutex::new(None),
        });
    }
    Ok(profiles)
}

fn sweep_config(options: &ServeArgs) -> SweepConfig {
    SweepConfig {
        region: options.region.clone(),
        max_pages: options.max_pages,
        concurrency: options.concurrency,
        // Resume-versus-new is resolved per run against the store (`plan_run`).
        resume: false,
        endpoint: options.endpoint.clone(),
        ca_bundle: options.ca_bundle.clone(),
        idle_timeout: options
            .idle_timeout_ms
            .map(std::time::Duration::from_millis),
        pool_idle_per_host: options.pool_idle_per_host,
        slice_timeout_secs: None,
        max_rps: options
            .max_rps_milli
            .filter(|rate| rate.is_multiple_of(1_000))
            .map(|rate| rate / 1_000),
        max_rps_milli: options
            .max_rps_milli
            .filter(|rate| !rate.is_multiple_of(1_000)),
        max_page_size: None,
    }
}

struct ServeEnv {
    catalog_bytes: Vec<u8>,
    store: BinaryStore,
    profiles: Vec<ProfileConfig>,
    sweep_config: SweepConfig,
    shutdown: Shutdown,
}

impl ServeEnv {
    fn catalog(&self) -> inventory_core::catalog::runtime::RuntimeCatalog<'_> {
        inventory_core::catalog::runtime::RuntimeCatalog::from_bytes(&self.catalog_bytes)
            .unwrap_or_else(|_| {
                std::process::exit(1);
            })
    }

    /// Profile identity: resolved once from the credentials, then retained.
    async fn account(&self, profile: &ProfileConfig) -> Result<AccountId, SweepError> {
        let mut cached = profile.account.lock().await;
        if let Some(account) = cached.as_ref() {
            return Ok(account.clone());
        }
        let catalog = self.catalog();
        let resolved = tokio::select! {
            biased;
            () = self.shutdown.requested() => {
                return Err(inventory_engine::limiter::ApiError::Cancelled.into());
            }
            resolved = inventory_sweep::resolve_account(
                &catalog,
                &self.sweep_config,
                &profile.credentials,
            ) => resolved?,
        };
        *cached = Some(resolved.clone());
        Ok(resolved)
    }

    async fn freshness(&self, profile: &ProfileConfig) -> Option<Millis> {
        let account = match self.account(profile).await {
            Ok(account) => account,
            Err(e) => {
                error!(profile = profile.name, error = %e, "resolving profile account");
                return None;
            }
        };
        match last_completed_sweep(&self.store, account.as_str()).await {
            Ok(Some(sweep)) => Some(Millis::new(u64::try_from(sweep.ended_at_ms).unwrap_or(0))),
            Ok(None) => None,
            Err(e) => {
                error!(
                    profile = profile.name,
                    account = account.as_str(),
                    error = %e,
                    "querying last completed sweep"
                );
                None
            }
        }
    }
}

fn classify(result: &Result<SweepSummary, SweepError>, profile: &str) -> RunOutcome {
    match result {
        Ok(summary) => {
            let outcomes = &summary.outcomes;
            if outcomes.failed > 0
                || outcomes.partial > 0
                || outcomes.inaccessible > 0
                || outcomes.unsupported > 0
            {
                error!(
                    profile,
                    failed = outcomes.failed,
                    partial = outcomes.partial,
                    inaccessible = outcomes.inaccessible,
                    unsupported = outcomes.unsupported,
                    "sweep completed with issues"
                );
                RunOutcome::Failed
            } else {
                info!(profile, "sweep completed successfully");
                RunOutcome::Completed
            }
        }
        Err(e) if e.is_cancelled() => {
            info!(profile, "sweep interrupted by shutdown");
            RunOutcome::Interrupted
        }
        Err(e) => {
            error!(profile, error = %e, "sweep failed");
            RunOutcome::Failed
        }
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

    async fn last_completed(&self, _count: usize) -> Vec<Option<Millis>> {
        let mut results = Vec::with_capacity(self.profiles.len());
        for profile in &self.profiles {
            results.push(self.freshness(profile).await);
        }
        results
    }

    fn stop_requested(&self) -> bool {
        self.shutdown.is_requested()
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
            () = tokio::time::sleep(Duration::from_millis(remaining)) => Wake::Elapsed,
            () = self.shutdown.requested() => Wake::Stop,
        }
    }

    async fn run_profile(&self, profile_index: usize) -> RunOutcome {
        let Some(profile) = self.profiles.get(profile_index) else {
            return RunOutcome::Failed;
        };
        if self.shutdown.is_requested() {
            return RunOutcome::Interrupted;
        }
        let account = match self.account(profile).await {
            Ok(account) => account,
            Err(e) if e.is_cancelled() => return RunOutcome::Interrupted,
            Err(e) => {
                error!(profile = profile.name, error = %e, "resolving profile account");
                return RunOutcome::Failed;
            }
        };
        let resume = match plan_run(&self.store, &account).await {
            Ok(RunStart::Resume(sweep)) => {
                info!(
                    profile = profile.name,
                    sweep = sweep.as_str(),
                    "resuming pending sweep"
                );
                true
            }
            Ok(RunStart::Fresh) => false,
            Ok(RunStart::Blocked(sweep)) => {
                info!(
                    profile = profile.name,
                    sweep = sweep.as_str(),
                    "pending sweep belongs to another account; deferring"
                );
                return RunOutcome::NotOwner;
            }
            Err(e) => {
                error!(profile = profile.name, error = %e, "planning sweep start");
                return RunOutcome::Failed;
            }
        };
        info!(profile = profile.name, resume, "starting sweep");
        let catalog = self.catalog();
        let config = SweepConfig {
            resume,
            ..self.sweep_config.clone()
        };
        let control = SweepControl {
            cancel: self.shutdown.token(),
            ..SweepControl::default()
        };
        let result =
            inventory_sweep::run(&catalog, &self.store, config, &profile.credentials, control)
                .await;
        classify(&result, &profile.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inventory_engine::limiter::ApiError;

    #[test]
    fn given_a_cancelled_sweep_when_classified_then_the_daemon_is_interrupted_not_failed() {
        let cancelled: Result<SweepSummary, SweepError> = Err(ApiError::Cancelled.into());
        assert_eq!(classify(&cancelled, "p"), RunOutcome::Interrupted);
    }

    #[test]
    fn given_an_ordinary_sweep_error_when_classified_then_it_is_a_failure() {
        let failed: Result<SweepSummary, SweepError> = Err(SweepError::Operation("boom"));
        assert_eq!(classify(&failed, "p"), RunOutcome::Failed);
    }

    #[test]
    fn given_interruption_and_failure_when_exiting_then_interruption_wins() {
        assert_eq!(
            format!("{:?}", exit_code(1, 3)),
            format!("{:?}", ExitCode::from(EXIT_INTERRUPTED))
        );
        assert_eq!(
            format!("{:?}", exit_code(0, 3)),
            format!("{:?}", ExitCode::from(EXIT_FAILED))
        );
    }
}
