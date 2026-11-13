//! Durable job commands. Persisted specifications contain references, not credentials.
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
    plan::create(PlanArgs {
        profile: options.profile.clone(),
        region: options.region.clone(),
        service: options.service.clone(),
        cloud_control: options.cloud_control,
        resource_type: options.resource_type.clone(),
        config_aggregator: options.config_aggregator.clone(),
        resource_explorer_view: options.resource_explorer_view.clone(),
        discover_sources: options.discover_sources,
        accounts: options.accounts.clone(),
    })?;
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
mod worker {
    use super::*;
    use futures_util::{StreamExt, TryStreamExt, future::BoxFuture, stream};
    use inventory_engine::jobs::{Completion, JobRecord, JobState, Lease};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio_util::sync::CancellationToken;

    const MAX_CLAIMS: usize = 1024;
    const MAX_HEARTBEATS: usize = 32768;
    const MAX_JOB_RUNTIME: Duration = Duration::from_secs(86400);
    const MAX_DRAIN_RUNTIME: Duration = Duration::from_secs(30);
    const MAX_SUMMARY_BYTES: usize = 4096;
    const MAX_WORKER_OUTPUT_BYTES: usize = 4_194_304;

    struct Outcomes {
        rows: Vec<Value>,
        bytes: usize,
        stop_reason: Option<&'static str>,
    }

    impl Outcomes {
        fn new(limit: usize) -> Result<Self> {
            let mut rows = Vec::new();
            rows.try_reserve(limit)
                .context("reserve worker summaries")?;
            Ok(Self {
                rows,
                bytes: 0,
                stop_reason: None,
            })
        }

        fn record(&mut self, outcome: Value) -> Result<()> {
            inventory_core::validate_serialized_size(&outcome, MAX_SUMMARY_BYTES, "job summary")?;
            let bytes = serde_json::to_vec(&outcome)?.len();
            self.bytes = self
                .bytes
                .checked_add(bytes)
                .context("worker result byte count overflow")?;
            ensure!(
                self.bytes <= MAX_WORKER_OUTPUT_BYTES.saturating_sub(8192),
                "worker result byte budget exceeded; complete results remain in the queue"
            );
            if outcome.get("state").and_then(Value::as_str) == Some("cancelled") {
                self.stop_reason = Some("cancelled_job");
            }
            self.rows.push(outcome);
            Ok(())
        }

        fn report(self, owner: &str) -> Result<Value> {
            let cancelled = matches!(self.stop_reason, Some("cancelled_job" | "worker_shutdown"));
            let partial = self
                .rows
                .iter()
                .any(|row| row.get("state").and_then(Value::as_str) != Some("succeeded"));
            let state = if cancelled {
                "cancelled"
            } else if partial {
                "partial"
            } else {
                "complete"
            };
            let value = json!({"status":state,"worker":owner,"executed":self.rows.len(),"jobs":self.rows,
                "stop_reason":self.stop_reason.unwrap_or("claim_limit"),
                "mode":"bounded worker invocation; no background daemon remains",
                "results":"Terminal results are retained by job ID in the queue; oversized results declare result_truncated. This response contains bounded summaries."});
            inventory_core::validate_serialized_size(
                &value,
                MAX_WORKER_OUTPUT_BYTES,
                "worker output",
            )?;
            Ok(value)
        }
    }

    #[tracing::instrument(skip_all)]
    pub(super) async fn run(
        store: &JobStore,
        limit: usize,
        lease_seconds: u64,
        worker: Option<String>,
    ) -> Result<Value> {
        ensure!(
            (1..=MAX_CLAIMS).contains(&limit),
            "job limit must be 1..1024"
        );
        ensure!(
            (10..=3600).contains(&lease_seconds),
            "lease-seconds must be 10..3600"
        );
        let lease_ms = i64::try_from(lease_seconds.checked_mul(1000).context("lease overflow")?)?;
        let owner = worker.unwrap_or_else(|| format!("worker-{}", uuid::Uuid::new_v4()));
        let shutdown = CancellationToken::new();
        let work: BoxFuture<'_, Result<Value>> =
            Box::pin(run_queue(store, limit, lease_ms, &owner, &shutdown));
        tokio::pin!(work);
        tokio::select! {
            result = &mut work => result,
            signal = tokio::signal::ctrl_c() => {
                if let Err(error) = signal { tracing::error!(%error, "worker shutdown signal failed; stopping intake"); }
                shutdown.cancel();
                work.await
            }
        }
    }

