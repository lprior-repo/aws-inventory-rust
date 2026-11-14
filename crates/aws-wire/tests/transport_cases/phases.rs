use crate::transport_support::{
    TestResult,
    clock::Clock,
    config, request,
    watchdog::watch,
    wire::{self, Endpoint},
};
use aws_wire::transport::{Client, Config, TransportError};
use bytes::Bytes;
use std::time::Duration;
use tokio::{
    io::AsyncReadExt,
    net::{TcpSocket, TcpStream},
};

#[tokio::test]
async fn configured_deadline_is_cumulative_across_headers_and_body() -> TestResult {
    // Given a non-default 500 ms whole-request deadline, not two phase-local deadlines.
    let endpoint = Endpoint::bind(false, b"").await?;
    let client = Client::new(short(&endpoint))?;
    let clock = Clock::pause();
    let mut send = Box::pin(client.send(request(&endpoint.url)?));
    let mut stream = endpoint.accept(&mut send).await?;
    wire::until(&mut send, wire::head(&mut stream)).await??;
    // When 300 ms is spent before headers and the rest waiting for the remaining body.
    tokio::time::advance(Duration::from_millis(300)).await;
    crate::deadlines::poll_pending(&mut send).await?;
    let partial = b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\nx";
    wire::write(&mut stream, partial).await?;
    wire::read_at_least(&client, &mut send, partial.len()).await?;
    expired(clock.expire(send, Duration::from_millis(500)).await?);
    assert_eq!(client.statistics().timeouts, 1);
    endpoint.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn tls_handshake_stall_obeys_the_independent_connector_bound() -> TestResult {
    // Given a TCP peer that reads ClientHello but never answers TLS.
    let endpoint = Endpoint::bind(true, b"http/1.1").await?;
    let client = Client::new(short(&endpoint))?;
    let clock = Clock::pause();
    let mut send = Box::pin(client.send(request(&endpoint.url)?));
    let (mut stream, _) = wire::until(&mut send, endpoint.listener.accept()).await??;
    let mut hello = [0; 5];
    wire::until(&mut send, stream.read_exact(&mut hello)).await??;
    assert_eq!(client.statistics().connections_open, 1);
    // Then the 100 ms connector bound, not the 500 ms foreground bound, expires.
    expired(clock.expire(send, Duration::from_millis(100)).await?);
    assert_eq!(client.statistics().connections_open, 0);
    assert_eq!(client.statistics().timeouts, 1);
    endpoint.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn blocked_request_write_is_covered_by_the_whole_request_deadline() -> TestResult {
    // Given a small receive window and a body much larger than socket buffers.
    let endpoint = Endpoint::bind(false, b"").await?;
    let client = Client::new(short(&endpoint))?;
    let request = http::Request::builder()
        .method("PUT")
        .uri(&endpoint.url)
        .body(Bytes::from(vec![0; 33_554_432]))?;
    let clock = Clock::pause();
    let mut send = Box::pin(client.send(request));
    let mut stream = endpoint.accept(&mut send).await?;
    let head = wire::until(&mut send, wire::head(&mut stream)).await??;
    assert!(std::str::from_utf8(&head)?.contains("33554432"));
    // When the peer stops reading after headers, then writing cannot escape the deadline.
    expired(clock.expire(send, Duration::from_millis(500)).await?);
    assert_eq!(client.statistics().bytes_read, 0);
    assert_eq!(client.statistics().timeouts, 1);
    endpoint.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn stalled_tcp_connect_releases_admission_at_the_connector_bound() -> TestResult {
    // Given Linux's bounded listen backlog filled without accepting either connection.
    let socket = TcpSocket::new_v4()?;
    socket.bind("127.0.0.1:0".parse()?)?;
    let listener = socket.listen(1)?;
    let address = listener.local_addr()?;
    let _first = watch(TcpStream::connect(address)).await??;
    let _second = watch(TcpStream::connect(address)).await??;
    let client = Client::new(connect_bound())?;
    let url = format!("http://{address}/");
    let clock = Clock::pause();
    let mut send = Box::pin(client.send(request(&url)?));
    crate::deadlines::poll_pending(&mut send).await?;
    // Then the still-unestablished TCP attempt times out, without opening a counted socket.
    expired(clock.expire(send, Duration::from_millis(100)).await?);
    assert_eq!(client.statistics().connections_opened, 0);
    assert_eq!(client.statistics().timeouts, 1);
    Ok(())
}

fn short(endpoint: &Endpoint) -> Config {
    Config {
        request_deadline: Duration::from_millis(500),
        connect_timeout: Duration::from_millis(100),
        ..endpoint.config(1)
    }
}

fn connect_bound() -> Config {
    Config {
        connect_timeout: Duration::from_millis(100),
        ..config()
    }
}

fn expired(result: wire::SendResult) {
    assert!(matches!(result, Err(TransportError::Timeout)));
}
