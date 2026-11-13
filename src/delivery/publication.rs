use super::*;

#[tracing::instrument(skip_all)]
pub(super) async fn write_outputs(
    store: &Store,
    scan_id: &str,
    options: &DeliveryArgs,
    directory: &Path,
    checkpoint: PathBuf,
    cancellation: &CancellationToken,
    outputs: &mut Value,
) -> Result<()> {
    let filename = match options.export_format {
        ExportFormat::Json => "inventory.json",
        ExportFormat::Jsonl => "inventory.jsonl",
    };
    let inventory_path = directory.join(filename);
    let summary = export::export_snapshot(
        store,
        scan_id,
        &inventory_path,
        export_options(
            options.export_format,
            options.provenance,
            options.allow_partial,
        ),
        cancellation,
    )
    .await?;
    outputs
        .as_object_mut()
        .context("outputs must be an object")?
        .insert(
            "inventory".into(),
            json!({"path":inventory_path,"summary":summary}),
        );
    if options.falkor {
        project(
            store,
            scan_id,
            options,
            directory,
            checkpoint,
            cancellation,
            outputs,
        )
        .await?;
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(super) async fn project(
    store: &Store,
    scan_id: &str,
    options: &DeliveryArgs,
    directory: &Path,
    checkpoint: PathBuf,
    cancellation: &CancellationToken,
    outputs: &mut Value,
) -> Result<()> {
    let plan = directory.join("graph.jsonl");
    let export = graph::export_snapshot_with_cancellation(
        store,
        scan_id,
        &options.graph,
        &plan,
        cancellation,
    )
    .await?;
    outputs
        .as_object_mut()
        .context("outputs must be an object")?
        .insert(
            "falkor".into(),
            json!({"plan":plan,"checkpoint":checkpoint,"export":export}),
        );
    let url = destination_url(options)?;
    let pushed = graph::push_file_with_options(
        plan.clone(),
        checkpoint.clone(),
        &url,
        graph::PushOptions {
            max_attempts: options.delivery_attempts,
            overall_timeout_ms: options.delivery_timeout_seconds.saturating_mul(1_000),
            ..graph::PushOptions::default()
        },
        cancellation,
    )
    .await?;
    outputs
        .get_mut("falkor")
        .and_then(Value::as_object_mut)
        .context("Falkor output missing")?
        .insert("push".into(), serde_json::to_value(pushed)?);
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(super) async fn persist_receipt(directory: &Path, value: &Value) -> Result<()> {
    let path = directory.join("delivery.json");
    let temporary = directory.join("delivery.json.partial");
    let bytes = serde_json::to_vec_pretty(value)?;
    ensure!(
        !tokio::fs::try_exists(&path).await?,
        "delivery receipt already exists"
    );
    let mut file = tokio::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .await?;
    let result = async {
        file.write_all(&bytes).await?;
        file.sync_all().await
    }
    .await;
    drop(file);
    // This private function publishes once inside a newly owned UUID directory.
    // Rename is the final operation, avoiding a post-publication unlink that
    // could disagree with the durable receipt's outcome.
    let result = match result {
        Ok(()) => tokio::fs::rename(&temporary, &path).await,
        Err(error) => Err(error),
    };
    if let Err(error) = result {
        if let Err(cleanup) = tokio::fs::remove_file(&temporary).await {
            anyhow::bail!(
                "receipt publication failed: {error}; temporary cleanup failed: {cleanup}"
            );
        }
        return Err(error.into());
    }
    Ok(())
}
