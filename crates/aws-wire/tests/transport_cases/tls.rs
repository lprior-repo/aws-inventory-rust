use crate::transport_support::{TestResult, config, count, request, tls::TlsServer};
use aws_wire::transport::{Client, Config, TransportError};
use http::{StatusCode, Version};

#[tokio::test]
async fn generated_loopback_certificate_requires_the_extra_ca_bundle() -> TestResult {
    let server = TlsServer::start(b"http/1.1").await?;
    let untrusted = Client::new(config())?;

    assert!(matches!(
        untrusted.send(request(&server.url)?).await,
        Err(TransportError::Tls(_))
    ));
    let trusted = Client::new(Config {
        extra_ca_bundle: Some(server.ca.clone()),
        ..config()
    })?;

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
    let server = TlsServer::start(b"h2").await?;
    let client = Client::new(Config {
        extra_ca_bundle: Some(server.ca.clone()),
        ..config()
    })?;

    let _clock = crate::transport_support::clock::Clock::pause();

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
    let server = TlsServer::start(b"http/1.1").await?;
    let client = Client::new(Config {
        extra_ca_bundle: Some(server.ca.clone()),
        ..config()
    })?;
    let _clock = crate::transport_support::clock::Clock::pause();

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
    let server = TlsServer::start(b"h2").await?;
    let client = Client::new(Config {
        extra_ca_bundle: Some(server.ca.clone()),
        ..config()
    })?;
    let _clock = crate::transport_support::clock::Clock::pause();
    let mut stalled = Box::pin(client.send(request(&format!("{}stall", server.url))?));
    observe_http2_stall(&server, &mut stalled).await?;

    assert_eq!(
        client.send(request(&server.url)?).await?.version,
        Version::HTTP_2
    );
    drop(stalled);

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
    let server = TlsServer::start_named(b"http/1.1", "wrong.example").await?;
    let client = Client::new(Config {
        extra_ca_bundle: Some(server.ca.clone()),
        ..config()
    })?;

    let result =
        crate::transport_support::watchdog::watch(client.send(request(&server.url)?)).await?;
    assert!(matches!(result, Err(TransportError::Tls(_))));
    assert_eq!(count(&server.accepts), 1);
    server.stop().await
}
