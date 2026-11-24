use crate::transport_support::{
    TestError, TestResult,
    clock::Clock,
    request,
    watchdog::watch,
    wire::{self, Endpoint},
};
use aws_wire::transport::{Client, HostConnections, Statistics, TransportError};
use tokio::{io::AsyncReadExt, net::TcpSocket};

const GOOD: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
const PARTIAL: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\nx";

#[tokio::test]
async fn multiple_connections_have_exact_reuse_and_plaintext_byte_totals() -> TestResult {
    let first = Endpoint::bind(false, b"").await?;
    let second = Endpoint::bind(false, b"").await?;
    let client = Client::new(first.config(2))?;
    let _clock = Clock::pause();
    let mut send = Box::pin(client.send(request(&first.url)?));
    let mut one = first.accept(&mut send).await?;
    assert_eq!(
        wire::complete(send, &mut one, GOOD).await??.body.as_ref(),
        b"ok"
    );
    let send = Box::pin(client.send(request(&first.url)?));
    assert_eq!(
        wire::complete(send, &mut one, GOOD).await??.body.as_ref(),
        b"ok"
    );
    let mut send = Box::pin(client.send(request(&second.url)?));
    let mut two = second.accept(&mut send).await?;
    assert_eq!(
        wire::complete(send, &mut two, GOOD).await??.body.as_ref(),
        b"ok"
    );

    assert_eq!(
        counted(&client)?,
        snapshot(2, 2, 3, 0, GOOD.len().saturating_mul(3), 2)
    );
    // The second request to the first endpoint reused its connection, so
    // each authority has exactly one.
    let mut expected = vec![host(&first.url, 1)?, host(&second.url, 1)?];
    expected.sort_by(|left, right| left.authority.cmp(&right.authority));
    assert_eq!(client.hosts(), expected);
    first.cleanup()?;
    second.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn invalid_requests_leave_every_counter_unchanged() -> TestResult {
    let client = Client::new(crate::config())?;
    assert!(matches!(
        client.send(request("/relative")?).await,
        Err(TransportError::InvalidRequest(_))
    ));
    assert_eq!(client.statistics(), Statistics::default());
    assert_eq!(client.hosts(), Vec::new());
    Ok(())
}

#[tokio::test]
async fn failed_connection_records_one_attempt_and_no_socket_or_timeout() -> TestResult {
    let socket = TcpSocket::new_v4()?;
    socket.bind("127.0.0.1:0".parse()?)?;
    let client = Client::new(crate::config())?;
    let url = format!("http://{}/", socket.local_addr()?);
    assert!(matches!(
        watch(client.send(request(&url)?)).await?,
        Err(TransportError::Connect(_))
    ));
    // The refused attempt took time and names its authority, with no
    // connection counted for it.
    assert_eq!(counted(&client)?, snapshot(0, 0, 1, 0, 0, 1));
    assert_eq!(client.hosts(), vec![host(&url, 0)?]);
    Ok(())
}

#[tokio::test]
async fn caller_cancellation_is_not_a_timeout_and_closes_the_partial_socket() -> TestResult {
    let endpoint = Endpoint::bind(false, b"").await?;
    let client = Client::new(endpoint.config(1))?;
    let _clock = Clock::pause();
    let mut send = Box::pin(client.send(request(&endpoint.url)?));
    let mut stream = endpoint.accept(&mut send).await?;
    wire::until(&mut send, wire::head(&mut stream)).await??;
    wire::write(&mut stream, PARTIAL).await?;
    wire::read_at_least(&client, &mut send, PARTIAL.len()).await?;

    drop(send);
    eof(&mut stream).await?;
    assert_eq!(counted(&client)?, snapshot(1, 0, 1, 0, PARTIAL.len(), 1));
    endpoint.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn request_timeout_is_counted_exactly_once_after_socket_cleanup() -> TestResult {
    let endpoint = Endpoint::bind(false, b"").await?;
    let config = endpoint.config(1);
    let deadline = config.request_deadline;
    let client = Client::new(config)?;
    let clock = Clock::pause();
    let mut send = Box::pin(client.send(request(&endpoint.url)?));
    let mut stream = endpoint.accept(&mut send).await?;
    wire::until(&mut send, wire::head(&mut stream)).await??;
    wire::write(&mut stream, PARTIAL).await?;
    wire::read_at_least(&client, &mut send, PARTIAL.len()).await?;
    assert!(matches!(
        clock.expire(send, deadline).await?,
        Err(TransportError::Timeout)
    ));
    eof(&mut stream).await?;
    assert_eq!(counted(&client)?, snapshot(1, 0, 1, 1, PARTIAL.len(), 1));
    endpoint.cleanup()?;
    Ok(())
}

async fn eof(stream: &mut wire::Io) -> TestResult {
    let mut byte = [0];
    assert_eq!(watch(stream.read(&mut byte)).await??, 0);
    Ok(())
}

/// The client's statistics with the connect time removed, so the rest can be
/// compared exactly. Connect time is wall-clock and differs per run; a zero
/// means it was never recorded, which fails here.
fn counted(client: &Client) -> Result<Statistics, TestError> {
    let statistics = client.statistics();
    if statistics.handshake_ns == 0 {
        return Err("no connect time was recorded".into());
    }
    Ok(Statistics {
        handshake_ns: 0,
        ..statistics
    })
}

/// The authority of a test URL of the form `http://<authority>/`.
fn host(url: &str, connections: usize) -> Result<HostConnections, TestError> {
    let authority = url
        .strip_prefix("http://")
        .and_then(|rest| rest.strip_suffix('/'))
        .ok_or("test URL is not http://<authority>/")?;
    Ok(HostConnections {
        authority: authority.into(),
        connections,
    })
}

fn snapshot(
    opened: usize,
    open: usize,
    sent: usize,
    timeouts: usize,
    bytes_read: usize,
    hosts: usize,
) -> Statistics {
    Statistics {
        connections_opened: opened,
        connections_open: open,
        requests_sent: sent,
        requests_reused: sent.saturating_sub(opened),
        timeouts,
        bytes_read,
        http2_requests: 0,
        handshake_ns: 0,
        hosts,
    }
}
