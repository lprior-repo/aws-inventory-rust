use crate::transport_support::{
    TestResult,
    clock::Clock,
    request,
    watchdog::watch,
    wire::{self, Endpoint},
};
use aws_wire::transport::{Client, Config, TransportError};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use tokio::{io::AsyncReadExt, net::TcpStream};

const GOOD: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
const CLOSE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok";

#[tokio::test]
async fn checkout_wins_but_background_tls_cannot_keep_the_last_permit_forever() -> TestResult {
    // Given a started connector transferred into Hyper's background task after checkout wins.
    let endpoint = Endpoint::bind(true, b"http/1.1").await?;
    let client = bounded_client(&endpoint)?;
    let _clock = Clock::pause();
    make_checkout_win(&endpoint, &client).await?;
    let (mut stalled, _) = watch(endpoint.listener.accept()).await??;
    let mut hello = [0; 5];
    watch(stalled.read_exact(&mut hello)).await??;
    // When the abandoned TLS handshake reaches its original connector bound.
    expire_background(&client, &mut stalled).await?;
    // Then another host gets that exact permit, without counting a completed send as timed out.
    let next = Endpoint::bind(false, b"").await?;
    let mut send = Box::pin(client.send(request(&next.url)?));
    let mut stream = next.accept(&mut send).await?;
    let response = wire::complete(send, &mut stream, GOOD).await??;
    assert_eq!(response.body.as_ref(), b"ok");
    assert_eq!(client.statistics().connections_opened, 3);
    assert_eq!(client.statistics().timeouts, 0);
    endpoint.cleanup()?;
    next.cleanup()?;
    Ok(())
}

async fn make_checkout_win(endpoint: &Endpoint, client: &Client) -> TestResult {
    let mut first = Box::pin(client.send(request(&endpoint.url)?));
    let mut stream = endpoint.accept(&mut first).await?;
    wire::until(&mut first, wire::head(&mut stream)).await??;
    let mut second = Box::pin(client.send(request(&endpoint.url)?));
    crate::deadlines::poll_pending(&mut second).await?;
    tokio::time::advance(Duration::from_millis(50)).await;
    wire::write(&mut stream, GOOD).await?;
    assert_eq!(watch(first).await??.body.as_ref(), b"ok");
    // Receiving request two on the original TLS socket independently proves checkout won.
    assert_eq!(
        wire::complete(second, &mut stream, CLOSE)
            .await??
            .body
            .as_ref(),
        b"ok"
    );
    assert_eq!(client.statistics().requests_sent, 2);
    Ok(())
}

async fn expire_background(client: &Client, stream: &mut TcpStream) -> TestResult {
    assert_eq!(client.statistics().connections_opened, 2);
    assert_eq!(client.statistics().connections_open, 1);
    tokio::time::advance(Duration::from_millis(149)).await;
    assert_eq!(client.statistics().connections_open, 1);
    tokio::time::advance(Duration::from_millis(2)).await;
    let mut remaining = Vec::with_capacity(65_536);
    watch(stream.take(65_536).read_to_end(&mut remaining)).await??;
    assert_eq!(client.statistics().connections_open, 0);
    assert_eq!(client.statistics().timeouts, 0);
    Ok(())
}

#[tokio::test]
async fn cancellation_on_a_reused_connection_never_retries_a_second_attempt() -> TestResult {
    // Given a successfully pooled connection whose peer is closed before dispatcher polling.
    let endpoint = Endpoint::bind(false, b"").await?;
    let client = Client::new(endpoint.config(2))?;
    let _clock = Clock::pause();
    let mut first = Box::pin(client.send(request(&endpoint.url)?));
    let mut stream = endpoint.accept(&mut first).await?;
    let empty = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
    assert_eq!(
        wire::complete(first, &mut stream, empty)
            .await??
            .body
            .as_ref(),
        b""
    );
    let attempts = AtomicUsize::new(1); // Count the request actually read by the independent peer.
    drop(stream); // Do not yield: checkout sees the stale sender before its read task observes EOF.
    let mut second = Box::pin(client.send(request(&endpoint.url)?));
    crate::deadlines::poll_pending(&mut second).await?;
    // Then a forbidden retry would be observed as another actual accepted/requested connection.
    let result = reject_retry(second, &endpoint, &attempts).await?;
    assert!(matches!(result, Err(TransportError::Connect(_))));
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    endpoint.cleanup()?;
    Ok(())
}

fn bounded_client(endpoint: &Endpoint) -> Result<Client, TransportError> {
    Client::new(Config {
        connect_timeout: Duration::from_millis(200),
        ..endpoint.config(1)
    })
}

async fn reject_retry<F>(
    second: std::pin::Pin<Box<F>>,
    endpoint: &Endpoint,
    attempts: &AtomicUsize,
) -> Result<wire::SendResult, crate::transport_support::TestError>
where
    F: Future<Output = wire::SendResult>,
{
    watch(async {
        tokio::select! {
            result = second => Ok::<_, crate::transport_support::TestError>(result),
            accepted = endpoint.listener.accept() => {
                let (mut retry, _) = accepted?;
                let _attempt = wire::head(&mut retry).await?;
                attempts.fetch_add(1, Ordering::SeqCst);
                Err("server observed forbidden second request attempt".into())
            }
        }
    })
    .await?
}
