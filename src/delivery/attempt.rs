use super::*;

#[tracing::instrument(skip_all)]
pub(super) async fn deliver_one(
    db: &Path,
    scan_id: &str,
    options: &DeliveryArgs,
    cancellation: &CancellationToken,
) -> Result<Value> {
    validate(options)?;
    validate_name(scan_id)?;
    if cancellation.is_cancelled() {
        return Ok(json!({"scan_id":scan_id,"status":"cancelled","delivery_status":"cancelled"}));
    }
    let store = Store::open(db).await?;
    let result = deliver_open_store(&store, scan_id, options, cancellation).await;
    store.close().await;
    result
}

#[tracing::instrument(skip_all)]
pub(super) async fn deliver_open_store(
    store: &Store,
    scan_id: &str,
    options: &DeliveryArgs,
    cancellation: &CancellationToken,
) -> Result<Value> {
    let status = store.scan_status(scan_id).await?;
    ensure!(
        status != ScanStatus::Running,
        "cannot deliver a running scan"
    );
    if status != ScanStatus::Complete && !options.allow_partial {
        return Ok(
            json!({"scan_id":scan_id,"status":status.as_str(),"delivery_status":"skipped",
            "reason":"scan is not complete; use --allow-partial to preserve its actual status in exported outputs"}),
        );
    }
    let base = options
        .output_dir
        .as_ref()
        .context("delivery requires output-dir")?;
    let directory = create_attempt_directory(base, scan_id).await?;
    let checkpoint = base.join("projection.sqlite");
    let local_cancel = cancellation.child_token();
    let mut outputs = json!({});

    let work: futures_util::future::BoxFuture<'_, Result<()>> = Box::pin(write_outputs(
        store,
        scan_id,
        options,
        &directory,
        checkpoint,
        &local_cancel,
        &mut outputs,
    ));
    let result = bounded_delivery(work, &local_cancel, options.delivery_timeout_seconds).await;
    let mut receipt = delivery_receipt(
        scan_id,
        status,
        &directory,
        result,
        outputs,
        cancellation.is_cancelled(),
    );
    if let Err(error) = persist_receipt(&directory, &receipt).await {
        let object = receipt
            .as_object_mut()
            .context("delivery receipt is not an object")?;
        object.insert("status".into(), json!("partial"));
        object.insert("delivery_status".into(), json!("failed"));
        object.insert("receipt_error".into(), json!(error.to_string()));
    }
    Ok(receipt)
}

#[tracing::instrument(skip_all)]
pub(super) async fn create_attempt_directory(base: &Path, scan_id: &str) -> Result<PathBuf> {
    let parent = base.join(scan_id);
    tokio::fs::create_dir_all(&parent).await?;
    let directory = parent.join(uuid::Uuid::new_v4().to_string());
    tokio::fs::create_dir(&directory).await?;
    Ok(directory)
}

#[tracing::instrument(skip_all)]
pub(super) async fn bounded_delivery<F, T>(
    work: F,
    cancellation: &CancellationToken,
    seconds: u64,
) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    tokio::pin!(work);
    tokio::select! {
        result = &mut work => result,
        () = tokio::time::sleep(Duration::from_secs(seconds)) => {
            cancellation.cancel();
            match tokio::time::timeout(CLEANUP_TIMEOUT, work).await {
                Ok(result) => {
                    let cleanup_complete = result.as_ref().err().is_none_or(|error| !cleanup_failed(error));
                    let cleanup_error = result.err().map(|error| format!("{error:#}").chars().take(MAX_ERROR_CHARS).collect());
                    Err(DeliveryTimeout { drain_complete: true, cleanup_complete, cleanup_error }.into())
                }
                Err(_) => Err(DeliveryTimeout { drain_complete: false, cleanup_complete: false, cleanup_error: Some("cleanup exceeded 30 seconds; incomplete files may remain".into()) }.into()),
            }
        }
    }
}

pub(super) fn cleanup_failed(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<export::ExportError>()
            .is_some_and(|error| {
                matches!(
                    error,
                    export::ExportError::Cleanup { .. }
                        | export::ExportError::PublishedCleanup { .. }
                )
            })
            || cause
                .downcast_ref::<graph::GraphError>()
                .is_some_and(|error| matches!(error, graph::GraphError::Cleanup { .. }))
    })
}

pub(super) fn delivery_receipt(
    scan_id: &str,
    status: ScanStatus,
    directory: &Path,
    result: Result<()>,
    outputs: Value,
    cancelled: bool,
) -> Value {
    match result {
        Ok(()) => json!({"version":1,"scan_id":scan_id,"status":status.as_str(),
            "collection_status":status.as_str(),"delivery_status":"complete","directory":directory,"outputs":outputs}),
        Err(error) => {
            let deadline = error.downcast_ref::<DeliveryTimeout>();
            json!({"version":1,"scan_id":scan_id,"status":if cancelled {"cancelled"} else {"partial"},
                "collection_status":status.as_str(),"delivery_status":if cancelled {"cancelled"} else if deadline.is_some() {"timed_out"} else {"failed"},
                "cleanup_complete":deadline.map(|timeout| timeout.cleanup_complete),
                "drain_complete":deadline.map(|timeout| timeout.drain_complete),
                "cleanup_error":deadline.and_then(|timeout| timeout.cleanup_error.as_ref()),"outputs":outputs,
                "directory":directory,"error":error.to_string().chars().take(MAX_ERROR_CHARS).collect::<String>(),
                "recovery":"Run deliver against this scan ID. AWS collection is already persisted; graph ACKs resume from projection.sqlite."})
        }
    }
}
