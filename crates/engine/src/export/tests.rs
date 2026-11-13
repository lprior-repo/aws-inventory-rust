//! Deterministic publication and cancellation boundaries inaccessible to callers.

use super::*;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::AsyncWrite;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

fn receipt() -> ExportSummary {
    ExportSummary {
        scan_id: "finished".into(),
        status: ScanStatus::Complete,
        format: ExportFormat::Jsonl,
        provenance: ProvenanceMode::Normalized,
        counts: ExportCounts::default(),
        bytes: 2,
        sha256: "fixture-receipt".into(),
    }
}

#[tokio::test]
async fn cancelled_after_writing_before_publication_removes_its_partial() -> TestResult {
    let directory = tempfile::tempdir()?;
    let output = directory.path().join("finished.jsonl");
    let temporary = partial_path(&output)?;
    tokio::fs::write(&temporary, b"{}\n").await?;
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    assert!(matches!(
        publish_or_discard(Ok(receipt()), &temporary, &output, &cancellation).await,
        Err(ExportError::Cancelled)
    ));
    assert!(!temporary.exists());
    assert!(!output.exists());
    Ok(())
}

#[tokio::test]
async fn publication_race_preserves_the_winner_and_discards_only_our_partial() -> TestResult {
    let directory = tempfile::tempdir()?;
    let output = directory.path().join("finished.jsonl");
    let temporary = partial_path(&output)?;
    reject_existing(&output).await?;
    tokio::fs::write(&temporary, b"our candidate").await?;
    tokio::fs::write(&output, b"another publisher won").await?;
    assert!(matches!(
        publish_or_discard(Ok(receipt()), &temporary, &output, &CancellationToken::new()).await,
        Err(ExportError::Io(error)) if error.kind() == std::io::ErrorKind::AlreadyExists
    ));
    assert!(!temporary.exists());
    assert_eq!(tokio::fs::read(output).await?, b"another publisher won");
    Ok(())
}

#[tokio::test]
async fn read_wait_is_cancelled_while_the_future_is_pending() -> TestResult {
    let cancellation = CancellationToken::new();
    let mut operation = Box::pin(await_read(
        &cancellation,
        std::future::pending::<Result<(), StoreError>>(),
    ));
    assert!(futures_util::poll!(&mut operation).is_pending());
    cancellation.cancel();
    assert!(matches!(operation.await, Err(ExportError::Cancelled)));
    Ok(())
}

struct CancelOnWrite {
    token: CancellationToken,
    writes: usize,
    bytes: usize,
}

impl AsyncWrite for CancelOnWrite {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        self.writes = self.writes.saturating_add(1);
        self.bytes = self.bytes.saturating_add(bytes.len());
        self.token.cancel();
        Poll::Ready(Ok(bytes.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[tokio::test]
async fn cancellation_during_a_record_drains_framing_then_stops_intake() -> TestResult {
    let cancellation = CancellationToken::new();
    let sink = CancelOnWrite {
        token: cancellation.clone(),
        writes: 0,
        bytes: 0,
    };
    let mut writer = ArtifactWriter::new(sink, ExportOptions::default());
    let record = ExportRecord::Scope(ExportScope {
        scope: CollectionScope {
            account_id: "123456789012".into(),
            region: "us-east-1".into(),
            collector: "empty".into(),
            parent_id: None,
            source: inventory_core::Source::Direct,
        },
        last_page: 1,
        complete: true,
        failed: false,
    });
    assert!(matches!(
        writer.record(record.clone(), &cancellation).await,
        Err(ExportError::Cancelled)
    ));
    assert_eq!(writer.sink.writes, 2); // Complete envelope, then newline.
    let previous = writer.sink.bytes;
    assert!(matches!(
        writer.record(record, &cancellation).await,
        Err(ExportError::Cancelled)
    ));
    assert_eq!(writer.sink.bytes, previous);
    Ok(())
}

#[test]
fn drain_failure_preserves_original_failure_instead_of_claiming_success() {
    let failure = combine_drain(
        Err(ExportError::Cancelled),
        Err(std::io::Error::other("drain failed")),
    );
    assert!(
        matches!(failure, Err(ExportError::Cleanup { cause, drain: Some(_), removal: None }) if matches!(*cause, ExportError::Cancelled))
    );
}
