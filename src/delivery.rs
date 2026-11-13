//! Explicit, replayable outputs for immutable inventory snapshots.
//!
//! Each attempt owns a new directory. A shared SQLx checkpoint acknowledges
//! graph writes across attempts; failed delivery never restarts AWS collection.
use crate::cli::{DeliverArgs, DeliveryArgs, ExportArgs, ExportFormat, ExportProvenance};
use anyhow::{Context, Result, ensure};
use futures_util::{StreamExt, stream};
use inventory_core::ScanStatus;
use inventory_engine::{export, graph, store::Store};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

const MAX_ERROR_CHARS: usize = 1_024;
const CLEANUP_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug)]
struct DeliveryTimeout {
    drain_complete: bool,
    cleanup_complete: bool,
    cleanup_error: Option<String>,
}

impl std::fmt::Display for DeliveryTimeout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "delivery time budget exhausted; drain_complete={}, cleanup_complete={}; replay the finalized scan with deliver",
            self.drain_complete, self.cleanup_complete
        )
    }
}

impl std::error::Error for DeliveryTimeout {}

pub fn validate(options: &DeliveryArgs) -> Result<()> {
    ensure!(
        !options.falkor || options.output_dir.is_some(),
        "FalkorDB delivery requires --output-dir"
    );
    if let Some(path) = &options.output_dir {
        ensure!(!path.as_os_str().is_empty(), "empty output directory");
    }
    validate_name(&options.graph)?;
    ensure!(
        (1..=10).contains(&options.delivery_attempts),
        "delivery-attempts must be 1..10"
    );
    ensure!(
        (1..=86_400).contains(&options.delivery_timeout_seconds),
        "delivery-timeout-seconds must be 1..86400"
    );
    ensure!(
        !options.falkor_url_env.is_empty()
            && options.falkor_url_env.len() <= 128
            && options.falkor_url_env.bytes().enumerate().all(|(i, c)| {
                c == b'_' || c.is_ascii_alphabetic() || (i != 0 && c.is_ascii_digit())
            }),
        "invalid FalkorDB URL environment variable name"
    );
    Ok(())
}

fn validate_name(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty()
            && value.len() <= 128
            && value
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-'),
        "scan and graph names require 1..128 ASCII letters, numbers, underscores or hyphens"
    );
    Ok(())
}

