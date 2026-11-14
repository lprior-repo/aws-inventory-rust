use crate::transport_support::{
    TestError, TestResult,
    clock::Clock,
    request,
    watchdog::watch,
    wire::{self, Endpoint, Io, SendResult},
};
use aws_wire::transport::Client;
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
