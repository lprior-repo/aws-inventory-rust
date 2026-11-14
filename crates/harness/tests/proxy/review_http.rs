use super::{
    clock::{FrozenClock, within},
    support::{OK, Server, TestResult, UNSIGNED, exchange, field},
    wire,
};
use inventory_harness::proxy::{Fault, Proxy};
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};

#[tokio::test]
async fn mixed_and_bare_lf_heads_reject_hidden_requests_before_forwarding() -> TestResult {
    for request in [
        b"GET /first HTTP/1.1\r\nHost: x\n\nGET /hidden HTTP/1.1\r\nHost: x\r\n\r\n".as_slice(),
        b"GET /first HTTP/1.1\nHost: x\n\n",
    ] {
        rejected_request(request, 400, "invalid-request").await?;
    }
    Ok(())
}

async fn rejected_request(request: &[u8], status: u16, code: &str) -> TestResult {
    let server = Server::start(OK).await?;
    let proxy = Proxy::start(server.config()).await?;
    let bytes = exchange(proxy.address(), request).await?;
    assert!(bytes.starts_with(format!("HTTP/1.1 {status} ").as_bytes()));
    assert_eq!(field(&bytes, "x-proxy-error")?, code);
    assert_eq!(field(&bytes, "connection")?, "close");
    assert_eq!(proxy.snapshot()?.counters.forwarded, 0);
    within("rejected request shutdown", proxy.shutdown()).await?;
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn expect_continue_is_rejected_before_waiting_for_its_body() -> TestResult {
    let _clock = FrozenClock::new();

    rejected_request(
        b"POST / HTTP/1.1\r\nHost: x\r\nExpect: 100-continue\r\nContent-Length: 12\r\n\r\n",
        417,
        "expectation-failed",
    )
    .await
}

#[tokio::test]
async fn ambiguous_upstream_head_is_rejected_as_bad_gateway() -> TestResult {
    rejected_response(b"HTTP/1.1 200 OK\r\nHost: x\n\nHTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
        .await
}

async fn rejected_response(response: &[u8]) -> TestResult {
    let server = Server::start(response).await?;
    let proxy = Proxy::start(server.config()).await?;
    let bytes = exchange(proxy.address(), UNSIGNED).await?;
    assert!(bytes.starts_with(b"HTTP/1.1 502 "));
    assert_eq!(field(&bytes, "x-proxy-error")?, "upstream-framing");
    assert_eq!(field(&bytes, "connection")?, "close");
    assert_eq!(super::support::split(&bytes)?.1, b"");
    within("bad gateway shutdown", proxy.shutdown()).await?;
    Ok(())
}

#[tokio::test]
async fn only_final_chunked_selects_chunk_framing() -> TestResult {
    for response in [
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked, gzip\r\nConnection: close\r\n\r\nopaque payload".as_slice(),
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip\r\nConnection: close\r\n\r\nopaque payload",
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip, chunked\r\nConnection: close\r\n\r\n3\r\nabc\r\n0\r\n\r\n",
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip\r\nTransfer-Encoding: CHUNKED\r\nConnection: close\r\n\r\n3\r\nabc\r\n0\r\n\r\n",
    ] {
        preserved_response(response).await?;
    }
    Ok(())
}

async fn preserved_response(response: &[u8]) -> TestResult {
    let server = Server::start(response).await?;
    let proxy = Proxy::start(server.config()).await?;
    assert_eq!(exchange(proxy.address(), UNSIGNED).await?, response);
    within("forwarded coding shutdown", proxy.shutdown()).await?;
    Ok(())
}

#[tokio::test]
async fn repeated_chunked_and_transfer_encoding_with_length_return_closed_502() -> TestResult {
    for response in [
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked, chunked\r\n\r\n0\r\n\r\n".as_slice(),
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n",
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Length: 5\r\n\r\n0\r\n\r\n",
    ] {
        rejected_response(response).await?;
    }
    Ok(())
}

#[tokio::test]
async fn canned_head_omits_payload_and_preserves_next_keep_alive_response() -> TestResult {
    within("HEAD followed by GET", head_then_get()).await
}

async fn head_then_get() -> TestResult {
    let server = Server::start(OK).await?;
    let proxy = Proxy::start(server.fault(Fault::Throttle)).await?;
    let mut reader = BufReader::new(TcpStream::connect(proxy.address()).await?);
    reader
        .get_mut()
        .write_all(b"HEAD / HTTP/1.1\r\nHost: x\r\n\r\n")
        .await?;
    let head = wire::head(&mut reader).await?;
    check_head(&head)?;
    reader.get_mut().write_all(UNSIGNED).await?;
    assert_eq!(wire::remaining(&mut reader).await?, OK);
    assert_eq!(proxy.snapshot()?.counters.requests, 2);
    proxy.shutdown().await?;
    Ok(())
}

fn check_head(head: &[u8]) -> TestResult {
    assert!(head.starts_with(b"HTTP/1.1 400 "));
    let representation =
        br#"{"__type":"ThrottlingException","message":"Injected by deterministic proxy"}"#;
    assert_eq!(
        field(head, "content-length")?.parse::<usize>()?,
        representation.len()
    );
    Ok(())
}
