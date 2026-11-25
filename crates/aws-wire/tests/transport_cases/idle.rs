use crate::transport_support::{
    Mode, Server, TestError, TestResult,
    clock::Clock,
    config, count, request,
    watchdog::watch,
    wire::{self, Endpoint, Io, SendResult},
};
use aws_wire::transport::{Client, Config};
use std::{pin::Pin, time::Duration};

const GOOD: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
const CLOSE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok";
type Sending<'a> = Pin<Box<dyn Future<Output = SendResult> + Send + 'a>>;
struct Active<'a> {
    send: Sending<'a>,
    stream: Io,
}

#[tokio::test]
async fn an_idle_tls_socket_retains_the_last_permit_until_eviction() -> TestResult {
    retention(true).await
}

pub(crate) async fn retention(tls: bool) -> TestResult {
    let first = Endpoint::bind(tls, b"http/1.1").await?;
    let second = Endpoint::bind(false, b"").await?;
    let client = Client::new(first.config(1))?;
    let _clock = Clock::pause();
    let mut send = Box::pin(client.send(request(&first.url)?));
    let mut stream = first.accept(&mut send).await?;
    let response = wire::complete(send, &mut stream, GOOD).await??;
    assert_eq!(response.body.as_ref(), b"ok");
    let mut next = Box::pin(client.send(request(&second.url)?));
    before_eviction(&client, &second, &mut next).await?;

    tokio::time::advance(Duration::from_millis(91)).await;
    let mut next_stream = second.accept(&mut next).await?;
    let response = wire::complete(next, &mut next_stream, GOOD).await??;
    assert_eq!(response.body.as_ref(), b"ok");
    assert_eq!(client.statistics().connections_opened, 2);
    assert_eq!(client.statistics().connections_open, 1);
    first.cleanup()?;
    second.cleanup()?;
    Ok(())
}

async fn before_eviction<F>(
    client: &Client,
    endpoint: &Endpoint,
    send: &mut Pin<Box<F>>,
) -> TestResult
where
    F: Future<Output = SendResult>,
{
    crate::deadlines::poll_pending(send).await?;
    tokio::time::advance(Duration::from_millis(9)).await;
    crate::deadlines::poll_pending(send).await?;

    wire::no_accept(&endpoint.listener).await?;
    assert_eq!(client.statistics().connections_open, 1);
    Ok(())
}

#[tokio::test]
async fn gated_active_connections_block_an_extra_endpoint_until_a_socket_closes() -> TestResult {
    let first = Endpoint::bind(false, b"").await?;
    let second = Endpoint::bind(false, b"").await?;
    let third = Endpoint::bind(false, b"").await?;
    let client = Client::new(first.config(2))?;
    let _clock = Clock::pause();
    let one = active(&first, &client).await?;
    let two = active(&second, &client).await?;
    let mut three = Box::pin(client.send(request(&third.url)?));
    crate::deadlines::poll_pending(&mut three).await?;
    wire::no_accept(&third.listener).await?;
    assert_eq!(client.statistics().connections_open, 2);

    finish(one).await?;
    let mut stream = third.accept(&mut three).await?;
    let response = wire::complete(three, &mut stream, CLOSE).await??;
    assert_eq!(response.body.as_ref(), b"ok");
    finish(two).await?;
    assert_eq!(client.statistics().connections_opened, 3);
    first.cleanup()?;
    second.cleanup()?;
    third.cleanup()?;
    Ok(())
}

async fn active<'a>(endpoint: &Endpoint, client: &'a Client) -> Result<Active<'a>, TestError> {
    let mut send: Sending<'a> = Box::pin(client.send(request(&endpoint.url)?));
    let mut stream = endpoint.accept(&mut send).await?;
    wire::until(&mut send, wire::head(&mut stream)).await??;
    Ok(Active { send, stream })
}

async fn finish(mut active: Active<'_>) -> TestResult {
    wire::write(&mut active.stream, CLOSE).await?;
    assert_eq!(watch(active.send).await??.body.as_ref(), b"ok");
    Ok(())
}

#[test]
fn idle_timeout_can_exceed_connect_timeout_without_extending_request_deadline() -> TestResult {
    let config = Config {
        connect_timeout: Duration::from_secs(10),
        request_deadline: Duration::from_secs(30),
        idle_timeout: Duration::from_secs(12),
        ..Config::default()
    };
    assert_eq!(Client::new(config)?.statistics(), Default::default());
    Ok(())
}

#[tokio::test]
async fn paced_bucket_reuses_connection_when_five_seconds_is_within_idle_timeout() -> TestResult {
    paced_bucket(Duration::from_secs(15), 1).await
}

#[tokio::test]
async fn paced_bucket_reconnects_when_five_seconds_exceeds_idle_timeout() -> TestResult {
    paced_bucket(Duration::from_secs(1), 2).await
}

async fn paced_bucket(idle_timeout: Duration, expected_accepts: usize) -> TestResult {
    let server = Server::start(Mode::Good, Default::default()).await?;
    let client = Client::new(aws_wire::transport::Config {
        idle_timeout,
        request_deadline: Duration::from_secs(30),
        connect_timeout: Duration::from_secs(15),
        ..config()
    })?;
    let _clock = Clock::pause();

    let first = watch(client.send(request(&server.url)?)).await??;
    assert_eq!(first.body.as_ref(), b"ok");
    tokio::time::advance(Duration::from_secs(5)).await;
    let second = watch(client.send(request(&server.url)?)).await??;

    assert_eq!(second.body.as_ref(), b"ok");
    assert_eq!(count(&server.counts.accepts), expected_accepts);
    assert_eq!(client.statistics().connections_opened, expected_accepts);
    server.stop().await
}

#[tokio::test]
async fn given_different_authorization_headers_when_sending_then_the_pool_reuses_the_connection()
-> TestResult {
    let first = Server::start(Mode::Good, Default::default()).await?;
    let other = Server::start(Mode::Good, Default::default()).await?;
    let client = Client::new(aws_wire::transport::Config {
        idle_timeout: Duration::from_secs(15),
        request_deadline: Duration::from_secs(30),
        connect_timeout: Duration::from_secs(15),
        ..config()
    })?;
    let _clock = Clock::pause();
    let signed = |uri: &str, authorization: &'static str| {
        http::Request::builder()
            .uri(uri)
            .header(http::header::AUTHORIZATION, authorization)
            .body(bytes::Bytes::new())
    };
    for (server, authorization) in [
        (&first, "AWS4-HMAC-SHA256 Credential=first-account"),
        (&first, "AWS4-HMAC-SHA256 Credential=second-account"),
        (&other, "AWS4-HMAC-SHA256 Credential=second-account"),
    ] {
        let response = watch(client.send(signed(&server.url, authorization)?)).await??;
        assert_eq!(response.body.as_ref(), b"ok");
    }

    assert_eq!(count(&first.counts.accepts), 1);
    assert_eq!(count(&other.counts.accepts), 1);
    assert_eq!(client.statistics().hosts, 2);
    tokio::try_join!(first.stop(), other.stop())?;
    Ok(())
}
