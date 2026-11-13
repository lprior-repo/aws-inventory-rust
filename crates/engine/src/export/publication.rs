//! Cancellation-safe inventory export publication and cleanup.
use super::*;

/// Stream an immutable scan, then atomically install a new output file.
///
/// Complete is required unless allow_partial explicitly permits Partial or
/// Cancelled. Running is always rejected. Existing output or `.partial` paths
/// are never overwritten. Cooperative cancellation removes only the partial
/// file created by this call. Once the final no-clobber link begins, publication
/// wins a concurrent cancellation; callers must retain the successful result.
#[tracing::instrument(skip(store, output, cancellation), fields(scan_id))]
pub async fn export_snapshot(
    store: &Store,
    scan_id: &str,
    output: &Path,
    options: ExportOptions,
    cancellation: &CancellationToken,
) -> Result<ExportSummary, ExportError> {
    options.validate()?;
    validate_identifier(scan_id, "scan id")?;
    check_cancel(cancellation)?;
    let temporary = partial_path(output)?;
    reject_existing(output).await?;
    let header = snapshot_header(store, scan_id, options, cancellation).await?;
    check_cancel(cancellation)?;
    let file = create_partial(&temporary).await?;
    let result = write_partial(store, file, &header, options, cancellation).await;
    publish_or_discard(result, &temporary, output, cancellation).await
}

pub(super) fn partial_path(output: &Path) -> Result<PathBuf, ExportError> {
    if output.file_name().is_none() {
        return Err(ExportError::Invalid("output must name a file"));
    }
    let mut path = output.as_os_str().to_owned();
    path.push(".partial");
    Ok(PathBuf::from(path))
}

#[tracing::instrument(skip_all)]
pub(super) async fn reject_existing(output: &Path) -> Result<(), ExportError> {
    match tokio::fs::symlink_metadata(output).await {
        Ok(_) => Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "export output already exists",
        )
        .into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[tracing::instrument(skip_all)]
pub(super) async fn create_partial(temporary: &Path) -> Result<File, ExportError> {
    let mut options = File::options();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    Ok(options.open(temporary).await?)
}

#[tracing::instrument(skip_all)]
pub(super) async fn snapshot_header(
    store: &Store,
    scan_id: &str,
    options: ExportOptions,
    cancellation: &CancellationToken,
) -> Result<ExportHeader, ExportError> {
    let status = await_read(cancellation, store.scan_status(scan_id)).await?;
    match status {
        ScanStatus::Running => return Err(ExportError::Invalid("cannot export a running scan")),
        ScanStatus::Partial | ScanStatus::Cancelled if !options.allow_partial => {
            return Err(ExportError::Invalid(
                "partial or cancelled scan requires allow_partial",
            ));
        }
        _ => {}
    }
    let summary = await_read(cancellation, store.summary(scan_id)).await?;
    let row = await_read(
        cancellation,
        sqlx::query("SELECT started_at,finished_at FROM scans WHERE scan_id=?1")
            .bind(scan_id)
            .fetch_one(store.export_reader()),
    )
    .await?;
    Ok(ExportHeader {
        artifact: EXPORT_ARTIFACT.into(),
        version: EXPORT_VERSION,
        provenance: options.provenance,
        scan: ExportScan {
            summary,
            status,
            started_at: row.try_get("started_at")?,
            finished_at: row.try_get("finished_at")?,
        },
    })
}

#[tracing::instrument(skip_all)]
pub(super) async fn write_partial(
    store: &Store,
    file: File,
    header: &ExportHeader,
    options: ExportOptions,
    cancellation: &CancellationToken,
) -> Result<ExportSummary, ExportError> {
    let mut writer =
        ArtifactWriter::new(BufWriter::with_capacity(FILE_BUFFER_BYTES, file), options);
    let written = records::write_records(store, &mut writer, header, cancellation).await;
    // Drain outstanding file writes even after cancellation or a record error.
    let drained = writer.sink.flush().await;
    let result = combine_drain(written, drained);
    result?;
    check_cancel(cancellation)?;
    writer.sink.get_ref().sync_all().await?;
    check_cancel(cancellation)?;
    Ok(writer.summary(header))
}

pub(super) fn combine_drain(
    result: Result<(), ExportError>,
    drain: Result<(), std::io::Error>,
) -> Result<(), ExportError> {
    match (result, drain) {
        (Ok(()), Ok(())) => Ok(()),
        (Ok(()), Err(error)) => Err(error.into()),
        (Err(error), Ok(())) => Err(error),
        (Err(cause), Err(drain)) => Err(ExportError::Cleanup {
            cause: Box::new(cause),
            drain: Some(drain),
            removal: None,
        }),
    }
}

#[tracing::instrument(skip_all)]
pub(super) async fn publish_or_discard(
    result: Result<ExportSummary, ExportError>,
    temporary: &Path,
    output: &Path,
    cancellation: &CancellationToken,
) -> Result<ExportSummary, ExportError> {
    let result = result.and_then(|summary| {
        check_cancel(cancellation)?;
        Ok(summary)
    });
    let summary = match result {
        Ok(summary) => summary,
        Err(cause) => return Err(discard_partial(temporary, cause).await),
    };
    // This commit is deliberately not raced against cancellation. Same-directory
    // hard-link installation is atomic and cannot overwrite an existing target.
    if let Err(error) = tokio::fs::hard_link(temporary, output).await {
        return Err(discard_partial(temporary, error.into()).await);
    }
    tokio::fs::remove_file(temporary)
        .await
        .map_err(|source| ExportError::PublishedCleanup {
            output: output.to_path_buf(),
            source,
        })?;
    Ok(summary)
}

#[tracing::instrument(skip_all)]
pub(super) async fn discard_partial(temporary: &Path, cause: ExportError) -> ExportError {
    match tokio::fs::remove_file(temporary).await {
        Ok(()) => cause,
        Err(removal) => ExportError::Cleanup {
            cause: Box::new(cause),
            drain: None,
            removal: Some(removal),
        },
    }
}

pub(super) fn check_cancel(cancellation: &CancellationToken) -> Result<(), ExportError> {
    if cancellation.is_cancelled() {
        Err(ExportError::Cancelled)
    } else {
        Ok(())
    }
}

#[tracing::instrument(skip_all)]
pub(super) async fn await_read<T, E: Into<ExportError>>(
    cancellation: &CancellationToken,
    read: impl Future<Output = Result<T, E>>,
) -> Result<T, ExportError> {
    tokio::select! {
        biased;
        () = cancellation.cancelled() => Err(ExportError::Cancelled),
        result = read => result.map_err(Into::into),
    }
}
