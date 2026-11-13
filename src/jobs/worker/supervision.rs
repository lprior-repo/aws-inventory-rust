use super::*;

#[tracing::instrument(skip_all)]
pub(super) async fn supervise<F>(
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
pub(super) async fn supervise_futures<F, H>(
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

pub(super) fn drain_deadline(drain: Duration) -> Result<tokio::time::Instant> {
    tokio::time::Instant::now()
        .checked_add(drain)
        .context("scan drain deadline overflow")
}

#[tracing::instrument(skip_all)]
pub(super) async fn drain_with_heartbeats<F, H>(
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
pub(super) async fn drain_until<F>(scan: F, deadline: tokio::time::Instant) -> Result<Value>
where
    F: std::future::Future<Output = Result<Value>>,
{
    tokio::time::timeout_at(deadline, scan).await
        .map_err(|_| anyhow::anyhow!("cancelled scan exceeded its drain deadline; queued or in-flight inventory finalization may be incomplete"))?
}

pub(super) fn lease_and_drain(lease: Result<()>, drained: Result<Value>) -> Result<Value> {
    match (lease, drained) {
        (Err(lease), Err(drain)) => Err(anyhow::anyhow!(
            "lease supervision failed: {lease:#}; {drain:#}"
        )),
        (Err(lease), Ok(_)) => Err(lease.context("lease supervision failed; scan drained")),
        (Ok(()), Err(drain)) => {
            Err(drain
                .context("lease heartbeat loop ended unexpectedly; scan cancellation requested"))
        }
        (Ok(()), Ok(_)) => Err(anyhow::anyhow!(
            "lease heartbeat loop ended unexpectedly; scan cancellation requested"
        )),
    }
}

#[tracing::instrument(skip_all)]
pub(super) async fn heartbeats(
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
