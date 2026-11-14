use super::support::{OK, Server, TestResult, UNSIGNED, exchange, field, request};
use inventory_harness::proxy::{MAX_BODY_BYTES, MAX_HEAD_BYTES, Proxy, Scope};

#[tokio::test]
async fn keep_alive_forwards_two_requests_on_one_client_and_upstream_connection() -> TestResult {
    super::reuse::verify().await
}

#[tokio::test]
async fn chunked_response_preserves_chunk_extensions_payload_and_trailers() -> TestResult {
    let response = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nTrailer: Checksum\r\n\r\n4;test=yes\r\nWiki\r\n5\r\npedia\r\n0\r\nChecksum: yes\r\n\r\n";
    let server = Server::start(response).await?;
    let proxy = Proxy::start(server.config()).await?;

    assert_eq!(exchange(proxy.address(), UNSIGNED).await?, response);
    super::support::shutdown(proxy).await?;
    Ok(())
}

#[tokio::test]
async fn close_delimited_response_is_forwarded_until_eof() -> TestResult {
    let response = b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nclose delimited payload";
    let server = Server::start(response).await?;
    let proxy = Proxy::start(server.config()).await?;

    assert_eq!(exchange(proxy.address(), UNSIGNED).await?, response);
    super::support::shutdown(proxy).await?;
    Ok(())
}

#[tokio::test]
async fn signed_host_authorization_and_request_body_are_forwarded_unchanged() -> TestResult {
    let mut server = Server::start(OK).await?;
    let proxy = Proxy::start(server.config()).await?;
    let bytes = signed_request_body()?;
    assert_eq!(exchange(proxy.address(), &bytes).await?, OK);

    assert_eq!(super::support::captured(&mut server).await?, bytes);
    assert_eq!(
        proxy
            .snapshot()?
            .decisions
            .first()
            .and_then(|e| signed_account(&e.scope)),
        Some("key")
    );
    super::support::shutdown(proxy).await?;
    Ok(())
}

#[tokio::test]
async fn oversized_head_is_rejected_before_upstream_contact() -> TestResult {
    let server = Server::start(OK).await?;
    let proxy = Proxy::start(server.config()).await?;
    let bytes = [
        b"GET / HTTP/1.1\r\nHost: example\r\nX-Large: ".as_slice(),
        vec![b'x'; MAX_HEAD_BYTES].as_slice(),
        b"\r\n\r\n",
    ]
    .concat();
    let response = exchange(proxy.address(), &bytes).await?;

    assert!(response.starts_with(b"HTTP/1.1 431 "));
    assert_eq!(field(&response, "x-proxy-error")?, "request-head");
    assert_eq!(proxy.snapshot()?.counters.forwarded, 0);
    super::support::shutdown(proxy).await?;
    Ok(())
}

#[tokio::test]
async fn oversized_declared_request_body_is_rejected_without_reading_it() -> TestResult {
    let server = Server::start(OK).await?;
    let proxy = Proxy::start(server.config()).await?;
    let length = MAX_BODY_BYTES.checked_add(1).ok_or("length overflow")?;
    let request = format!("POST / HTTP/1.1\r\nHost: example\r\nContent-Length: {length}\r\n\r\n");
    let response = exchange(proxy.address(), request.as_bytes()).await?;

    assert!(response.starts_with(b"HTTP/1.1 413 "));
    assert_eq!(field(&response, "x-proxy-error")?, "request-body");
    assert_eq!(proxy.snapshot()?.counters.forwarded, 0);
    super::support::shutdown(proxy).await?;
    Ok(())
}

#[tokio::test]
async fn chunked_request_is_rejected_with_501_and_never_forwarded() -> TestResult {
    let server = Server::start(OK).await?;
    let proxy = Proxy::start(server.config()).await?;
    let response = exchange(
        proxy.address(),
        b"POST / HTTP/1.1\r\nHost: example\r\nTransfer-Encoding: chunked\r\n\r\n",
    )
    .await?;

    assert!(response.starts_with(b"HTTP/1.1 501 "));
    assert_eq!(field(&response, "x-proxy-error")?, "chunked-request");
    assert_eq!(proxy.snapshot()?.counters.forwarded, 0);
    super::support::shutdown(proxy).await?;
    Ok(())
}

fn signed_request_body() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let body = b"exact\0payload\xff";
    let head = request("key", "us-east-1", "s3");
    let text = std::str::from_utf8(&head)?
        .strip_suffix("\r\n")
        .ok_or("request suffix")?;
    Ok([
        format!("{text}Content-Length: {}\r\n\r\n", body.len()).as_bytes(),
        body,
    ]
    .concat())
}

fn signed_account(scope: &Scope) -> Option<&str> {
    match scope {
        Scope::Signed { account, .. } => Some(account.as_str()),
        Scope::Unsigned => None,
    }
}
