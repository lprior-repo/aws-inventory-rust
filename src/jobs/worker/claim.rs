use super::*;

#[tracing::instrument(skip_all, fields(job_id=%lease.job.job_id, attempt=lease.job.attempt))]
pub(super) async fn run_claim(
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
            state: completion_state(&value, Cancellation::from(executor.is_cancelled())),
            result: Some(compact_result(value)),
            error: None,
        },
        Err(error) => Completion {
            state: if executor.is_cancelled() {
                TerminalJobState::Cancelled
            } else {
                TerminalJobState::Failed
            },
            result: None,
            error: Some(format!("{error:#}")),
        },
    };
    let done = store.complete(&lease, completion, demo::now_ms()?).await?;
    Ok(job_summary(&done))
}

#[tracing::instrument(skip_all)]
pub(super) async fn prepare_claim(lease: &Lease) -> Result<(ScanArgs, Arc<ApiExecutor>)> {
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
pub(super) async fn terminal_error(
    store: &JobStore,
    lease: &Lease,
    error: anyhow::Error,
) -> Result<Value> {
    let done = store
        .complete(
            lease,
            Completion {
                state: TerminalJobState::Failed,
                result: None,
                error: Some(format!("{error:#}")),
            },
            demo::now_ms()?,
        )
        .await?;
    Ok(job_summary(&done))
}

#[tracing::instrument(skip_all)]
pub(super) async fn cancel_before_scan(store: &JobStore, lease: &Lease) -> Result<Value> {
    let completion = Completion {
        state: TerminalJobState::Cancelled,
        result: None,
        error: Some("worker stopped before AWS scan admission".into()),
    };
    Ok(job_summary(
        &store.complete(lease, completion, demo::now_ms()?).await?,
    ))
}
