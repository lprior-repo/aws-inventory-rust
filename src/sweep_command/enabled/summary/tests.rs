use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_summary_when_metrics_printed_then_original_first_line_is_byte_identical() -> TestResult {
    let mut output = Vec::new();
    write_summary(&mut output, SweepSummary::default(), 7)?;
    let text = std::str::from_utf8(&output)?;
    let mut lines = text.lines();
    assert_eq!(
        lines.next(),
        Some(
            "resource_writes_scope=invocation resource_writes=0 rows=0 scopes=0 complete=0 partial=0 inaccessible=0 unsupported=0 failed=0 unobserved=0 graph_pending=0 work_remaining=0 unattributed_resources=7"
        )
    );
    assert_eq!(
        lines.next(),
        Some(
            "metrics bootstrap_ms=0 seed_ms=0 drain_ms=0 api_attempts=0 api_retries=0 api_throttles=0 connections_opened=0 requests_sent=0 requests_reused=0 http2_requests=0 bytes_read=0 pages=0 decode_ns=0 store_commit_ns=0 store_commits=0 claims=0 progress_dropped=0 cpu_user_ms=0 cpu_system_ms=0 peak_rss_kb=0 process_metrics=unavailable"
        )
    );
    assert_eq!(lines.next(), None);
    Ok(())
}

#[test]
fn given_available_process_metrics_when_printed_then_values_and_availability_are_reported()
-> TestResult {
    let mut summary = SweepSummary::default();
    summary.metrics.process_metrics_available = true;
    summary.metrics.cpu_user_ms = 10;
    summary.metrics.cpu_system_ms = 20;
    summary.metrics.peak_rss_kb = 30;
    let mut output = Vec::new();
    write_summary(&mut output, summary, 0)?;
    assert!(
        std::str::from_utf8(&output)?
            .contains("cpu_user_ms=10 cpu_system_ms=20 peak_rss_kb=30 process_metrics=available\n")
    );
    Ok(())
}

#[test]
fn given_failed_without_partial_when_summary_printed_then_failure_remains_visible() -> TestResult {
    let mut summary = SweepSummary::default();
    summary.scopes = 1;
    summary.outcomes.failed = 1;
    let mut output = Vec::new();
    write_summary(&mut output, summary, 0)?;
    let text = std::str::from_utf8(&output)?;
    let first = text.lines().next().ok_or("summary line missing")?;
    assert!(first.contains("scopes=1 complete=0 partial=0"));
    assert!(first.contains("failed=1"));
    Ok(())
}
