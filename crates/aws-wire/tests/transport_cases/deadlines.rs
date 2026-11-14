use crate::transport_support::{Mode, Server, TestResult, count, request};
use aws_wire::transport::{Client, Response, TransportError};
use std::{pin::Pin, sync::Arc};

#[tokio::test]
async fn connection_cap_wait_is_covered_by_the_whole_request_deadline() -> TestResult {
    let mut active = Server::start(Mode::BeforeHeaders, Arc::default()).await?;
    let blocked = Server::start(Mode::Good, Arc::default()).await?;
    let client = Client::new(crate::capped_config(1))?;
    let clock = crate::transport_support::clock::Clock::pause();
    let mut first = Box::pin(client.send(request(&active.url)?));
    crate::observe_stall(&mut active, &mut first).await?;

    let mut second = Box::pin(client.send(request(&blocked.url)?));
    poll_pending(&mut second).await?;
    let result = clock
        .expire(second, crate::config().request_deadline)
        .await?;

    assert_timeout(result);
    assert_timeout(crate::transport_support::watchdog::watch(first).await?);
    assert_eq!(client.statistics().timeouts, 2);
    assert_eq!(count(&blocked.counts.accepts), 0);
    active.stop().await?;
    blocked.stop().await
}

pub(crate) async fn poll_pending<F>(send: &mut Pin<Box<F>>) -> TestResult
where
    F: Future<Output = Result<Response, TransportError>>,
{
    std::future::poll_fn(|cx| match send.as_mut().poll(cx) {
        std::task::Poll::Pending => std::task::Poll::Ready(Ok(())),
        std::task::Poll::Ready(result) => std::task::Poll::Ready(Err(format!(
            "unexpected early completion: {result:?}"
        )
        .into())),
    })
    .await
}

pub(crate) async fn observe_body<F>(client: &Client, send: &mut Pin<Box<F>>) -> TestResult
where
    F: Future<Output = Result<Response, TransportError>>,
{
    crate::transport_support::watchdog::watch(std::future::poll_fn(|cx| {
        if let std::task::Poll::Ready(result) = send.as_mut().poll(cx) {
            return std::task::Poll::Ready(Err(format!("early completion: {result:?}").into()));
        }

        if client.statistics().bytes_read >= 41 {
            return std::task::Poll::Ready(Ok(()));
        }
        cx.waker().wake_by_ref();
        std::task::Poll::Pending
    }))
    .await?
}
fn assert_timeout(result: Result<Response, TransportError>) {
    assert!(matches!(result, Err(TransportError::Timeout)));
}

#[tokio::test]
async fn body_timeout_discards_the_incomplete_socket_before_the_next_request() -> TestResult {
    let mut server = Server::start(Mode::MidBodyThenGood, Arc::default()).await?;
    let client = Client::new(crate::capped_config(1))?;
    let clock = crate::transport_support::clock::Clock::pause();
    let mut send = Box::pin(client.send(request(&server.url)?));
    crate::observe_stall(&mut server, &mut send).await?;
    observe_body(&client, &mut send).await?;

    assert_timeout(clock.expire(send, crate::config().request_deadline).await?);
    tokio::time::resume();
    let response =
        crate::transport_support::watchdog::watch(client.send(request(&server.url)?)).await??;
    assert_eq!(response.body.as_ref(), b"ok");
    assert_eq!(count(&server.counts.accepts), 2);
    assert_eq!(client.statistics().connections_opened, 2);
    server.stop().await
}
