use anyhow::Result;
use inventory_sweep::{SweepMetrics, SweepSummary};
use std::io::Write;

pub(super) fn print_summary(summary: SweepSummary, unattributed_resources: u64) -> Result<()> {
    write_summary(
        &mut std::io::stdout().lock(),
        summary,
        unattributed_resources,
    )
}

fn write_summary(output: &mut impl Write, summary: SweepSummary, unattributed: u64) -> Result<()> {
    let outcomes = summary.outcomes;
    writeln!(
        output,
        "resource_writes_scope=invocation resource_writes={} rows={} scopes={} complete={} partial={} inaccessible={} unsupported={} failed={} unobserved={} graph_pending={} work_remaining={} unattributed_resources={}",
        summary.resource_writes,
        summary.rows,
        summary.scopes,
        outcomes.complete,
        outcomes.partial,
        outcomes.inaccessible,
        outcomes.unsupported,
        outcomes.failed,
        outcomes.unobserved,
        summary.graph_pending,
        summary.work_remaining,
        unattributed
    )?;
    write_metrics(output, &summary.metrics)?;
    if outcomes.not_applicable > 0 {
        writeln!(output, "outcomes_v1 not_applicable={}", outcomes.not_applicable)?;
    }
    Ok(())
}

fn write_metrics(output: &mut impl Write, metrics: &SweepMetrics) -> Result<()> {
    write!(
        output,
        "metrics bootstrap_ms={} seed_ms={} drain_ms={}",
        metrics.bootstrap_ms, metrics.seed_ms, metrics.drain_ms
    )?;
    write_network(output, metrics)?;
    write_store(output, metrics)?;
    write_process(output, metrics)
}

fn write_network(output: &mut impl Write, metrics: &SweepMetrics) -> Result<()> {
    write!(
        output,
        " api_attempts={} api_retries={} api_throttles={} connections_opened={} requests_sent={} requests_reused={} http2_requests={} bytes_read={} handshake_ns={} hosts={}",
        metrics.api_attempts,
        metrics.api_retries,
        metrics.api_throttles,
        metrics.connections_opened,
        metrics.requests_sent,
        metrics.requests_reused,
        metrics.http2_requests,
        metrics.bytes_read,
        metrics.handshake_ns,
        metrics.hosts
    )?;
    Ok(())
}

fn write_store(output: &mut impl Write, metrics: &SweepMetrics) -> Result<()> {
    write!(
        output,
        " pages={} decode_ns={} store_commit_ns={} store_commits={} claims={} progress_dropped={} child_requests_skipped={}",
        metrics.pages,
        metrics.decode_ns,
        metrics.store_commit_ns,
        metrics.store_commits,
        metrics.claims,
        metrics.progress_dropped,
        metrics.child_requests_skipped
    )?;
    Ok(())
}

fn write_process(output: &mut impl Write, metrics: &SweepMetrics) -> Result<()> {
    let available = if metrics.process_metrics_available {
        "available"
    } else {
        "unavailable"
    };
    writeln!(
        output,
        " cpu_user_ms={} cpu_system_ms={} peak_rss_kb={} process_metrics={available}",
        metrics.cpu_user_ms, metrics.cpu_system_ms, metrics.peak_rss_kb
    )?;
    Ok(())
}

#[cfg(test)]
mod tests;
