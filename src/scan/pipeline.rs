use super::*;

#[tracing::instrument(skip_all)]
pub(super) async fn run_open(
    store: &Store,
    jobs: Vec<Job>,
    executor: Arc<ApiExecutor>,
) -> Result<Value> {
    initialize_scans(store, &jobs).await?;
    let result = pipeline(store, jobs, Arc::clone(&executor)).await;
    finalize_scans(store, result, executor).await
}

#[tracing::instrument(skip_all)]
pub(super) async fn initialize_scans(store: &Store, jobs: &[Job]) -> Result<()> {
    let time = demo::now_ms()?;
    stream::iter(jobs.iter().map(Ok::<_, anyhow::Error>))
        .try_for_each(|job| async move {
            store.begin_scan(&job.scan_id, time).await?;
            Ok(())
        })
        .await
}

#[tracing::instrument(skip_all)]
pub(super) async fn pipeline(
    store: &Store,
    jobs: Vec<Job>,
    executor: Arc<ApiExecutor>,
) -> PipelineResult {
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
pub(super) async fn supervise(
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
pub(super) async fn collect_jobs(
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
pub(super) async fn collect_job(
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
    let selection = job.options.selection.clone();
    if executor.is_cancelled() {
        return Outcome {
            scan_id: job.scan_id,
            profile: job.options.profile,
            target_account,
            error: Some("collection cancelled before credential loading".into()),
            selection,
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
        selection,
    }
}

#[tracing::instrument(skip_all)]
pub(super) async fn forward_events(
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

pub(super) fn startup_failure(
    scan_id: &str,
    target_account: Option<&str>,
    message: &str,
) -> RoutedEvent {
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
pub(super) async fn write_events(
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
pub(super) async fn finalize_scans(
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
pub(super) async fn finish_profile(
    store: &Store,
    outcome: Outcome,
    requested: ScanStatus,
) -> Result<Value> {
    let status = if outcome.error.is_some() && requested == ScanStatus::Complete {
        ScanStatus::Partial
    } else {
        requested
    };
    let universal = outcome.selection.is_universal();
    let scan_id = outcome.scan_id.clone();
    let summary = store
        .finish_scan(&outcome.scan_id, status, demo::now_ms()?)
        .await?;
    let actual = store.scan_status(&outcome.scan_id).await?;
    let mut scan = json!({"scan_id":outcome.scan_id,"profile":outcome.profile,"target_account":outcome.target_account,"status":actual.as_str(),"summary":summary,"error":outcome.error});
    if universal {
        let catalog = inventory_aws::cloud_control::universal_report()?;
        let outcomes = store.universal_outcomes(&scan_id).await?;
        let object = scan
            .as_object_mut()
            .context("scan result must be an object")?;
        object.insert(
            "universal".into(),
            json!({
                "catalog_digest": catalog.catalog_digest,
                "total": catalog.total,
                "list_capable": catalog.list_capable,
                "attempted": outcomes.attempted,
                "succeeded": outcomes.succeeded,
                "unsupported": outcomes.unsupported,
                "failed": outcomes.failed,
                "skipped": catalog.parent_required.saturating_add(catalog.unsupported),
                "skipped_parent_required": catalog.parent_required,
                "skipped_unsupported": catalog.unsupported,
                "skipped_parent_required_types": catalog.skipped_parent_required,
                "skipped_unsupported_types": catalog.skipped_unsupported,
            }),
        );
    }
    Ok(scan)
}
