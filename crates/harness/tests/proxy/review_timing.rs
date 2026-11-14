use super::{
    clock::{FrozenClock, within},
    support::{OK, Server, TestResult, exchange, field, ready, request},
    wire,
};
use inventory_harness::proxy::{Config, Proxy, ScopeFilter, ScopeName};
use std::time::Duration;
use tokio::{io::AsyncWriteExt, net::TcpStream};

fn delayed(server: &Server, account: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let mut config = server.config();
    config.latency.push((
        ScopeFilter {
            account: Some(ScopeName::parse(account)?),
            ..Default::default()
        },
        Duration::from_secs(2),
    ));
    Ok(config)
}

#[tokio::test(start_paused = true)]
async fn forwarded_latency_delays_matching_scope_but_not_other_scopes() -> TestResult {
    let _clock = FrozenClock::new();
    within("scoped fault-free latency", forwarded_latency()).await
}

async fn forwarded_latency() -> TestResult {
    let server = Server::start(OK).await?;
    let proxy = Proxy::start(delayed(&server, "a")?).await?;
    let mut client = TcpStream::connect(proxy.address()).await?;
    client.write_all(&request("a", "us-east-1", "s3")).await?;
    pending(&proxy, &client, 1, 0).await?;
    let other = request("b", "us-east-1", "s3");
    assert_eq!(exchange(proxy.address(), &other).await?, OK);
    tokio::time::advance(Duration::from_secs(1)).await;
    pending(&proxy, &client, 2, 1).await?;
    tokio::time::advance(Duration::from_secs(1)).await;
    assert_eq!(wire::remaining(&mut client).await?, OK);
    assert_eq!(proxy.snapshot()?.counters.delayed, 1);
    proxy.shutdown().await?;
    Ok(())
}

async fn pending(proxy: &Proxy, client: &TcpStream, prepared: u64, written: u64) -> TestResult {
    ready(proxy, prepared).await?;
    let counters = proxy.snapshot()?.counters;
    assert_eq!(counters.responses_ready, prepared);
    assert_eq!(counters.responses_written, written);
    wire::held(client);
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn scope_capacity_rejection_keeps_parsed_scope_and_its_latency() -> TestResult {
    let _clock = FrozenClock::new();
    within("latency on scope capacity rejection", scope_capacity()).await
}

async fn scope_capacity() -> TestResult {
    let server = Server::start(OK).await?;
    let mut config = delayed(&server, "b")?;
    config.scope_capacity = 1;
    let proxy = Proxy::start(config).await?;
    assert_eq!(
        exchange(proxy.address(), &request("a", "us-east-1", "s3")).await?,
        OK
    );
    delayed_rejection(&proxy, &request("b", "us-east-1", "s3"), 2, 503).await?;
    assert_eq!(proxy.snapshot()?.counters.forwarded, 1);
    proxy.shutdown().await?;
    Ok(())
}

async fn delayed_rejection(
    proxy: &Proxy,
    request: &[u8],
    prepared: u64,
    status: u16,
) -> TestResult {
    let mut client = TcpStream::connect(proxy.address()).await?;
    client.write_all(request).await?;
    pending(proxy, &client, prepared, prepared.saturating_sub(1)).await?;
    tokio::time::advance(Duration::from_secs(1)).await;
    pending(proxy, &client, prepared, prepared.saturating_sub(1)).await?;
    tokio::time::advance(Duration::from_secs(1)).await;
    let bytes = wire::remaining(&mut client).await?;
    assert!(bytes.starts_with(format!("HTTP/1.1 {status} ").as_bytes()));
    assert_eq!(field(&bytes, "connection")?, "close");
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn parsed_expectation_rejection_also_receives_matching_latency() -> TestResult {
    let _clock = FrozenClock::new();
    within("latency on parsed Expect rejection", expectation()).await
}

async fn expectation() -> TestResult {
    let server = Server::start(OK).await?;
    let proxy = Proxy::start(delayed(&server, "b")?).await?;
    let bytes = b"POST / HTTP/1.1\r\nHost: x\r\nAuthorization: AWS4-HMAC-SHA256 Credential=b/20261113/us-east-1/s3/aws4_request\r\nExpect: 100-continue\r\nContent-Length: 1\r\n\r\n";
    delayed_rejection(&proxy, bytes, 1, 417).await?;
    assert_eq!(proxy.snapshot()?.counters.forwarded, 0);
    proxy.shutdown().await?;
    Ok(())
}

pub async fn canned_latency() -> TestResult {
    let server = Server::start(OK).await?;
    let mut config = server.fault(inventory_harness::proxy::Fault::Throttle);
    config
        .latency
        .push((ScopeFilter::default(), Duration::from_secs(2)));
    let proxy = Proxy::start(config).await?;
    let mut client = TcpStream::connect(proxy.address()).await?;
    client.write_all(super::support::UNSIGNED).await?;
    pending(&proxy, &client, 1, 0).await?;
    tokio::time::advance(Duration::from_secs(1)).await;
    pending(&proxy, &client, 1, 0).await?;
    tokio::time::advance(Duration::from_secs(1)).await;
    let bytes = wire::remaining(&mut client).await?;
    assert_eq!(field(&bytes, "x-amzn-ErrorType")?, "ThrottlingException");
    assert_eq!(proxy.snapshot()?.counters.delayed, 1);
    proxy.shutdown().await?;
    Ok(())
}
