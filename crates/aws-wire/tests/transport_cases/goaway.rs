use crate::transport_support::{
    TestResult, frames, request,
    watchdog::watch,
    wire::{self, Endpoint},
};
use aws_wire::transport::{Client, TransportError};

#[tokio::test]
async fn remote_goaway_no_error_excluding_the_new_stream_is_connect() -> TestResult {
    // Last-stream-id zero excludes the independently observed request stream one.
    retirement(b"\0\0\x08\x07\0\0\0\0\0\0\0\0\0\0\0\0\0", true).await
}

#[tokio::test]
async fn remote_refused_stream_is_connect() -> TestResult {
    retirement(b"\0\0\x04\x03\0\0\0\0\x01\0\0\0\x07", true).await
}

#[tokio::test]
async fn remote_protocol_error_is_not_normal_connection_retirement() -> TestResult {
    retirement(b"\0\0\x08\x07\0\0\0\0\0\0\0\0\0\0\0\0\x01", false).await
}

async fn retirement(frame: &[u8], connect: bool) -> TestResult {
    // Given a real TLS/ALPN h2 connection and a known new request stream.
    let endpoint = Endpoint::bind(true, b"h2").await?;
    let client = Client::new(endpoint.config(1))?;
    let mut send = Box::pin(client.send(request(&endpoint.url)?));
    let mut stream = endpoint.accept(&mut send).await?;
    frames::request_headers(&mut send, &mut stream).await?;
    // When the peer sends typed retirement or a real protocol failure.
    wire::write(&mut stream, frame).await?;
    let result = watch(send).await?;
    if connect {
        assert!(matches!(result, Err(TransportError::Connect(_))));
    } else {
        assert!(matches!(result, Err(TransportError::Protocol(_))));
    }
    // Then no internal retry appears as another accepted connection.
    wire::no_accept(&endpoint.listener).await?;
    assert_eq!(client.statistics().requests_sent, 1);
    endpoint.cleanup()?;
    Ok(())
}
