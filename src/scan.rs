//! Owned scan futures, bounded event routing, and one SQLx inventory writer.
use crate::{
    cli::{AccountArgs, ScanArgs},
    demo,
};
use anyhow::{Context, Result};
use futures_util::{StreamExt, TryStreamExt, stream};
use inventory_aws::{
    AwsScanOptions,
    discovery::{self, AssumeRoleOptions, DiscoveryOptions, OrganizationDiscovery},
};
use inventory_core::{CollectionEvent, CollectionFailure, CollectionScope, ScanStatus, Source};
use inventory_engine::{
    limiter::{ApiExecutor, ExecutorConfig},
    store::Store,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;

const MAX_PROFILES: usize = 128;
const PROFILE_CONCURRENCY: usize = 4;
const WRITER_QUEUE: usize = 4;
const MAX_EVENTS: u64 = 1_000_000;

#[cfg(test)]
mod tests;

#[derive(Clone)]
struct Job {
    scan_id: String,
    options: AwsScanOptions,
}
struct RoutedEvent {
    scan_id: String,
    event: CollectionEvent,
}
struct Outcome {
    scan_id: String,
    profile: Option<String>,
    target_account: Option<String>,
    error: Option<String>,
}
struct PipelineResult {
    outcomes: Vec<Outcome>,
    cancelled: bool,
    write_error: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn run(options: ScanArgs) -> Result<Value> {
    crate::jobs::validate_scan(&options)?;
    let executor = configure_executor(&options).await?;
    run_with_executor(options, executor).await
}

/// The durable worker owns cancellation and lease renewal; all scan futures
/// remain under this call and its one bounded inventory writer.
#[tracing::instrument(skip_all)]
pub(crate) async fn run_with_executor(
    options: ScanArgs,
    executor: Arc<ApiExecutor>,
) -> Result<Value> {
    crate::jobs::validate_scan(&options)?;
    crate::delivery::preflight(&options.delivery).await?;
    anyhow::ensure!(
        !executor.is_cancelled(),
        "scan cancelled before target discovery"
    );
    let preparation = prepare_jobs(&options, Arc::clone(&executor));
    tokio::pin!(preparation);
    let (jobs, discovery) = tokio::select! {
        result = &mut preparation => result?,
        signal = tokio::signal::ctrl_c() => {
            if let Err(error) = signal { tracing::error!(%error, "shutdown signal failed"); }
            executor.cancel();
            let _ = preparation.await;
            anyhow::bail!("scan cancelled during target discovery");
        }
    };
    anyhow::ensure!(
        !executor.is_cancelled(),
        "scan cancelled before inventory initialization"
    );
    let store = Store::open(&options.db).await?;
    let mut result = run_and_close(store, jobs, Arc::clone(&executor)).await?;
    if let Some(discovery) = discovery {
        result
            .as_object_mut()
            .context("scan result must be an object")?
            .insert("target_discovery".into(), discovery);
    }
    deliver_scan_outputs(&options, result, executor).await
}

#[tracing::instrument(skip_all)]
async fn deliver_scan_outputs(
    options: &ScanArgs,
    result: Value,
    executor: Arc<ApiExecutor>,
) -> Result<Value> {
    if options.delivery.output_dir.is_none() {
        return Ok(result);
    }
    let cancellation = tokio_util::sync::CancellationToken::new();
    if executor.is_cancelled() {
        cancellation.cancel();
    }
    let work = crate::delivery::after_scan(&options.db, &options.delivery, result, &cancellation);
    tokio::pin!(work);
    tokio::select! {
        result = &mut work => result,
        () = executor.cancelled() => {
            cancellation.cancel();
            tokio::time::timeout(std::time::Duration::from_secs(30), work).await
                .context("delivery cleanup exceeded 30 seconds after scan cancellation")?
        }
        signal = tokio::signal::ctrl_c() => {
            executor.cancel();
            cancellation.cancel();
            if let Err(error) = signal { tracing::error!(%error, "shutdown signal failed"); }
            tokio::time::timeout(std::time::Duration::from_secs(30), work).await
                .context("delivery cleanup exceeded 30 seconds after interrupt")?
        }
    }
}

#[tracing::instrument(skip_all)]
pub(crate) async fn configure_executor(options: &ScanArgs) -> Result<Arc<ApiExecutor>> {
    let config = ExecutorConfig {
        global_concurrency: options.concurrency,
        per_bucket_concurrency: options.per_bucket_concurrency,
        requests_per_second: options.rps,
        max_attempts: options.attempts,
        ..ExecutorConfig::default()
    };
    let executor = if let Some(path) = &options.rate_policy {
        let file = tokio::fs::File::open(path).await?;
        let mut bytes = Vec::new();
        file.take(1_048_577).read_to_end(&mut bytes).await?;
        anyhow::ensure!(bytes.len() <= 1_048_576, "rate policy exceeds 1 MiB");
        let policies: BTreeMap<String, inventory_engine::limiter::BucketPolicy> =
            serde_json::from_slice(&bytes)?;
        anyhow::ensure!(policies.len() <= 4096, "rate policy exceeds 4096 entries");
        ApiExecutor::with_rate_policies(config, policies)?
    } else {
        ApiExecutor::new(config)?
    };
    Ok(Arc::new(executor))
}

#[tracing::instrument(skip_all)]
async fn run_and_close(store: Store, jobs: Vec<Job>, executor: Arc<ApiExecutor>) -> Result<Value> {
    let result = run_open(&store, jobs, executor).await;
    store.close().await;
    result
}

fn profiles(options: &ScanArgs) -> Result<Vec<Option<String>>> {
    anyhow::ensure!(
        options.profile.len() <= MAX_PROFILES,
        "at most 128 profiles per invocation"
    );
    let profiles = if options.profile.is_empty() {
        vec![None]
    } else {
        options
            .profile
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(Some)
            .collect()
    };
    Ok(profiles)
}

fn job(options: &ScanArgs, profile: Option<String>, assume_role: Option<AssumeRoleOptions>) -> Job {
    Job {
        scan_id: uuid::Uuid::new_v4().to_string(),
        options: AwsScanOptions {
            profile,
            regions: options.region.clone(),
            services: options.service.clone(),
            config_aggregator: options.config_aggregator.clone(),
            resource_explorer_view: options.resource_explorer_view.clone(),
            discover_sources: options.discover_sources,
            assume_role,
            cloud_control: options.cloud_control,
            resource_types: options.resource_type.clone(),
            max_pages: options.max_pages,
            page_size: options.page_size,
        },
    }
}

#[tracing::instrument(skip_all)]
async fn prepare_jobs(
    options: &ScanArgs,
    executor: Arc<ApiExecutor>,
) -> Result<(Vec<Job>, Option<Value>)> {
    let profiles = profiles(options)?;
    let external_id = options
        .accounts
        .external_id_env
        .as_ref()
        .map(|name| {
            std::env::var(name).with_context(|| {
                format!("external ID environment variable {name} is missing or not UTF-8")
            })
        })
        .transpose()?;
    if let Some(role_name) = &options.accounts.organization_role {
        let report = discovery::discover(
            DiscoveryOptions {
                profile: options.profile.first().cloned(),
                region: options
                    .region
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "us-east-1".into()),
                organization: true,
                hierarchy: false,
                config_aggregators: false,
                resource_explorer: false,
                max_pages: options.max_pages,
                page_size: options.page_size,
            },
            executor,
        )
        .await?;
        let organization = report
            .organization
            .context("organization discovery returned no account report")?;
        let accounts = select_accounts(&organization, &options.accounts)?;
        let jobs = accounts
            .iter()
            .map(|account| {
                let arn = discovery::account_role_arn(&report.partition, account, role_name)?;
                let role = role_options(&options.accounts, arn, external_id.clone())?;
                Ok(job(options, options.profile.first().cloned(), Some(role)))
            })
            .collect::<Result<Vec<_>>>()?;
        let summary = json!({"source_account":report.account_id,"partition":report.partition,
            "accounts_discovered":organization.accounts.len(),"eligible_accounts":organization.eligible_accounts().count(),
            "selected_accounts":accounts,"account_listing_complete":organization.account_coverage.complete,
            "role_name":role_name,"semantics":"ACTIVE organization accounts only; one immutable scan per selected role target"});
        return Ok((jobs, Some(summary)));
    }
    let role = options
        .accounts
        .role_arn
        .as_ref()
        .map(|arn| role_options(&options.accounts, arn.clone(), external_id))
        .transpose()?;
    Ok((
        profiles
            .into_iter()
            .map(|profile| job(options, profile, role.clone()))
            .collect(),
        None,
    ))
}

fn role_options(
    options: &AccountArgs,
    role_arn: String,
    external_id: Option<String>,
) -> Result<AssumeRoleOptions> {
    let role = AssumeRoleOptions {
        role_arn,
        session_name: options.role_session_name.clone(),
        external_id,
        duration_seconds: options.role_duration_seconds,
    };
    role.validate()?;
    Ok(role)
}

fn select_accounts(
    organization: &OrganizationDiscovery,
    options: &AccountArgs,
) -> Result<Vec<String>> {
    anyhow::ensure!(
        organization.account_coverage.complete,
        "organization account listing is incomplete; inspect `discover --organization` before selecting targets"
    );
    let eligible: BTreeSet<&str> = organization
        .eligible_accounts()
        .map(|account| account.id.as_str())
        .collect();
    let requested: BTreeSet<&str> = options.account_id.iter().map(String::as_str).collect();
    for account in &requested {
        anyhow::ensure!(
            eligible.contains(account),
            "requested account {account} is absent or not ACTIVE"
        );
    }
    let selected: Vec<String> = eligible
        .into_iter()
        .filter(|id| requested.is_empty() || requested.contains(id))
        .map(str::to_owned)
        .collect();
    anyhow::ensure!(
        !selected.is_empty(),
        "organization target selection is empty"
    );
    anyhow::ensure!(
        selected.len() <= options.max_accounts,
        "{} selected accounts exceeds --max-accounts {}; narrow --account-id or raise the bound explicitly",
        selected.len(),
        options.max_accounts
    );
    Ok(selected)
}

#[tracing::instrument(skip_all)]
async fn run_open(store: &Store, jobs: Vec<Job>, executor: Arc<ApiExecutor>) -> Result<Value> {
    initialize_scans(store, &jobs).await?;
    let result = pipeline(store, jobs, Arc::clone(&executor)).await;
    finalize_scans(store, result, executor).await
}

#[tracing::instrument(skip_all)]
async fn initialize_scans(store: &Store, jobs: &[Job]) -> Result<()> {
    let time = demo::now_ms()?;
    stream::iter(jobs.iter().map(Ok::<_, anyhow::Error>))
        .try_for_each(|job| async move {
            store.begin_scan(&job.scan_id, time).await?;
            Ok(())
        })
        .await
}

#[tracing::instrument(skip_all)]
async fn pipeline(store: &Store, jobs: Vec<Job>, executor: Arc<ApiExecutor>) -> PipelineResult {
    let (sender, receiver) = mpsc::channel(WRITER_QUEUE);
    let collect = supervise(jobs, Arc::clone(&executor), sender);
    let write = write_events(store, receiver, Arc::clone(&executor));
    let ((outcomes, cancelled), writes) = tokio::join!(collect, write);
    PipelineResult {
        outcomes,
        cancelled,
        write_error: writes.err().map(|error| format!("{error:#}")),
    }
}

#[tracing::instrument(skip_all)]
async fn supervise(
    jobs: Vec<Job>,
    executor: Arc<ApiExecutor>,
    sender: mpsc::Sender<RoutedEvent>,
) -> (Vec<Outcome>, bool) {
    let collection = collect_jobs(jobs, Arc::clone(&executor), sender);
    tokio::pin!(collection);
    tokio::select! {
        outcomes = &mut collection => (outcomes, false),
        signal = tokio::signal::ctrl_c() => {
            if let Err(error) = signal { tracing::error!(%error, "unable to observe shutdown signal; cancelling collection"); }
            tracing::warn!("cancelling API calls and draining persisted pages");
            executor.cancel();
            (collection.await, true)
        }
    }
}

#[tracing::instrument(skip_all)]
async fn collect_jobs(
    jobs: Vec<Job>,
    executor: Arc<ApiExecutor>,
    sender: mpsc::Sender<RoutedEvent>,
) -> Vec<Outcome> {
    stream::iter(jobs)
        .map(|job| collect_job(job, Arc::clone(&executor), sender.clone()))
        .buffer_unordered(PROFILE_CONCURRENCY)
        .collect()
        .await
}

#[tracing::instrument(skip_all, fields(scan_id = %job.scan_id))]
async fn collect_job(
    job: Job,
    executor: Arc<ApiExecutor>,
    sender: mpsc::Sender<RoutedEvent>,
) -> Outcome {
    let target_account = job
        .options
        .assume_role
        .as_ref()
        .and_then(|role| role.role_arn.split(':').nth(4))
        .map(str::to_owned);
    if executor.is_cancelled() {
        return Outcome {
            scan_id: job.scan_id,
            profile: job.options.profile,
            target_account,
            error: Some("collection cancelled before credential loading".into()),
        };
    }
    let (native_sender, native_receiver) = mpsc::channel(1);
    let profile = job.options.profile.clone();
    let collection = inventory_aws::scan(job.options, executor, native_sender);
    let forwarding = forward_events(native_receiver, &sender, &job.scan_id);
    let (collected, forwarded) = tokio::join!(collection, forwarding);
    let error = collected
        .err()
        .map(|error| error.to_string())
        .or_else(|| forwarded.err().map(|error| error.to_string()));
    if let Some(message) = &error {
        let event = startup_failure(&job.scan_id, target_account.as_deref(), message);
        if sender.send(event).await.is_err() {
            tracing::warn!(scan_id = %job.scan_id, "writer unavailable for final failure record");
        }
    }
    Outcome {
        scan_id: job.scan_id,
        profile,
        target_account,
        error,
    }
}

#[tracing::instrument(skip_all)]
async fn forward_events(
    receiver: mpsc::Receiver<CollectionEvent>,
    sender: &mpsc::Sender<RoutedEvent>,
    scan_id: &str,
) -> Result<()> {
    stream::unfold(receiver, |mut receiver| async move {
        receiver.recv().await.map(|event| (event, receiver))
    })
    .map(Ok::<_, anyhow::Error>)
    .try_for_each(|event| async {
        sender
            .send(RoutedEvent {
                scan_id: scan_id.into(),
                event,
            })
            .await
            .context("inventory writer closed")?;
        Ok(())
    })
    .await
}

fn startup_failure(scan_id: &str, target_account: Option<&str>, message: &str) -> RoutedEvent {
    RoutedEvent {
        scan_id: scan_id.into(),
        event: CollectionEvent::Failure(CollectionFailure {
            scope: CollectionScope {
                account_id: target_account.unwrap_or("unknown").into(),
                region: "global".into(),
                collector: "scan_orchestration".into(),
                parent_id: None,
                source: Source::Direct,
            },
            category: "scan_failed".into(),
            message: message.chars().take(2_048).collect(),
        }),
    }
}

#[tracing::instrument(skip_all)]
async fn write_events(
    store: &Store,
    receiver: mpsc::Receiver<RoutedEvent>,
    executor: Arc<ApiExecutor>,
) -> Result<()> {
    let result = stream::unfold(receiver, |mut receiver| async move {
        receiver.recv().await.map(|event| (event, receiver))
    })
    .map(Ok::<_, anyhow::Error>)
    .try_fold(0_u64, |count, routed| async move {
        anyhow::ensure!(
            count < MAX_EVENTS,
            "writer event ceiling reached; scan is partial"
        );
        demo::persist_event(store, &routed.scan_id, routed.event).await?;
        Ok(count.saturating_add(1))
    })
    .await;
    if result.is_err() {
        executor.cancel();
    }
    result.map(|_| ())
}

#[tracing::instrument(skip_all)]
async fn finalize_scans(
    store: &Store,
    result: PipelineResult,
    executor: Arc<ApiExecutor>,
) -> Result<Value> {
    let cancelled = result.cancelled || executor.is_cancelled();
    let status = if result.write_error.is_some() {
        ScanStatus::Partial
    } else if cancelled {
        ScanStatus::Cancelled
    } else {
        ScanStatus::Complete
    };
    let scans = stream::iter(result.outcomes.into_iter().map(Ok::<_, anyhow::Error>))
        .and_then(|outcome| finish_profile(store, outcome, status))
        .try_collect::<Vec<Value>>()
        .await?;
    let all_complete = scans
        .iter()
        .all(|scan| scan.get("status").and_then(Value::as_str) == Some("complete"));
    let overall = if result.write_error.is_some() {
        "partial"
    } else if cancelled {
        "cancelled"
    } else if all_complete {
        "complete"
    } else {
        "partial"
    };
    Ok(
        json!({"status":overall,"scans":scans,"api":executor.stats(),"write_error":result.write_error,
        "semantics":"one immutable scan per credential profile or assumed-role account target; complete means selected scopes completed, not all AWS resource types"}),
    )
}

#[tracing::instrument(skip_all)]
async fn finish_profile(store: &Store, outcome: Outcome, requested: ScanStatus) -> Result<Value> {
    let status = if outcome.error.is_some() && requested == ScanStatus::Complete {
        ScanStatus::Partial
    } else {
        requested
    };
    let summary = store
        .finish_scan(&outcome.scan_id, status, demo::now_ms()?)
        .await?;
    let actual = store.scan_status(&outcome.scan_id).await?;
    Ok(
        json!({"scan_id":outcome.scan_id,"profile":outcome.profile,"target_account":outcome.target_account,"status":actual.as_str(),"summary":summary,"error":outcome.error}),
    )
}
