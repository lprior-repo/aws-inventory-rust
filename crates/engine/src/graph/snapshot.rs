//! Cancellation-safe durable graph snapshot publication.
use super::*;

/// Export an immutable scan through bounded SQLx streams and async file writes.
#[tracing::instrument(skip(store, output))]
pub async fn export_snapshot(
    store: &Store,
    scan_id: &str,
    graph: &str,
    output: &Path,
) -> Result<GraphExportSummary, GraphError> {
    export_snapshot_with_cancellation(store, scan_id, graph, output, &CancellationToken::new())
        .await
}

/// Cancel an owned export and remove its incomplete temporary file before returning.
#[tracing::instrument(skip(store, output, cancellation))]
pub async fn export_snapshot_with_cancellation(
    store: &Store,
    scan_id: &str,
    graph: &str,
    output: &Path,
    cancellation: &CancellationToken,
) -> Result<GraphExportSummary, GraphError> {
    if cancellation.is_cancelled() {
        return Err(GraphError::Cancelled);
    }
    validate_name(graph)?;
    validate_name(scan_id)?;
    let (status, summary) = tokio::select! {
        biased;
        () = cancellation.cancelled() => return Err(GraphError::Cancelled),
        result = snapshot_header(store, scan_id) => result?,
    };
    let mut temporary = output.as_os_str().to_owned();
    temporary.push(".partial");
    let temporary = PathBuf::from(temporary);
    // Establish ownership before selecting cancellation. In particular, never
    // remove a pre-existing .partial belonging to another invocation.
    let file = File::options()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .await?;
    let mut batch = BatchWriter {
        writer: BufWriter::with_capacity(65_536, file),
        graph: graph.into(),
        scan_id: summary.scan_id.clone(),
        sequence: 0,
        rows: Vec::with_capacity(MAX_BATCH_ROWS),
        bytes: 0,
    };
    let result = tokio::select! {
        biased;
        () = cancellation.cancelled() => Err(GraphError::Cancelled),
        result = write_snapshot(store, &summary, status, &mut batch) => result,
    };
    let result = if cancellation.is_cancelled() {
        Err(GraphError::Cancelled)
    } else {
        result
    };
    // Retain the writer outside the selected future: dropping a Tokio file
    // write future alone does not wait for its owned blocking file operation.
    let result = match tokio::time::timeout(Duration::from_secs(10), batch.writer.flush()).await {
        Ok(Ok(())) => result,
        Ok(Err(source)) => Err(GraphError::Cleanup { source }),
        Err(_) => Err(GraphError::Cleanup {
            source: std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "graph temporary writer drain deadline exceeded",
            ),
        }),
    };
    drop(batch);
    finish_export(result, &temporary, output).await
}

#[tracing::instrument(skip(store))]
pub(super) async fn snapshot_header(
    store: &Store,
    scan_id: &str,
) -> Result<(ScanStatus, ScanSummary), GraphError> {
    let status = store.scan_status(scan_id).await?;
    if status == ScanStatus::Running {
        return Err(GraphError::Invalid("cannot project a running scan".into()));
    }
    Ok((status, store.summary(scan_id).await?))
}

#[tracing::instrument(skip_all)]
pub(super) async fn write_snapshot(
    store: &Store,
    summary: &ScanSummary,
    status: ScanStatus,
    batch: &mut BatchWriter,
) -> Result<GraphExportSummary, GraphError> {
    begin_export(batch, summary).await?;
    let batch = export_rows(store, batch).await?;
    end_export(batch, status, summary).await
}

#[tracing::instrument(skip_all)]
pub(super) async fn begin_export(
    batch: &mut BatchWriter,
    summary: &ScanSummary,
) -> Result<(), GraphError> {
    batch.emit("index", INDEX_QUERY.into(), 0).await?;
    batch
        .emit(
            "begin",
            scan_marker(&summary.scan_id, "loading", summary)?,
            0,
        )
        .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(super) async fn export_rows<'a>(
    store: &Store,
    batch: &'a mut BatchWriter,
) -> Result<&'a mut BatchWriter, GraphError> {
    let batch = export_resources(store, batch).await?;
    batch.flush_rows("resources").await?;
    export_edges(store, batch).await
}

#[tracing::instrument(skip_all)]
pub(super) async fn export_resources<'a>(
    store: &Store,
    batch: &'a mut BatchWriter,
) -> Result<&'a mut BatchWriter, GraphError> {
    let scan = batch.scan_id.clone();
    store
        .resource_stream(&scan)
        .map_err(GraphError::Store)
        .try_fold(batch, |batch, resource| async move {
            batch
                .push(resource_row(&batch.scan_id, resource)?, "resources")
                .await?;
            Ok(batch)
        })
        .await
}

#[tracing::instrument(skip_all)]
pub(super) async fn export_edges<'a>(
    store: &Store,
    batch: &'a mut BatchWriter,
) -> Result<&'a mut BatchWriter, GraphError> {
    let scan = batch.scan_id.clone();
    store
        .edge_stream(&scan)
        .map_err(GraphError::Store)
        .try_fold(batch, |batch, edge| async move {
            batch.push(edge_row(&batch.scan_id, edge)?, "edges").await?;
            Ok(batch)
        })
        .await
}

#[tracing::instrument(skip_all)]
pub(super) async fn end_export(
    batch: &mut BatchWriter,
    status: ScanStatus,
    summary: &ScanSummary,
) -> Result<GraphExportSummary, GraphError> {
    batch.flush_rows("edges").await?;
    batch
        .emit(
            "end",
            scan_marker(&summary.scan_id, status.as_str(), summary)?,
            0,
        )
        .await?;
    sync_export(batch).await?;
    Ok(GraphExportSummary {
        graph: batch.graph.clone(),
        scan_id: batch.scan_id.clone(),
        batches: batch.sequence,
        resources: summary.resources,
        edges: summary.edges,
    })
}

#[tracing::instrument(skip_all)]
pub(super) async fn sync_export(batch: &mut BatchWriter) -> Result<(), GraphError> {
    batch.writer.flush().await?;
    batch.writer.get_ref().sync_all().await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(super) async fn finish_export(
    result: Result<GraphExportSummary, GraphError>,
    temporary: &Path,
    output: &Path,
) -> Result<GraphExportSummary, GraphError> {
    let installed = match result {
        Ok(summary) => {
            // Atomic no-clobber installation on the same filesystem. In
            // particular, output must never replace the live inventory DB.
            tokio::fs::hard_link(temporary, output)
                .await
                .map(|()| summary)
                .map_err(GraphError::Io)
        }
        Err(error) => Err(error),
    };
    let cleanup = tokio::fs::remove_file(temporary).await;
    match cleanup {
        Ok(()) => installed,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => installed,
        Err(source) => Err(GraphError::Cleanup { source }),
    }
}

pub(super) fn scan_marker(
    scan: &str,
    status: &str,
    summary: &ScanSummary,
) -> Result<String, GraphError> {
    Ok(format!(
        "CYPHER scan={} state={} resources={} edges={} {MARKER_QUERY}",
        literal(scan)?,
        literal(status)?,
        summary.resources,
        summary.edges
    ))
}
