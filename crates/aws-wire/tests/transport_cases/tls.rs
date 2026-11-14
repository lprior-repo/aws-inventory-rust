use crate::transport_support::{TestResult, config, count, request, tls::TlsServer};
use aws_wire::transport::{Client, Config, TransportError};
use http::{StatusCode, Version};

#[tokio::test]
async fn generated_loopback_certificate_requires_the_extra_ca_bundle() -> TestResult {
    // Given a runtime-generated self-signed IP certificate, not in system roots.
    let server = TlsServer::start(b"http/1.1").await?;
    let untrusted = Client::new(config())?;
    // When sent without its CA, verification must fail as Tls, not generic Connect.
    assert!(matches!(
        untrusted.send(request(&server.url)?).await,
        Err(TransportError::Tls(_))
    ));
    let trusted = Client::new(Config {
        extra_ca_bundle: Some(server.ca.clone()),
        ..config()
    })?;
    // Then appending its CA permits a real handshake and complete HTTP response.
    let response = trusted.send(request(&server.url)?).await?;
    assert_eq!(response.status, StatusCode::OK);
    assert_eq!(response.body.as_ref(), b"tls");
    assert_eq!(response.version, Version::HTTP_11);
    assert_eq!(count(&server.accepts), 2);
    assert_eq!(count(&server.rejected), 1);
    server.stop().await
}

#[tokio::test]
async fn alpn_negotiates_http2_and_reuses_one_tls_connection() -> TestResult {
    // Given a real TLS endpoint offering only h2.
    let server = TlsServer::start(b"h2").await?;
    let client = Client::new(Config {
        extra_ca_bundle: Some(server.ca.clone()),
        ..config()
    })?;
    // Reuse is independent of wall-clock TLS packet delays and idle eviction.
    let _clock = crate::transport_support::clock::Clock::pause();
    // When requesting repeatedly, then ALPN selects h2 and the same socket is reused.
    for _ in 0..3 {
        let response = client.send(request(&server.url)?).await?;
        assert_eq!(response.version, Version::HTTP_2);
        assert_eq!(response.body.as_ref(), b"tls");
    }
    assert_eq!(count(&server.accepts), 1);
    assert_eq!(client.statistics().connections_opened, 1);
    assert_eq!(client.statistics().requests_reused, 2);
    server.stop().await
}

#[tokio::test]
async fn http1_tls_fallback_keeps_the_connection_alive() -> TestResult {
    // Given a TLS endpoint offering only HTTP/1.1.
    let server = TlsServer::start(b"http/1.1").await?;
    let client = Client::new(Config {
        extra_ca_bundle: Some(server.ca.clone()),
        ..config()
    })?;
    let _clock = crate::transport_support::clock::Clock::pause();
    // When requesting repeatedly, then HTTP/1.1 is reused without reconnecting.
    for _ in 0..3 {
        let response = client.send(request(&server.url)?).await?;
        assert_eq!(response.version, Version::HTTP_11);
        assert_eq!(response.body.as_ref(), b"tls");
    }
    assert_eq!(count(&server.accepts), 1);
    assert_eq!(client.statistics().connections_opened, 1);
    server.stop().await
}

#[tokio::test]
async fn dropping_an_http2_request_preserves_other_streams_and_pool_reuse() -> TestResult {
    // Given one blocked stream on a real multiplexed TLS connection.
    let server = TlsServer::start(b"h2").await?;
    let client = Client::new(Config {
        extra_ca_bundle: Some(server.ca.clone()),
        ..config()
    })?;
    let _clock = crate::transport_support::clock::Clock::pause();
    let mut stalled = Box::pin(client.send(request(&format!("{}stall", server.url))?));
    observe_http2_stall(&server, &mut stalled).await?;
    // When another stream succeeds and the blocked request is cancelled.
    assert_eq!(
        client.send(request(&server.url)?).await?.version,
        Version::HTTP_2
    );
    drop(stalled);
    // Then subsequent streams still use the original, unpoisoned connection.
    assert_eq!(
        client.send(request(&server.url)?).await?.body.as_ref(),
        b"tls"
    );
    assert_eq!(count(&server.accepts), 1);
    assert_eq!(client.statistics().connections_opened, 1);
    server.stop().await
}

async fn observe_http2_stall<F>(server: &TlsServer, send: &mut std::pin::Pin<Box<F>>) -> TestResult
where
    F: Future<Output = Result<aws_wire::transport::Response, TransportError>>,
{
    crate::transport_support::wire::until(send, server.stalled.notified()).await
}

#[tokio::test]
async fn a_trusted_certificate_with_the_wrong_san_is_rejected() -> TestResult {
    // Given a trusted certificate that names a different host than the actual request.
    let server = TlsServer::start_named(b"http/1.1", "wrong.example").await?;
    let client = Client::new(Config {
        extra_ca_bundle: Some(server.ca.clone()),
        ..config()
    })?;
    // When requesting 127.0.0.1, normal chain trust must not bypass hostname verification.
    let result =
        crate::transport_support::watchdog::watch(client.send(request(&server.url)?)).await?;
    assert!(matches!(result, Err(TransportError::Tls(_))));
    assert_eq!(count(&server.accepts), 1);
    server.stop().await
}
