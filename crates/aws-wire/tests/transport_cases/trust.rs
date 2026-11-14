use crate::transport_support::{TestResult, config, tls::TlsServer};
use aws_wire::transport::{Client, Config, TransportError};

#[tokio::test]
async fn empty_extra_ca_bundle_is_not_silently_accepted() -> TestResult {
    let server = TlsServer::start(b"http/1.1").await?;
    std::fs::write(&server.ca, b"")?;

    let config = Config {
        extra_ca_bundle: Some(server.ca.clone()),
        ..config()
    };
    assert!(matches!(
        Client::new(config),
        Err(TransportError::InvalidRequest(_))
    ));
    server.stop().await
}

#[tokio::test]
async fn malformed_pem_is_a_typed_tls_error() -> TestResult {
    let server = TlsServer::start(b"http/1.1").await?;
    std::fs::write(
        &server.ca,
        b"-----BEGIN CERTIFICATE-----\n!\n-----END CERTIFICATE-----\n",
    )?;

    let config = Config {
        extra_ca_bundle: Some(server.ca.clone()),
        ..config()
    };
    assert!(matches!(Client::new(config), Err(TransportError::Tls(_))));
    server.stop().await
}

#[tokio::test]
async fn oversized_ca_bundle_is_rejected_at_the_read_bound() -> TestResult {
    let server = TlsServer::start(b"http/1.1").await?;
    std::fs::write(&server.ca, vec![b'x'; 1_048_577])?;

    let config = Config {
        extra_ca_bundle: Some(server.ca.clone()),
        ..config()
    };
    assert!(matches!(
        Client::new(config),
        Err(TransportError::InvalidRequest(_))
    ));
    server.stop().await
}
