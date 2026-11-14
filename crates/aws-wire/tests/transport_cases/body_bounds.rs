use crate::transport_support::{
    TestResult, request,
    watchdog::watch,
    wire::{self, Endpoint},
};
use aws_wire::transport::{Client, Config, TransportError};
use bytes::Bytes;

#[tokio::test]
async fn content_length_exact_limit_is_complete() -> TestResult {
    boundary(false, "xx").await
}
#[tokio::test]
async fn content_length_limit_plus_one_is_rejected() -> TestResult {
    boundary(false, "xxx").await
}
#[tokio::test]
async fn chunked_exact_limit_is_complete() -> TestResult {
    boundary(true, "xx").await
}
#[tokio::test]
async fn chunked_limit_plus_one_is_rejected() -> TestResult {
    boundary(true, "xxx").await
}

async fn boundary(chunked: bool, offered: &str) -> TestResult {
    // Given a complete response at the inclusive bound or exactly one byte above it.
    let endpoint = Endpoint::bind(false, b"").await?;
    let client = Client::new(Config {
        response_body_limit: 2,
        ..endpoint.config(1)
    })?;
    let mut send = Box::pin(client.send(request(&endpoint.url)?));
    let mut stream = endpoint.accept(&mut send).await?;
    let response = encoded(chunked, offered);
    // When every offered byte and the chunk terminator are sent.
    let result = wire::complete(send, &mut stream, response.as_bytes()).await?;
    // Then equality is accepted and plus one fails with the exact configured limit.
    if offered.len() == 2 {
        assert_eq!(result?.body.as_ref(), offered.as_bytes());
    } else {
        assert!(matches!(
            result,
            Err(TransportError::BodyTooLarge { limit: 2 })
        ));
    }
    endpoint.cleanup()?;
    Ok(())
}

fn encoded(chunked: bool, body: &str) -> String {
    if chunked {
        return format!(
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n{body}\r\n0\r\n\r\n",
            body.len()
        );
    }
    format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )
}

#[tokio::test]
async fn oversized_advertised_body_is_rejected_without_a_single_body_byte() -> TestResult {
    // Given a peer sending only oversized Content-Length headers, never a body.
    let endpoint = Endpoint::bind(false, b"").await?;
    let client = Client::new(Config {
        response_body_limit: 1024,
        ..endpoint.config(1)
    })?;
    let mut send = Box::pin(client.send(request(&endpoint.url)?));
    let mut stream = endpoint.accept(&mut send).await?;
    let result = wire::complete(
        send,
        &mut stream,
        b"HTTP/1.1 200 OK\r\nContent-Length: 1048576\r\n\r\n",
    )
    .await?;
    // Then known length fails immediately rather than waiting for DATA until Timeout.
    assert!(matches!(
        result,
        Err(TransportError::BodyTooLarge { limit: 1024 })
    ));
    assert_eq!(client.statistics().timeouts, 0);
    endpoint.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn head_does_not_treat_representation_length_as_response_body() -> TestResult {
    // Given HEAD with a representation longer than the body bound.
    let endpoint = Endpoint::bind(false, b"").await?;
    let client = Client::new(Config {
        response_body_limit: 1,
        ..endpoint.config(1)
    })?;
    let request = http::Request::builder()
        .method("HEAD")
        .uri(&endpoint.url)
        .body(Bytes::new())?;
    let mut send = Box::pin(client.send(request));
    let mut stream = endpoint.accept(&mut send).await?;
    let response = wire::complete(
        send,
        &mut stream,
        b"HTTP/1.1 200 OK\r\nContent-Length: 1048576\r\n\r\n",
    )
    .await??;
    // Then the protocol's actual zero-length body is returned unchanged.
    assert_eq!(response.body.len(), 0);
    endpoint.cleanup()?;
    Ok(())
}

#[tokio::test]
async fn bodiless_statuses_ignore_representation_length() -> TestResult {
    // Given 204/304 statuses with oversized length metadata, no DATA exists.
    for status in [204, 304] {
        let endpoint = Endpoint::bind(false, b"").await?;
        let client = Client::new(Config {
            response_body_limit: 1,
            ..endpoint.config(1)
        })?;
        let mut send = Box::pin(client.send(request(&endpoint.url)?));
        let mut stream = endpoint.accept(&mut send).await?;
        let response = format!("HTTP/1.1 {status} Empty\r\nContent-Length: 1048576\r\n\r\n");
        let result = watch(wire::complete(send, &mut stream, response.as_bytes())).await???;
        assert_eq!(result.status.as_u16(), status);
        assert_eq!(result.body.len(), 0);
        endpoint.cleanup()?;
    }
    Ok(())
}
