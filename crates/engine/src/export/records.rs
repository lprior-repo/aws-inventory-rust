//! SQLx streams and typed provenance; never materialize a complete snapshot.

use super::format::ArtifactWriter;
use super::{
    ExportEdge, ExportError, ExportHeader, ExportRecord, ExportResource, ExportScope,
    MAX_EXPORT_RECORDS, ObservationProvenance, ProvenanceMode, check_cancel,
};
use crate::store::{MAX_RECORD_BYTES, Store};
use futures_util::{Stream, StreamExt, TryStreamExt};
use inventory_core::{
    CollectionFailure, CollectionScope, Edge, MAX_PAGES, Resource, validate_identifier,
};
use serde::de::DeserializeOwned;
use sqlx::Row;
use sqlx::sqlite::SqliteRow;
use std::pin::Pin;
use tokio::io::AsyncWrite;
use tokio_util::sync::CancellationToken;

const MAX_SCOPE_BYTES: usize = 131_072;
const MAX_FAILURE_BYTES: usize = 16_384;
type RecordStream<'a> = Pin<Box<dyn Stream<Item = Result<ExportRecord, ExportError>> + Send + 'a>>;

const RESOURCE_OBSERVATIONS: &str = "SELECT scope_key,page_number,resource_json AS payload
    FROM resource_observations WHERE scan_id=?1
    ORDER BY resource_id,scope_key,page_number";
const EDGE_OBSERVATIONS: &str = "SELECT scope_key,page_number,edge_json AS payload
    FROM edge_observations WHERE scan_id=?1
    ORDER BY from_id,to_id,relation,scope_key,page_number";
const SCOPES: &str = "SELECT scope_key,last_page,complete,failed
    FROM scope_checkpoints WHERE scan_id=?1 ORDER BY scope_key";
const FAILURES: &str = "SELECT scope_key,category,message
    FROM collection_failures WHERE scan_id=?1 ORDER BY scope_key,category,message";

#[tracing::instrument(skip_all)]
pub(super) async fn write_records<W: AsyncWrite + Unpin>(
    store: &Store,
    writer: &mut ArtifactWriter<W>,
    header: &ExportHeader,
    cancellation: &CancellationToken,
) -> Result<(), ExportError> {
    writer.begin(header, cancellation).await?;
    let scan = &header.scan.summary.scan_id;
    let records = resource_records(store, scan, header.provenance)
        .chain(edge_records(store, scan, header.provenance))
        .chain(sql_records(store, scan, SCOPES, scope_record))
        .chain(sql_records(store, scan, FAILURES, failure_record));
    records
        .take(MAX_EXPORT_RECORDS.saturating_add(1))
        .take_until(cancellation.cancelled())
        .try_fold(&mut *writer, |writer, record| async move {
            writer.record(record, cancellation).await?;
            Ok::<_, ExportError>(writer)
        })
        .await?;
    check_cancel(cancellation)?;
    writer.finish(header, cancellation).await
}

fn resource_records<'a>(
    store: &'a Store,
    scan_id: &'a str,
    provenance: ProvenanceMode,
) -> RecordStream<'a> {
    match provenance {
        ProvenanceMode::Normalized => Box::pin(
            store
                .resource_stream(scan_id)
                .map_err(ExportError::Store)
                .map_ok(|resource| {
                    ExportRecord::Resource(ExportResource {
                        resource,
                        observation: None,
                    })
                }),
        ),
        ProvenanceMode::AllObservations => Box::pin(sql_records(
            store,
            scan_id,
            RESOURCE_OBSERVATIONS,
            resource_observation,
        )),
    }
}

