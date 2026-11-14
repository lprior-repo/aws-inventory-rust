use super::{
    ExportCounts, ExportError, ExportFormat, ExportHeader, ExportOptions, ExportRecord,
    ExportSummary, MAX_EXPORT_RECORD_BYTES, ProvenanceMode, check_cancel,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::{self, Write};
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio_util::sync::CancellationToken;

pub(super) struct ArtifactWriter<W> {
    pub(super) sink: W,
    options: ExportOptions,
    buffer: EncodeBuffer,
    counts: ExportCounts,
    bytes: u64,
    digest: Sha256,
    completed_scopes: u64,
    failed_scopes: u64,
}

#[derive(Default)]
struct EncodeBuffer {
    bytes: Vec<u8>,
    exceeded: bool,
}

impl Write for EncodeBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let total = self.bytes.len().checked_add(bytes.len());
        let Some(total) = total.filter(|total| *total <= MAX_EXPORT_RECORD_BYTES) else {
            self.exceeded = true;
            return Err(io::Error::other("export record byte limit"));
        };
        if total > self.bytes.capacity() {
            let capacity = total
                .max(self.bytes.capacity().saturating_mul(2))
                .clamp(8192, MAX_EXPORT_RECORD_BYTES);
            self.bytes
                .try_reserve_exact(capacity.saturating_sub(self.bytes.len()))
                .map_err(io::Error::other)?;
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl EncodeBuffer {
    fn encode(&mut self, value: &impl Serialize) -> Result<(), ExportError> {
        self.bytes.clear();
        self.exceeded = false;
        let result = serde_json::to_writer(&mut *self, value);
        if self.exceeded {
            return Err(ExportError::Limit("serialized record bytes"));
        }
        Ok(result?)
    }
}

#[derive(Serialize)]
struct Envelope<'a, T> {
    record: &'static str,
    data: &'a T,
}

impl<W: AsyncWrite + Unpin> ArtifactWriter<W> {
    pub(super) fn new(sink: W, options: ExportOptions) -> Self {
        Self {
            sink,
            options,
            buffer: EncodeBuffer::default(),
            counts: ExportCounts::default(),
            bytes: 0,
            digest: Sha256::new(),
            completed_scopes: 0,
            failed_scopes: 0,
        }
    }

    #[tracing::instrument(skip_all)]
    pub(super) async fn begin(
        &mut self,
        header: &ExportHeader,
        cancellation: &CancellationToken,
    ) -> Result<(), ExportError> {
        check_cancel(cancellation)?;
        match self.options.format {
            ExportFormat::Json => {
                self.raw(b"{\"header\":").await?;
                self.json(header).await?;
                self.raw(b",\"records\":[").await
            }
            ExportFormat::Jsonl => {
                self.json(&Envelope {
                    record: "header",
                    data: header,
                })
                .await?;
                self.raw(b"\n").await
            }
        }
    }

    #[tracing::instrument(skip_all)]
    pub(super) async fn record(
        &mut self,
        record: ExportRecord,
        cancellation: &CancellationToken,
    ) -> Result<(), ExportError> {
        check_cancel(cancellation)?;
        let counts = next_counts(self.counts, &record, self.options.max_records)?;
        self.buffer.encode(&record)?;
        if self.options.format == ExportFormat::Json && self.counts.records != 0 {
            self.raw(b",").await?;
        }
        self.encoded().await?;
        if self.options.format == ExportFormat::Jsonl {
            self.raw(b"\n").await?;
        }
        self.track_coverage(&record)?;
        self.counts = counts;
        tokio::task::coop::consume_budget().await;
        check_cancel(cancellation)
    }

    fn track_coverage(&mut self, record: &ExportRecord) -> Result<(), ExportError> {
        if let ExportRecord::Scope(scope) = record {
            if scope.failed {
                increment(&mut self.failed_scopes)?;
            } else if scope.complete {
                increment(&mut self.completed_scopes)?;
            }
        }
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub(super) async fn finish(
        &mut self,
        header: &ExportHeader,
        cancellation: &CancellationToken,
    ) -> Result<(), ExportError> {
        check_cancel(cancellation)?;
        self.verify_counts(header)?;
        let counts = self.counts;
        match self.options.format {
            ExportFormat::Json => {
                self.raw(b"],\"summary\":").await?;
                self.json(&counts).await?;
                self.raw(b"}\n").await
            }
            ExportFormat::Jsonl => {
                self.json(&Envelope {
                    record: "summary",
                    data: &counts,
                })
                .await?;
                self.raw(b"\n").await
            }
        }
    }

    fn verify_counts(&self, header: &ExportHeader) -> Result<(), ExportError> {
        let expected = &header.scan.summary;
        let records_match = match self.options.provenance {
            ProvenanceMode::Normalized => {
                self.counts.resources == expected.resources && self.counts.edges == expected.edges
            }
            ProvenanceMode::AllObservations => {
                self.counts.resources >= expected.resources && self.counts.edges >= expected.edges
            }
        };
        if !records_match
            || self.completed_scopes != expected.completed_scopes
            || self.failed_scopes != expected.failed_scopes
            || self.counts.failures < self.failed_scopes
        {
            return Err(ExportError::Invalid(
                "export counts disagree with finalized scan",
            ));
        }
        Ok(())
    }

    pub(super) fn summary(&self, header: &ExportHeader) -> ExportSummary {
        ExportSummary {
            scan_id: header.scan.summary.scan_id.clone(),
            status: header.scan.status,
            format: self.options.format,
            provenance: self.options.provenance,
            counts: self.counts,
            bytes: self.bytes,
            sha256: format!(
                "{:x}",
                base16ct::HexDisplay(&self.digest.clone().finalize())
            ),
        }
    }

    #[tracing::instrument(skip_all)]
    async fn json(&mut self, value: &impl Serialize) -> Result<(), ExportError> {
        self.buffer.encode(value)?;
        self.encoded().await
    }

    #[tracing::instrument(skip_all)]
    async fn encoded(&mut self) -> Result<(), ExportError> {
        write_chunk(
            &mut self.sink,
            &mut self.digest,
            &mut self.bytes,
            self.options.max_bytes,
            &self.buffer.bytes,
        )
        .await
    }

    #[tracing::instrument(skip_all)]
    async fn raw(&mut self, bytes: &[u8]) -> Result<(), ExportError> {
        write_chunk(
            &mut self.sink,
            &mut self.digest,
            &mut self.bytes,
            self.options.max_bytes,
            bytes,
        )
        .await
    }
}

fn increment(value: &mut u64) -> Result<(), ExportError> {
    *value = value
        .checked_add(1)
        .ok_or(ExportError::Limit("record count overflow"))?;
    Ok(())
}

fn next_counts(
    mut counts: ExportCounts,
    record: &ExportRecord,
    max_records: usize,
) -> Result<ExportCounts, ExportError> {
    let limit = u64::try_from(max_records).map_err(|_| ExportError::Limit("record count"))?;
    if counts.records >= limit {
        return Err(ExportError::Limit("record count"));
    }
    increment(&mut counts.records)?;
    match record {
        ExportRecord::Resource(_) => increment(&mut counts.resources)?,
        ExportRecord::Edge(_) => increment(&mut counts.edges)?,
        ExportRecord::Scope(_) => increment(&mut counts.scopes)?,
        ExportRecord::Failure(_) => increment(&mut counts.failures)?,
        _ => return Err(ExportError::Invalid("envelope inside data records")),
    }
    Ok(counts)
}

#[tracing::instrument(skip_all)]
async fn write_chunk<W: AsyncWrite + Unpin>(
    sink: &mut W,
    digest: &mut Sha256,
    total: &mut u64,
    limit: u64,
    bytes: &[u8],
) -> Result<(), ExportError> {
    let length = u64::try_from(bytes.len()).map_err(|_| ExportError::Limit("output bytes"))?;
    let next = total
        .checked_add(length)
        .ok_or(ExportError::Limit("output byte overflow"))?;
    if next > limit {
        return Err(ExportError::Limit("output bytes"));
    }
    sink.write_all(bytes).await?;
    digest.update(bytes);
    *total = next;
    Ok(())
}
