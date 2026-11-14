use crate::transport_support::{
    TestError, TestResult,
    clock::Clock,
    count,
    http2::{Body, Peer},
    request,
    watchdog::watch,
    wire,
};
use aws_wire::transport::{Client, Config, TransportError};
use bytes::Bytes;
use std::{pin::Pin, task::Poll, time::Duration};

#[tokio::test]
async fn cancel_after_h2_headers_preserves_an_in_flight_survivor_and_pool_reuse() -> TestResult {
    isolation(false).await
}

#[tokio::test]
async fn timeout_after_h2_headers_preserves_an_in_flight_survivor_and_pool_reuse() -> TestResult {
    isolation(true).await
}

async fn isolation(timeout: bool) -> TestResult {
    let mut peer = Peer::start().await?;
    let client = Client::new(Config {
        idle_timeout: Duration::from_secs(1),
        ..peer.endpoint.config(2)
    })?;
    let clock = Clock::pause();
    let mut victim = Box::pin(client.send(request(&peer.endpoint.url)?));
    let _victim_body = partial(&mut peer, &client, &mut victim).await?;
    if timeout {
        keep_alive(&mut peer, &client).await?;
    }
    let mut survivor = Box::pin(client.send(request(&peer.endpoint.url)?));
    let body = headers(&mut peer, &mut survivor, 2).await?;
    crate::deadlines::poll_pending(&mut survivor).await?;

    cancel_or_timeout(victim, &clock, timeout).await?;
    finish_survivor(&mut peer, &client, survivor, body).await?;
    assert_eq!(client.statistics().timeouts, usize::from(timeout));
    peer.stop().await
}

async fn keep_alive(peer: &mut Peer, client: &Client) -> TestResult {
    for _ in 0..2 {
        tokio::time::advance(Duration::from_millis(900)).await;
        one(peer, client).await?;
    }
    Ok(())
}

async fn cancel_or_timeout<F>(send: Pin<Box<F>>, clock: &Clock, timeout: bool) -> TestResult
where
    F: Future<Output = wire::SendResult>,
{
    if timeout {
        assert!(matches!(
            clock.expire(send, Duration::from_secs(2)).await?,
            Err(TransportError::Timeout)
        ));
    } else {
        drop(send);
    }
    Ok(())
}

async fn finish_survivor<F>(
    peer: &mut Peer,
    client: &Client,
    send: Pin<Box<F>>,
    mut body: Body,
) -> TestResult
where
    F: Future<Output = wire::SendResult>,
{
    body.data(Bytes::from_static(b"ok"), true)?;
    assert_eq!(watch(send).await??.body.as_ref(), b"ok");

    one(peer, client).await?;
    assert_eq!(count(&peer.accepts), 1);
    assert_eq!(client.statistics().connections_open, 1);
    Ok(())
}

async fn headers<F>(
    peer: &mut Peer,
    send: &mut Pin<Box<F>>,
    length: usize,
) -> Result<Body, TestError>
where
    F: Future<Output = wire::SendResult>,
{
    let stream = wire::until(send, peer.requests.recv())
        .await?
        .ok_or("missing h2 stream")?;
    stream.headers(length)
}

async fn partial<F>(
    peer: &mut Peer,
    client: &Client,
    send: &mut Pin<Box<F>>,
) -> Result<Body, TestError>
where
    F: Future<Output = wire::SendResult>,
{
    let baseline = client.statistics().bytes_read;
    let mut body = headers(peer, send, 16_385).await?;
    body.data(Bytes::from(vec![b'x'; 16_384]), false)?;

    wire::read_at_least(client, send, baseline.saturating_add(8192)).await?;
    Ok(body)
}

async fn one(peer: &mut Peer, client: &Client) -> TestResult {
    let mut send = Box::pin(client.send(request(&peer.endpoint.url)?));
    let mut body = headers(peer, &mut send, 2).await?;
    body.data(Bytes::from_static(b"ok"), true)?;
    let response = watch(send).await??;
    assert_eq!(response.version, http::Version::HTTP_2);
    assert_eq!(response.body.as_ref(), b"ok");
    Ok(())
}

#[tokio::test]
async fn h2_connection_failure_recovers_on_a_new_accept() -> TestResult {
    let mut peer = Peer::start().await?;
    let client = Client::new(peer.endpoint.config(2))?;
    let _clock = Clock::pause();
    let mut send = Box::pin(client.send(request(&peer.endpoint.url)?));
    let body = partial(&mut peer, &client, &mut send).await?;
    body.close.notify_one();
    assert!(matches!(
        watch(send).await?,
        Err(TransportError::Connect(_))
    ));

    one(&mut peer, &client).await?;
    assert_eq!(count(&peer.accepts), 2);
    assert_eq!(client.statistics().connections_opened, 2);
    assert_eq!(client.statistics().connections_open, 1);
    peer.stop().await
}

#[tokio::test]
async fn h2_checkout_expiry_preserves_the_live_body_and_its_permit_until_completion() -> TestResult
{
    let mut peer = Peer::start().await?;
    let client = Client::new(peer.endpoint.config(2))?;
    let _clock = Clock::pause();
    let mut send = Box::pin(client.send(request(&peer.endpoint.url)?));
    let mut body = partial(&mut peer, &client, &mut send).await?;
    tokio::time::advance(Duration::from_millis(91)).await;

    one(&mut peer, &client).await?;
    assert_eq!(count(&peer.accepts), 2);
    assert_eq!(client.statistics().connections_open, 2);
    body.data(Bytes::from_static(b"z"), true)?;
    let response = watch(send).await??;
    let mut expected = vec![b'x'; 16_384];
    expected.push(b'z');
    assert_eq!(response.body.as_ref(), expected);
    settle(&client, 1).await?;
    one(&mut peer, &client).await?;
    assert_eq!(count(&peer.accepts), 2);
    peer.stop().await
}

async fn settle(client: &Client, open: usize) -> TestResult {
    watch(std::future::poll_fn(|cx| {
        if client.statistics().connections_open == open {
            return Poll::Ready(());
        }
        cx.waker().wake_by_ref();
        Poll::Pending
    }))
    .await?;
    assert_eq!(client.statistics().connections_open, open);
    Ok(())
}