fn edge_records<'a>(
    store: &'a Store,
    scan_id: &'a str,
    provenance: ProvenanceMode,
) -> RecordStream<'a> {
    match provenance {
        ProvenanceMode::Normalized => Box::pin(
            store
                .edge_stream(scan_id)
                .map_err(ExportError::Store)
                .map_ok(|edge| {
                    ExportRecord::Edge(ExportEdge {
                        edge,
                        observation: None,
                    })
                }),
        ),
        ProvenanceMode::AllObservations => Box::pin(sql_records(
            store,
            scan_id,
            EDGE_OBSERVATIONS,
            edge_observation,
        )),
    }
}

fn sql_records<'a, F>(
    store: &'a Store,
    scan_id: &'a str,
    query: &'static str,
    mut decode: F,
) -> impl Stream<Item = Result<ExportRecord, ExportError>> + Send + 'a
where
    F: FnMut(SqliteRow) -> Result<ExportRecord, ExportError> + Send + 'a,
{
    sqlx::query(query)
        .bind(scan_id)
        .fetch(store.export_reader())
        .take(MAX_EXPORT_RECORDS.saturating_add(1))
        .map(move |row| row.map_err(ExportError::Sql).and_then(&mut decode))
}

fn resource_observation(row: SqliteRow) -> Result<ExportRecord, ExportError> {
    let observation = observation(&row)?;
    let resource: Resource = decode_json(&row, "payload", MAX_RECORD_BYTES)?;
    resource.validate()?;
    if resource.source != observation.scope.source {
        return Err(ExportError::Invalid(
            "resource source differs from observation scope",
        ));
    }
    Ok(ExportRecord::Resource(ExportResource {
        resource,
        observation: Some(observation),
    }))
}

fn edge_observation(row: SqliteRow) -> Result<ExportRecord, ExportError> {
    let observation = observation(&row)?;
    let edge: Edge = decode_json(&row, "payload", MAX_RECORD_BYTES)?;
    edge.validate()?;
    if edge.source != observation.scope.source {
        return Err(ExportError::Invalid(
            "edge source differs from observation scope",
        ));
    }
    Ok(ExportRecord::Edge(ExportEdge {
        edge,
        observation: Some(observation),
    }))
}

fn observation(row: &SqliteRow) -> Result<ObservationProvenance, ExportError> {
    let page_number: u32 = row.try_get("page_number")?;
    if !(1..=MAX_PAGES).contains(&page_number) {
        return Err(ExportError::Invalid("stored observation page number"));
    }
    Ok(ObservationProvenance {
        scope: scope(row)?,
        page_number,
    })
}

fn scope(row: &SqliteRow) -> Result<CollectionScope, ExportError> {
    let scope: CollectionScope = decode_json(row, "scope_key", MAX_SCOPE_BYTES)?;
    scope.validate()?;
    Ok(scope)
}

fn scope_record(row: SqliteRow) -> Result<ExportRecord, ExportError> {
    let last_page: u32 = row.try_get("last_page")?;
    if last_page > MAX_PAGES {
        return Err(ExportError::Invalid("stored scope page count"));
    }
    Ok(ExportRecord::Scope(ExportScope {
        scope: scope(&row)?,
        last_page,
        complete: row.try_get("complete")?,
        failed: row.try_get("failed")?,
    }))
}

fn failure_record(row: SqliteRow) -> Result<ExportRecord, ExportError> {
    let category: &str = row.try_get("category")?;
    validate_identifier(category, "failure category")?;
    let message: &str = row.try_get("message")?;
    if message.len() > MAX_FAILURE_BYTES {
        return Err(ExportError::Limit("stored failure message bytes"));
    }
    Ok(ExportRecord::Failure(CollectionFailure {
        scope: scope(&row)?,
        category: category.to_owned(),
        message: message.to_owned(),
    }))
}

fn decode_json<T: DeserializeOwned>(
    row: &SqliteRow,
    column: &str,
    limit: usize,
) -> Result<T, ExportError> {
    let json: &str = row.try_get(column)?;
    if json.len() > limit {
        return Err(ExportError::Limit("stored record bytes"));
    }
    Ok(serde_json::from_str(json)?)
}
