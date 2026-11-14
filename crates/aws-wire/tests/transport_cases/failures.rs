use crate::transport_support::wire::{self, Endpoint};
use crate::transport_support::{Mode, Server, TestResult, client, count, request};
use aws_wire::transport::{Client, Config, TransportError};
use http::StatusCode;
use std::sync::Arc;
use tokio::net::TcpSocket;

#[tokio::test]
async fn refused_connection_returns_connect_without_retry() -> TestResult {
    // Given a retained bound, non-listening socket: its port cannot be stolen by another test.
    let socket = TcpSocket::new_v4()?;
    socket.bind("127.0.0.1:0".parse()?)?;
    let url = format!("http://{}/", socket.local_addr()?);
    let client = client()?;
    // When connecting once, then refusal remains classifiable.
    assert!(matches!(
        client.send(request(&url)?).await,
        Err(TransportError::Connect(_))
    ));
    assert_eq!(client.statistics().requests_sent, 1);
    assert_eq!(client.statistics().connections_open, 0);
    Ok(())
}

#[tokio::test]
async fn peer_reset_returns_connect_without_retry() -> TestResult {
    // Given a socket that resets after receiving the complete request.
    let server = Server::start(Mode::Reset, Arc::default()).await?;
    let client = client()?;
    // When sending, then reset is Connect and the server sees exactly one attempt.
    assert!(matches!(
        client.send(request(&server.url)?).await,
        Err(TransportError::Connect(_))
    ));
    assert_eq!(count(&server.counts.accepts), 1);
    server.stop().await
}

#[tokio::test]
async fn plain_http_is_rejected_unless_explicitly_allowed() -> TestResult {
    // Given a real endpoint and secure-by-default configuration.
    let server = Server::start(Mode::Good, Arc::default()).await?;
    let secure = Client::new(Config::default())?;
    // When plain HTTP is sent, then rejection happens before opening a socket.
    assert!(matches!(
        secure.send(request(&server.url)?).await,
        Err(TransportError::InvalidRequest(_))
    ));
    assert_eq!(count(&server.counts.accepts), 0);
    assert_eq!(
        client()?.send(request(&server.url)?).await?.body.as_ref(),
        b"ok"
    );
    server.stop().await
}

#[tokio::test]
async fn redirect_is_returned_unchanged_and_never_followed() -> TestResult {
    // Given a live redirect destination with independently counted accepts.
    let destination = Server::start(Mode::Good, Arc::default()).await?;
    let origin = Endpoint::bind(false, b"").await?;
    let client = client()?;
    let mut send = Box::pin(client.send(request(&origin.url)?));
    let mut stream = origin.accept(&mut send).await?;
    let head = format!(
        "HTTP/1.1 302 Found\r\nLocation: {}\r\nContent-Length: 0\r\n\r\n",
        destination.url,
    );
    let response = wire::complete(send, &mut stream, head.as_bytes()).await??;
    assert_eq!(response.status, StatusCode::FOUND);
    assert_eq!(
        response.headers.get("location"),
        Some(&http::HeaderValue::from_str(&destination.url)?)
    );
    assert_eq!(response.body.len(), 0);
    assert_eq!(count(&destination.counts.accepts), 0);
    origin.cleanup()?;
    destination.stop().await
}

#[tokio::test]
async fn invalid_http_response_is_a_protocol_error() -> TestResult {
    // Given a peer speaking invalid HTTP, when sending once, then classify Protocol.
    let server = Server::start(Mode::BadProtocol, Arc::default()).await?;
    assert!(matches!(
        client()?.send(request(&server.url)?).await,
        Err(TransportError::Protocol(_))
    ));
    assert_eq!(count(&server.counts.accepts), 1);
    server.stop().await
}

#[tokio::test]
async fn dropping_a_mid_body_send_does_not_poison_the_next_request() -> TestResult {
    // Given the first socket stalls mid-body and the next socket serves normally.
    let mut server = Server::start(Mode::MidBodyThenGood, Arc::default()).await?;
    let client = client()?;
    let mut send = Box::pin(client.send(request(&server.url)?));
    crate::observe_stall(&mut server, &mut send).await?;
    crate::deadlines::observe_body(&client, &mut send).await?;
    // When cancellation drops the future, incomplete bytes must not be reused.
    drop(send);
    let response = client.send(request(&server.url)?).await?;
    // Then a clean socket yields exactly the expected body.
    assert_eq!(response.body.as_ref(), b"ok");
    assert_eq!(count(&server.counts.accepts), 2);
    assert_eq!(client.statistics().connections_opened, 2);
    server.stop().await
}