    #[tracing::instrument(skip_all)]
    async fn run_queue(
        store: &JobStore,
        limit: usize,
        lease_ms: i64,
        owner: &str,
        shutdown: &CancellationToken,
    ) -> Result<Value> {
        let mut outcomes = stream::iter(0..limit)
            .map(Ok::<_, anyhow::Error>)
            .try_fold(Outcomes::new(limit)?, |mut results, _| async move {
                if results.stop_reason.is_some() {
                    return Ok(results);
                }
                if shutdown.is_cancelled() {
                    results.stop_reason = Some("worker_shutdown");
                    return Ok(results);
                }
                let token = uuid::Uuid::new_v4().to_string();
                let Some(lease) = store
                    .claim(owner, &token, demo::now_ms()?, lease_ms)
                    .await?
                else {
                    results.stop_reason = Some("queue_empty");
                    return Ok(results);
                };
                results.record(run_claim(store, lease, lease_ms, shutdown).await?)?;
                Ok(results)
            })
            .await?;
        if shutdown.is_cancelled() && outcomes.stop_reason != Some("cancelled_job") {
            outcomes.stop_reason = Some("worker_shutdown");
        }
        outcomes.report(owner)
    }

    #[tracing::instrument(skip_all, fields(job_id=%lease.job.job_id, attempt=lease.job.attempt))]
    async fn run_claim(
        store: &JobStore,
        lease: Lease,
        lease_ms: i64,
        shutdown: &CancellationToken,
    ) -> Result<Value> {
        if shutdown.is_cancelled() {
            return cancel_before_scan(store, &lease).await;
        }
        let prepared = tokio::select! {
            biased;
            () = shutdown.cancelled() => return cancel_before_scan(store, &lease).await,
            result = tokio::time::timeout(Duration::from_secs(5), prepare_claim(&lease)) => {
                result.map_err(|_|anyhow::anyhow!("job configuration preparation timed out")).and_then(|value| value)
            }
        };
        let (options, executor) = match prepared {
            Ok(value) => value,
            Err(error) => return terminal_error(store, &lease, error).await,
        };
        if store.heartbeat(&lease, demo::now_ms()?, lease_ms).await? || shutdown.is_cancelled() {
            return cancel_before_scan(store, &lease).await;
        }
        // One owned future per admitted job bounds the compiler's composed
        // Send proof and keeps the scan under this supervisor's cancellation.
        let scan: BoxFuture<'_, Result<Value>> = Box::pin(crate::scan::run_with_executor(
            options,
            Arc::clone(&executor),
        ));
        let supervised = supervise(
            store,
            &lease,
            lease_ms,
            Arc::clone(&executor),
            shutdown,
            scan,
        )
        .await;
        let completion = match supervised {
            Ok(value) => Completion {
                state: completion_state(&value, executor.is_cancelled()),
                result: Some(compact_result(value)),
                error: None,
            },
            Err(error) => Completion {
                state: if executor.is_cancelled() {
                    JobState::Cancelled
                } else {
                    JobState::Failed
                },
                result: None,
                error: Some(format!("{error:#}")),
            },
        };
        let done = store.complete(&lease, completion, demo::now_ms()?).await?;
        Ok(job_summary(&done))
    }

    fn completion_state(value: &Value, cancelled: bool) -> JobState {
        if cancelled {
            return JobState::Cancelled;
        }
        match value.get("status").and_then(Value::as_str) {
            Some("complete") => JobState::Succeeded,
            Some("cancelled") => JobState::Cancelled,
            _ => JobState::Partial,
        }
    }

    fn job_summary(job: &JobRecord) -> Value {
        let error = job
            .last_error
            .as_ref()
            .map(|value| value.chars().take(256).collect::<String>());
        let error_truncated = job
            .last_error
            .as_ref()
            .zip(error.as_ref())
            .is_some_and(|(full, brief)| full.len() > brief.len());
        let scan_count = job.result.as_ref().and_then(|value| {
            value.get("scan_count").and_then(Value::as_u64).or_else(|| {
                value
                    .get("scans")
                    .and_then(Value::as_array)
                    .and_then(|scans| u64::try_from(scans.len()).ok())
            })
        });
        let result_truncated = job
            .result
            .as_ref()
            .and_then(|value| value.get("result_truncated"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        json!({"job_id":job.job_id,"state":job.state,"attempt":job.attempt,
            "scan_count":scan_count,"result_available":job.result.is_some(),"error":error,
            "error_truncated":error_truncated,"result_truncated":result_truncated})
    }

    fn compact_result(value: Value) -> Value {
        if inventory_core::validate_serialized_size(
            &value,
            inventory_engine::jobs::MAX_RESULT_BYTES,
            "job result",
        )
        .is_ok()
        {
            return value;
        }
        let scan_count = value
            .get("scans")
            .and_then(Value::as_array)
            .map_or(0, Vec::len);
        let scans: Vec<Value> = value
            .get("scans")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .take(crate::targets::MAX_TARGET_ACCOUNTS)
            .map(compact_scan)
            .collect();
        let status = bounded_text(value.get("status"), 32);
        let compact = json!({"status":status,"scans":scans,"scan_count":scan_count,
            "result_truncated":true,"details":"Full observations and scope failures remain in the inventory database."});
        if inventory_core::validate_serialized_size(
            &compact,
            inventory_engine::jobs::MAX_RESULT_BYTES,
            "compacted job result",
        )
        .is_ok()
        {
            return compact;
        }
        json!({"status":status,"scans":[],"scan_count":scan_count,
            "result_truncated":true,"summaries_omitted":true,
            "details":"Terminal summaries exceeded the result budget. Observations and scope failures remain in the inventory database."})
    }

    fn compact_scan(scan: &Value) -> Value {
        let count = |key: &str| {
            scan.get("summary")
                .and_then(|summary| summary.get(key))
                .and_then(Value::as_u64)
        };
        json!({"scan_id":bounded_text(scan.get("scan_id"),256),
        "status":bounded_text(scan.get("status"),32),"summary":{
            "resources":count("resources"),"edges":count("edges"),
            "completed_scopes":count("completed_scopes"),"failed_scopes":count("failed_scopes")
        }})
    }

    fn bounded_text(value: Option<&Value>, bytes: usize) -> Option<&str> {
        value
            .and_then(Value::as_str)
            .filter(|value| value.len() <= bytes)
    }

    #[tracing::instrument(skip_all)]
    async fn prepare_claim(lease: &Lease) -> Result<(ScanArgs, Arc<ApiExecutor>)> {
        ensure!(lease.job.kind == "aws_scan", "unsupported queued job kind");
        let payload = lease
            .job
            .payload
            .clone()
            .context("queued job lacks payload")?;
        let options: ScanArgs = serde_json::from_value(payload)?;
        validate_scan(&options)?;
        let executor = crate::scan::configure_executor(&options).await?;
        Ok((options, executor))
    }

    #[tracing::instrument(skip_all)]
    async fn terminal_error(
        store: &JobStore,
        lease: &Lease,
        error: anyhow::Error,
    ) -> Result<Value> {
        let done = store
            .complete(
                lease,
                Completion {
                    state: JobState::Failed,
                    result: None,
                    error: Some(format!("{error:#}")),
                },
                demo::now_ms()?,
            )
            .await?;
        Ok(job_summary(&done))
    }

    #[tracing::instrument(skip_all)]
    async fn cancel_before_scan(store: &JobStore, lease: &Lease) -> Result<Value> {
        let completion = Completion {
            state: JobState::Cancelled,
            result: None,
            error: Some("worker stopped before AWS scan admission".into()),
        };
        Ok(job_summary(
            &store.complete(lease, completion, demo::now_ms()?).await?,
        ))
    }

    #[tracing::instrument(skip_all)]
    async fn supervise<F>(
        store: &JobStore,
        lease: &Lease,
        lease_ms: i64,
        executor: Arc<ApiExecutor>,
        shutdown: &CancellationToken,
        scan: F,
    ) -> Result<Value>
    where
        F: std::future::Future<Output = Result<Value>>,
    {
        let pulse = heartbeats(store, lease, lease_ms, Arc::clone(&executor));
        supervise_futures(
            executor,
            shutdown,
            scan,
            pulse,
            MAX_JOB_RUNTIME,
            MAX_DRAIN_RUNTIME,
        )
        .await
    }

    #[tracing::instrument(skip_all)]
    async fn supervise_futures<F, H>(
        executor: Arc<ApiExecutor>,
        shutdown: &CancellationToken,
        scan: F,
        pulse: H,
        runtime: Duration,
        drain: Duration,
    ) -> Result<Value>
    where
        F: std::future::Future<Output = Result<Value>>,
        H: std::future::Future<Output = Result<()>>,
    {
        tokio::pin!(scan);
        tokio::pin!(pulse);
        tokio::select! {
            biased;
            result = &mut scan => result,
            lease_result = &mut pulse => {
                executor.cancel();
                let drained = drain_until(&mut scan, drain_deadline(drain)?).await;
                lease_and_drain(lease_result, drained)
            },
            () = shutdown.cancelled() => {
                executor.cancel();
                drain_with_heartbeats(&mut scan, &mut pulse, drain_deadline(drain)?).await
            },
            () = executor.cancelled() => {
                drain_with_heartbeats(&mut scan, &mut pulse, drain_deadline(drain)?).await
            },
            () = tokio::time::sleep(runtime) => {
                executor.cancel();
                let mut value = drain_with_heartbeats(&mut scan, &mut pulse, drain_deadline(drain)?).await?;
                if let Some(object) = value.as_object_mut() {
                    object.insert("worker_timeout".into(), json!(true));
                    object.insert("status".into(), json!("cancelled"));
                }
                Ok(value)
            }
        }
    }

    fn drain_deadline(drain: Duration) -> Result<tokio::time::Instant> {
        tokio::time::Instant::now()
            .checked_add(drain)
            .context("scan drain deadline overflow")
    }

    #[tracing::instrument(skip_all)]
    async fn drain_with_heartbeats<F, H>(
        scan: F,
        pulse: H,
        deadline: tokio::time::Instant,
    ) -> Result<Value>
    where
        F: std::future::Future<Output = Result<Value>>,
        H: std::future::Future<Output = Result<()>>,
    {
        tokio::pin!(scan);
        tokio::pin!(pulse);
        tokio::select! {
            biased;
            result = drain_until(&mut scan, deadline) => result,
            lease_result = &mut pulse => {
                let drained = drain_until(&mut scan, deadline).await;
                lease_and_drain(lease_result, drained)
            }
        }
    }

    #[tracing::instrument(skip_all)]
    async fn drain_until<F>(scan: F, deadline: tokio::time::Instant) -> Result<Value>
    where
        F: std::future::Future<Output = Result<Value>>,
    {
        tokio::time::timeout_at(deadline, scan).await
            .map_err(|_| anyhow::anyhow!("cancelled scan exceeded its drain deadline; queued or in-flight inventory finalization may be incomplete"))?
    }

    fn lease_and_drain(lease: Result<()>, drained: Result<Value>) -> Result<Value> {
        match (lease, drained) {
            (Err(lease), Err(drain)) => Err(anyhow::anyhow!(
                "lease supervision failed: {lease:#}; {drain:#}"
            )),
            (Err(lease), Ok(_)) => Err(lease.context("lease supervision failed; scan drained")),
            (Ok(()), Err(drain)) => Err(drain
                .context("lease heartbeat loop ended unexpectedly; scan cancellation requested")),
            (Ok(()), Ok(_)) => Err(anyhow::anyhow!(
                "lease heartbeat loop ended unexpectedly; scan cancellation requested"
            )),
        }
    }

    #[tracing::instrument(skip_all)]
    async fn heartbeats(
        store: &JobStore,
        lease: &Lease,
        lease_ms: i64,
        executor: Arc<ApiExecutor>,
    ) -> Result<()> {
        let interval = u64::try_from(lease_ms.checked_div(3).context("heartbeat division")?)?;
        stream::iter(0..MAX_HEARTBEATS)
            .map(Ok::<_, anyhow::Error>)
            .try_for_each(|_| async {
                tokio::time::sleep(Duration::from_millis(interval)).await;
                if store.heartbeat(lease, demo::now_ms()?, lease_ms).await? {
                    executor.cancel();
                }
                Ok(())
            })
            .await?;
        anyhow::bail!("heartbeat budget exhausted")
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::sync::atomic::{AtomicBool, Ordering};

        fn executor() -> Result<Arc<ApiExecutor>> {
            Ok(Arc::new(ApiExecutor::new(ExecutorConfig::default())?))
        }

        #[tokio::test]
        async fn stopped_worker_does_not_claim_queued_jobs_or_load_aws_configuration() -> Result<()>
        {
            let store = JobStore::open(":memory:").await?;
            for job_id in ["first", "second"] {
                store
                    .submit(
                        JobSpec {
                            job_id: job_id.into(),
                            kind: "aws_scan".into(),
                            payload: json!({"intentionally_invalid_scan":true}),
                            max_attempts: 1,
                            available_at: 0,
                        },
                        0,
                    )
                    .await?;
            }
            let shutdown = CancellationToken::new();
            shutdown.cancel();
            let report = run_queue(&store, 2, 10_000, "stopped", &shutdown).await?;
            assert_eq!(report["executed"], 0);
            assert_eq!(report["status"], "cancelled");
            assert_eq!(report["stop_reason"], "worker_shutdown");
            for job_id in ["first", "second"] {
                let job = store.get(job_id).await?;
                assert_eq!(job.state, JobState::Queued);
                assert_eq!(job.attempt, 0);
            }
            store.close().await;
            Ok(())
        }

        #[tokio::test(start_paused = true)]
        async fn worker_shutdown_bounds_an_unresponsive_scan_and_drops_it() -> Result<()> {
            struct DropMark(Arc<AtomicBool>);
            impl Drop for DropMark {
                fn drop(&mut self) {
                    self.0.store(true, Ordering::SeqCst);
                }
            }
            let dropped = Arc::new(AtomicBool::new(false));
            let marker = DropMark(Arc::clone(&dropped));
            let scan = async move {
                let _marker = marker;
                std::future::pending::<Result<Value>>().await
            };
            let shutdown = CancellationToken::new();
            shutdown.cancel();
            let executor = executor()?;
            let start = tokio::time::Instant::now();
            let error = supervise_futures(
                Arc::clone(&executor),
                &shutdown,
                scan,
                std::future::pending(),
                Duration::from_secs(100),
                Duration::from_secs(2),
            )
            .await
            .err()
            .context("drain must time out")?;
            assert!(error.to_string().contains("drain deadline"));
            assert!(executor.is_cancelled());
            assert!(dropped.load(Ordering::SeqCst));
            assert_eq!(
                tokio::time::Instant::now().duration_since(start),
                Duration::from_secs(2)
            );
            Ok(())
        }

        #[tokio::test(start_paused = true)]
        async fn lease_loss_preserves_error_and_bounds_scan_drain() -> Result<()> {
            let executor = executor()?;
            let shutdown = CancellationToken::new();
            let error = supervise_futures(
                Arc::clone(&executor),
                &shutdown,
                std::future::pending(),
                std::future::ready(Err(anyhow::anyhow!("synthetic lost lease"))),
                Duration::from_secs(100),
                Duration::from_secs(2),
            )
            .await
            .err()
            .context("lost lease must fail")?;
            assert!(format!("{error:#}").contains("synthetic lost lease"));
            assert!(format!("{error:#}").contains("drain deadline"));
            assert!(executor.is_cancelled());
            Ok(())
        }

        #[tokio::test(start_paused = true)]
        async fn timeout_cancels_and_allows_a_cooperative_scan_to_finalize() -> Result<()> {
            let executor = executor()?;
            let scan_executor = Arc::clone(&executor);
            let scan = async move {
                scan_executor.cancelled().await;
                tokio::time::sleep(Duration::from_millis(20)).await;
                Ok(json!({"status":"cancelled","scans":[],"finalized":true}))
            };
            let result = supervise_futures(
                executor,
                &CancellationToken::new(),
                scan,
                std::future::pending(),
                Duration::from_secs(1),
                Duration::from_secs(2),
            )
            .await?;
            assert_eq!(result["worker_timeout"], true);
            assert_eq!(result["finalized"], true);
            assert_eq!(result["status"], "cancelled");
            Ok(())
        }

        #[tokio::test(start_paused = true)]
        async fn heartbeat_future_keeps_running_while_cancelled_work_drains() -> Result<()> {
            let executor = executor()?;
            let shutdown = CancellationToken::new();
            shutdown.cancel();
            let pulsed = Arc::new(AtomicBool::new(false));
            let pulsed_copy = Arc::clone(&pulsed);
            let pulse = async move {
                tokio::time::sleep(Duration::from_millis(10)).await;
                pulsed_copy.store(true, Ordering::SeqCst);
                std::future::pending::<Result<()>>().await
            };
            let scan = async {
                tokio::time::sleep(Duration::from_millis(20)).await;
                Ok(json!({"status":"cancelled"}))
            };
            let result = supervise_futures(
                executor,
                &shutdown,
                scan,
                pulse,
                Duration::from_secs(1),
                Duration::from_secs(2),
            )
            .await?;
            assert!(pulsed.load(Ordering::SeqCst));
            assert_eq!(result["status"], "cancelled");
            Ok(())
        }

        #[tokio::test(start_paused = true)]
        async fn completed_scan_does_not_wait_for_heartbeat_or_deadline() -> Result<()> {
            let executor = executor()?;
            let result = supervise_futures(
                Arc::clone(&executor),
                &CancellationToken::new(),
                std::future::ready(Ok(json!({"status":"complete"}))),
                std::future::pending(),
                Duration::from_secs(100),
                Duration::from_secs(2),
            )
            .await?;
            assert_eq!(result["status"], "complete");
            assert!(!executor.is_cancelled());
            Ok(())
        }

        #[test]
        fn cancellation_wins_a_success_shaped_completion() {
            assert_eq!(
                completion_state(&json!({"status":"complete"}), true),
                JobState::Cancelled
            );
            assert_eq!(
                completion_state(&json!({"status":"complete"}), false),
                JobState::Succeeded
            );
        }

        #[test]
        fn oversized_results_retain_bounded_scan_summaries_and_explicit_counts() -> Result<()> {
            let small = json!({"status":"complete","scans":[],"api":{"attempts":1}});
            assert_eq!(compact_result(small.clone()), small);
            let oversized = json!({"status":"partial","unneeded":"x".repeat(1_048_576),
            "scans":[{"scan_id":"one","status":"partial","summary":{
                "resources":12,"edges":3,"completed_scopes":2,"failed_scopes":1,
                "unexpected":"x".repeat(1_048_576)
            }}]});
            let compact = compact_result(oversized);
            inventory_core::validate_serialized_size(
                &compact,
                inventory_engine::jobs::MAX_RESULT_BYTES,
                "test result",
            )?;
            assert_eq!(compact["result_truncated"], true);
            assert_eq!(compact["scan_count"], 1);
            assert_eq!(compact["scans"][0]["summary"]["resources"], 12);
            assert!(compact["scans"][0]["summary"].get("unexpected").is_none());
            assert!(compact.get("unneeded").is_none());

            let maximum = json!({"status":"partial","scans":
                (0..crate::targets::MAX_TARGET_ACCOUNTS).map(|_| json!({
                    "scan_id":"\u{1}".repeat(256),"status":"partial","summary":{}
                })).collect::<Vec<_>>()});
            let fallback = compact_result(maximum);
            inventory_core::validate_serialized_size(
                &fallback,
                inventory_engine::jobs::MAX_RESULT_BYTES,
                "test fallback",
            )?;
            assert_eq!(fallback["summaries_omitted"], true);
            assert_eq!(fallback["scans"], json!([]));
            assert_eq!(fallback["scan_count"], crate::targets::MAX_TARGET_ACCOUNTS);
            Ok(())
        }

        #[test]
        fn summaries_omit_full_results_bound_error_text_and_stop_on_cancellation() -> Result<()> {
            let job = JobRecord {
                job_id: "summary".into(),
                kind: "aws_scan".into(),
                state: JobState::Cancelled,
                attempt: 1,
                max_attempts: 2,
                available_at: 0,
                created_at: 0,
                updated_at: 1,
                lease_owner: None,
                lease_until: None,
                cancel_requested: true,
                last_error: Some("🦀".repeat(2048)),
                payload: None,
                result: Some(json!({"large":"x".repeat(1_000_000),"scans":[{"scan_id":"one"}]})),
            };
            let row = job_summary(&job);
            assert!(row.get("result").is_none());
            assert_eq!(row["scan_count"], 1);
            assert_eq!(row["error_truncated"], true);
            assert_eq!(row["error"].as_str().map(str::len), Some(1024));
            let mut outcomes = Outcomes::new(1)?;
            outcomes.record(row)?;
            assert_eq!(outcomes.stop_reason, Some("cancelled_job"));
            assert_eq!(outcomes.report("worker")?["status"], "cancelled");
            Ok(())
        }
    }
}
#[cfg(feature = "aws")]
use worker::run;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Command};
    use clap::Parser;

    #[tokio::test]
    async fn queue_submission_is_offline_and_persists_paths_without_external_id_value() -> Result<()>
    {
        let folder =
            std::env::temp_dir().join(format!("inventory-job-cli-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir(&folder).await?;
        let queue = folder.join("jobs.sqlite");
        let db = folder.join("inventory.sqlite");
        let cli = Cli::try_parse_from([
            "aws-inventory",
            "job",
            "submit",
            "--queue",
            queue.to_str().context("queue path")?,
            "--db",
            db.to_str().context("db path")?,
            "--job-id",
            "offline-test",
            "--service",
            "ec2",
            "--role-arn",
            "arn:aws:iam::123456789012:role/reader",
            "--external-id-env",
            "UNSET_SYNTHETIC_EXTERNAL_ID",
        ])?;
        let Command::Job { command } = cli.command else {
            anyhow::bail!("expected job command");
        };
        let submitted = execute(command).await?;
        assert_eq!(
            submitted.pointer("/job/state").and_then(Value::as_str),
            Some("queued")
        );
        assert!(!db.exists());
        let store = JobStore::open(&queue).await?;
        let job = store.get("offline-test").await?;
        assert_eq!(
            job.payload
                .as_ref()
                .and_then(|v| v.pointer("/accounts/external_id_env"))
                .and_then(Value::as_str),
            Some("UNSET_SYNTHETIC_EXTERNAL_ID")
        );
        assert_eq!(job.attempt, 0);
        store.close().await;
        tokio::fs::remove_dir_all(folder).await?;
        Ok(())
    }
}
