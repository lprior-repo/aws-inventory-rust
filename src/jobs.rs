use anyhow::{Context, Result, ensure};
use inventory_engine::jobs::{JobSpec, JobStore};
use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

use crate::cli::{JobCommand, PlanArgs, ScanArgs};
use crate::{demo, plan};

#[tracing::instrument(skip_all)]
pub async fn execute(command: JobCommand) -> Result<Value> {
    let queue = queue_path(&command);
    if !matches!(command, JobCommand::Submit { .. }) {
        ensure!(
            queue.is_file(),
            "job queue does not exist: {}",
            queue.display()
        );
    }
    if matches!(command, JobCommand::Run { .. }) && !cfg!(feature = "aws") {
        anyhow::bail!("job execution requires an AWS-enabled build; no job was claimed");
    }
    let store = JobStore::open(queue).await?;
    let result = execute_open(&store, command).await;
    store.close().await;
    result
}

fn queue_path(command: &JobCommand) -> &Path {
    match command {
        JobCommand::Submit { queue, .. }
        | JobCommand::Run { queue, .. }
        | JobCommand::List { queue, .. }
        | JobCommand::Show { queue, .. }
        | JobCommand::Cancel { queue, .. }
        | JobCommand::Retry { queue, .. } => queue,
    }
}

#[tracing::instrument(skip_all)]
async fn execute_open(store: &JobStore, command: JobCommand) -> Result<Value> {
    match command {
        JobCommand::Submit {
            queue,
            job_id,
            max_job_attempts,
            delay_seconds,
            scan,
        } => {
            submit(
                store,
                &queue,
                job_id,
                max_job_attempts,
                delay_seconds,
                *scan,
            )
            .await
        }
        JobCommand::Run {
            limit,
            lease_seconds,
            worker,
            ..
        } => run(store, limit, lease_seconds, worker).await,
        JobCommand::List { limit, .. } => {
            let jobs = store.list(limit).await?;
            Ok(
                json!({"returned":jobs.len(),"limit":limit,"more_may_exist":jobs.len()==limit,"jobs":jobs}),
            )
        }
        JobCommand::Show { job_id, .. } => {
            let job = store.get(&job_id).await?;
            let attempts = store.history(&job_id).await?;
            Ok(json!({"job":job,"attempts":attempts}))
        }
        JobCommand::Cancel { job_id, .. } => {
            Ok(json!({"job":store.cancel(&job_id,demo::now_ms()?).await?}))
        }
        JobCommand::Retry {
            job_id,
            additional_attempts,
            ..
        } => Ok(json!({"job":store.retry(&job_id,additional_attempts,demo::now_ms()?).await?})),
    }
}

pub fn validate_scan(options: &ScanArgs) -> Result<()> {
    crate::delivery::validate(&options.delivery)?;

    let plan_args = PlanArgs {
        profile: options.profile.clone(),
        region: options.region.clone(),
        service: options.service.clone(),
        cloud_control: options.cloud_control,
        resource_type: options.resource_type.clone(),
        config_aggregator: options.config_aggregator.clone(),
        resource_explorer_view: options.resource_explorer_view.clone(),
        discover_sources: options.discover_sources,
        accounts: options.accounts.clone(),
    };
    let selection = plan_args.selection()?;
    plan::create(plan_args, &selection)?;
    ensure!(
        (1..=10000).contains(&options.max_pages) && (1..=1000).contains(&options.page_size),
        "max-pages must be 1..10000 and page-size 1..1000"
    );
    ensure!(
        !options.db.as_os_str().is_empty(),
        "empty inventory database path"
    );
    ApiExecutor::new(ExecutorConfig {
        global_concurrency: options.concurrency,
        per_bucket_concurrency: options.per_bucket_concurrency,
        requests_per_second: options.rps,
        max_attempts: options.attempts,
        ..Default::default()
    })?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn submit(
    store: &JobStore,
    queue: &Path,
    job_id: Option<String>,
    max_attempts: u32,
    delay_seconds: u64,
    mut scan: ScanArgs,
) -> Result<Value> {
    validate_scan(&scan)?;
    scan.db = absolute_path(&scan.db)?;
    ensure!(
        scan.db != absolute_path(queue)?,
        "queue and inventory must use different database files"
    );
    scan.rate_policy = scan
        .rate_policy
        .as_ref()
        .map(|p| absolute_path(p))
        .transpose()?;
    scan.delivery.output_dir = scan
        .delivery
        .output_dir
        .as_ref()
        .map(|path| absolute_path(path))
        .transpose()?;
    let delay = i64::try_from(
        delay_seconds
            .checked_mul(1000)
            .context("schedule delay overflow")?,
    )?;
    ensure!(delay <= 31_536_000_000, "job delay exceeds one year");
    let now = demo::now_ms()?;
    let available_at = now
        .checked_add(delay)
        .context("scheduled timestamp overflow")?;
    let spec = JobSpec {
        job_id: job_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        kind: "aws_scan".into(),
        payload: serde_json::to_value(scan)?,
        max_attempts,
        available_at,
    };
    Ok(
        json!({"status":"queued","job":store.submit(spec,now).await?,
        "recovery":"Expired leases create a fresh scan attempt. Partial/failed results require an explicit retry; API retries remain the shared executor's responsibility."}),
    )
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        Ok(path.canonicalize()?)
    } else {
        Ok(std::path::absolute(path)?)
    }
}

#[cfg(not(feature = "aws"))]
fn run(
    _store: &JobStore,
    _limit: usize,
    _lease_seconds: u64,
    _worker: Option<String>,
) -> std::future::Ready<Result<Value>> {
    std::future::ready(Err(anyhow::anyhow!(
        "job execution requires the aws Cargo feature"
    )))
}

#[cfg(feature = "aws")]
mod worker;
#[cfg(feature = "aws")]
use worker::run;

#[cfg(test)]
mod tests;