/// Validate local output configuration before credentials or AWS APIs are used.
/// URL values are never serialized, traced or included in error messages.
#[tracing::instrument(skip_all)]
pub async fn preflight(options: &DeliveryArgs) -> Result<()> {
    validate(options)?;
    if let Some(path) = &options.output_dir {
        match tokio::fs::metadata(path).await {
            Ok(metadata) => ensure!(metadata.is_dir(), "output-dir is not a directory"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    if options.falkor {
        let url = destination_url(options)?;
        ensure!(
            url.starts_with("redis://") || url.starts_with("rediss://"),
            "FalkorDB URL must use redis:// or rediss://"
        );
    }
    Ok(())
}

fn destination_url(options: &DeliveryArgs) -> Result<String> {
    let url = std::env::var(&options.falkor_url_env)
        .with_context(|| format!("set {} to the FalkorDB Redis URL", options.falkor_url_env))?;
    ensure!(
        !url.is_empty() && url.len() <= 8_192,
        "invalid FalkorDB URL length"
    );
    Ok(url)
}

fn export_options(
    format: ExportFormat,
    provenance: ExportProvenance,
    allow_partial: bool,
) -> export::ExportOptions {
    export::ExportOptions {
        format: match format {
            ExportFormat::Json => export::ExportFormat::Json,
            ExportFormat::Jsonl => export::ExportFormat::Jsonl,
        },
        provenance: match provenance {
            ExportProvenance::Normalized => export::ProvenanceMode::Normalized,
            ExportProvenance::AllObservations => export::ProvenanceMode::AllObservations,
        },
        allow_partial,
        ..export::ExportOptions::default()
    }
}

#[tracing::instrument(skip_all)]
pub async fn export_command(options: ExportArgs) -> Result<Value> {
    crate::require_database(&options.db)?;
    let store = Store::open(&options.db).await?;
    let token = CancellationToken::new();
    let work = export::export_snapshot(
        &store,
        &options.scan_id,
        &options.output,
        export_options(options.format, options.provenance, options.allow_partial),
        &token,
    );
    let result = until_signal(work, &token).await;
    store.close().await;
    Ok(serde_json::to_value(result?)?)
}

#[tracing::instrument(skip_all)]
pub async fn command(options: DeliverArgs) -> Result<Value> {
    ensure!(
        options.delivery.output_dir.is_some(),
        "deliver requires --output-dir"
    );
    crate::require_database(&options.db)?;
    preflight(&options.delivery).await?;
    let token = CancellationToken::new();
    let work = deliver_one(&options.db, &options.scan_id, &options.delivery, &token);
    until_signal(work, &token).await
}

/// Own and drain the operation after a signal so publication and checkpoint
/// cleanup is observable. A drain deadline is explicit, never reported success.
#[tracing::instrument(skip_all)]
pub(crate) async fn until_signal<F, T, E>(work: F, token: &CancellationToken) -> Result<T>
where
    F: std::future::Future<Output = std::result::Result<T, E>>,
    E: Into<anyhow::Error>,
{
    tokio::pin!(work);
    tokio::select! {
        result = &mut work => result.map_err(Into::into),
        signal = tokio::signal::ctrl_c() => {
            token.cancel();
            if let Err(error) = signal { tracing::error!(%error, "shutdown signal failed"); }
            tokio::time::timeout(CLEANUP_TIMEOUT, work).await
                .context("delivery cleanup exceeded 30 seconds; incomplete files may require removal")?
                .map_err(Into::into)
        }
    }
}

/// Sequential delivery bounds open files, SQL readers and remote graph writes.
/// Collection and delivery statuses stay separate in the scan result.
#[tracing::instrument(skip_all)]
pub async fn after_scan(
    db: &Path,
    options: &DeliveryArgs,
    mut result: Value,
    cancellation: &CancellationToken,
) -> Result<Value> {
    if options.output_dir.is_none() {
        return Ok(result);
    }
    let scans = result
        .get("scans")
        .and_then(Value::as_array)
        .context("scan results missing")?;
    ensure!(
        scans.len() <= crate::targets::MAX_TARGET_ACCOUNTS,
        "too many delivery targets"
    );
    let ids: Vec<String> = scans
        .iter()
        .map(|scan| {
            scan.get("scan_id")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .context("scan ID missing")
        })
        .collect::<Result<_>>()?;
    let deliveries = stream::iter(ids)
        .then(|id| async move {
            match deliver_one(db, &id, options, cancellation).await {
                Ok(report) => report,
                Err(error) => json!({"scan_id":id,"status":"partial","delivery_status":"failed",
                    "error":error.to_string().chars().take(MAX_ERROR_CHARS).collect::<String>()}),
            }
        })
        .collect::<Vec<Value>>()
        .await;
    merge_delivery_result(&mut result, deliveries, cancellation.is_cancelled())?;
    Ok(result)
}

fn merge_delivery_result(
    result: &mut Value,
    deliveries: Vec<Value>,
    cancelled: bool,
) -> Result<()> {
    let object = result
        .as_object_mut()
        .context("scan result is not an object")?;
    let collection_status = object
        .get("status")
        .cloned()
        .context("scan status missing")?;
    let failure = deliveries
        .iter()
        .any(|item| item.get("delivery_status").and_then(Value::as_str) != Some("complete"));
    let overall = if cancelled {
        json!("cancelled")
    } else if failure && collection_status == json!("complete") {
        json!("partial")
    } else {
        collection_status.clone()
    };
    object.insert("collection_status".into(), collection_status);
    object.insert("status".into(), overall);
    object.insert("deliveries".into(), Value::Array(deliveries));
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn deliver_one(
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
async fn deliver_open_store(
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
    // One allocation per delivery cuts the nested SQLx/Redis future layout at
    // the orchestration boundary; rows retain streaming backpressure.
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
async fn create_attempt_directory(base: &Path, scan_id: &str) -> Result<PathBuf> {
    let parent = base.join(scan_id);
    tokio::fs::create_dir_all(&parent).await?;
    let directory = parent.join(uuid::Uuid::new_v4().to_string());
    tokio::fs::create_dir(&directory).await?;
    Ok(directory)
}

#[tracing::instrument(skip_all)]
async fn bounded_delivery<F, T>(
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

fn cleanup_failed(error: &anyhow::Error) -> bool {
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

fn delivery_receipt(
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

#[tracing::instrument(skip_all)]
async fn write_outputs(
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
async fn project(
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
async fn persist_receipt(directory: &Path, value: &Value) -> Result<()> {
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

#[cfg(test)]
mod tests;
