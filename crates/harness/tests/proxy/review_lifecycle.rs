use super::{
    clock::{FrozenClock, until, within},
    support::{OK, Server, TestResult, UNSIGNED, exchange, finished, reached, request},
    wire,
};
use inventory_harness::proxy::{DrainReport, Fault, IDLE_TIMEOUT, MAX_BODY_BYTES, Proxy};
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::test]
async fn drain_report_distinguishes_completed_failed_and_cancelled_sessions() -> TestResult {
    within("mixed terminal outcomes", mixed_outcomes()).await
}

async fn mixed_outcomes() -> TestResult {
    let server = Server::start(OK).await?;
    let proxy = Proxy::start(server.fault(Fault::Stall)).await?;
    assert_eq!(
        exchange(proxy.address(), &request("a", "us-east-1", "s3")).await?,
        OK
    );
    let failure = exchange(proxy.address(), b"GET / HTTP/1.1\nHost: x\n\n").await?;
    assert!(failure.starts_with(b"HTTP/1.1 400 "));
    finished(&proxy, 2).await?;
    let mut stalled = TcpStream::connect(proxy.address()).await?;
    stalled.write_all(UNSIGNED).await?;
    reached(&proxy, 2).await?;
    let report = proxy.shutdown().await?;
    assert_report(&report, (3, 2, 1, 1));
    Ok(())
}

pub(super) fn assert_report(report: &DrainReport, expected: (u64, u64, u64, u64)) {
    assert_eq!(
        (
            report.accepted,
            report.completed,
            report.failed,
            report.cancelled
        ),
        expected
    );
    assert_eq!(report.aborted, 0);
    assert_eq!(report.panicked, 0);
    assert_eq!(
        report.accepted,
        report
            .completed
            .saturating_add(report.cancelled)
            .saturating_add(report.aborted)
            .saturating_add(report.panicked)
    );
}

#[tokio::test(start_paused = true)]
async fn stall_survives_idle_timeout_and_client_eof_completes_before_shutdown() -> TestResult {
    let _clock = FrozenClock::new();
    within("stall EOF without proxy shutdown", idle_stall()).await
}

async fn idle_stall() -> TestResult {
    let server = Server::start(OK).await?;
    let proxy = Proxy::start(server.fault(Fault::Stall)).await?;
    let mut client = TcpStream::connect(proxy.address()).await?;
    client.write_all(UNSIGNED).await?;
    reached(&proxy, 1).await?;
    let window = IDLE_TIMEOUT
        .checked_add(Duration::from_secs(1))
        .ok_or("window overflow")?;
    tokio::time::advance(window).await;
    client.write_all(b"probe").await?;
    stall_bytes(&proxy, 5).await?;
    wire::held(&client);
    client.shutdown().await?;
    finished(&proxy, 1).await?;
    assert_eq!(client.read(&mut [0]).await?, 0);
    assert_report(&proxy.shutdown().await?, (1, 1, 0, 0));
    Ok(())
}

pub async fn stall_bytes(proxy: &Proxy, bytes: u64) -> TestResult {
    until("stalled peer data consumed without a response", || {
        Ok(proxy.snapshot()?.counters.stalled_bytes >= bytes)
    })
    .await
}

#[tokio::test]
async fn stall_remains_open_after_discarding_more_than_the_body_cap() -> TestResult {
    within("stall beyond the old cumulative read cutoff", flood_stall()).await
}

async fn flood_stall() -> TestResult {
    const BLOCK: usize = 4_096;
    const BLOCKS: usize = (MAX_BODY_BYTES / BLOCK).saturating_add(1);
    let server = Server::start(OK).await?;
    let proxy = Proxy::start(server.fault(Fault::Stall)).await?;
    let mut client = TcpStream::connect(proxy.address()).await?;
    client.write_all(UNSIGNED).await?;
    reached(&proxy, 1).await?;
    for _ in 0..BLOCKS {
        client.write_all(&[1; BLOCK]).await?;
    }
    let bytes = u64::try_from(BLOCKS.checked_mul(BLOCK).ok_or("flood overflow")?)?;
    stall_bytes(&proxy, bytes).await?;
    wire::held(&client);
    client.shutdown().await?;
    finished(&proxy, 1).await?;
    assert_report(&proxy.shutdown().await?, (1, 1, 0, 0));
    Ok(())
}

pub async fn shutdown_stall() -> TestResult {
    let server = Server::start(OK).await?;
    let proxy = Proxy::start(server.fault(Fault::Stall)).await?;
    let mut client = TcpStream::connect(proxy.address()).await?;
    client.write_all(UNSIGNED).await?;
    reached(&proxy, 1).await?;
    let window = IDLE_TIMEOUT
        .checked_add(Duration::from_secs(1))
        .ok_or("window overflow")?;
    tokio::time::advance(window).await;
    client.write_all(b"probe").await?;
    stall_bytes(&proxy, 5).await?;
    assert_eq!(
        proxy
            .snapshot()?
            .decisions
            .first()
            .and_then(|entry| entry.decision.fault),
        Some(Fault::Stall)
    );
    wire::held(&client);
    let report = proxy.shutdown().await?;
    assert_report(&report, (1, 0, 0, 1));
    assert_eq!(client.read(&mut [0]).await?, 0);
    Ok(())
}
