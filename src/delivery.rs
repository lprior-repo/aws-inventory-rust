//! Explicit, replayable outputs for immutable inventory snapshots.
//!
//! Each attempt owns a new directory. A shared SQLx checkpoint acknowledges
//! graph writes across attempts; failed delivery never restarts AWS collection.
use crate::cli::{DeliverArgs, DeliveryArgs, ExportArgs, ExportFormat, ExportProvenance};
use anyhow::{Context, Result, ensure};
#[cfg(any(feature = "aws", test))]
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
    reference_mode: export::ReferenceMode,
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
        reference_mode,
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
        export_options(
            options.format,
            options.provenance,
            options.allow_partial,
            options.reference_mode(),
        ),
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
#[cfg(any(feature = "aws", test))]
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

#[cfg(any(feature = "aws", test))]
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

#[cfg(test)]
mod tests;

mod publication;
use publication::persist_receipt;
use publication::write_outputs;

mod attempt;
#[cfg(test)]
use attempt::bounded_delivery;
use attempt::deliver_one;
#[cfg(test)]
use attempt::delivery_receipt;
