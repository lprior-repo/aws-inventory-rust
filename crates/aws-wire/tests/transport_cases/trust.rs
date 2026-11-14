use crate::transport_support::{TestResult, config, tls::TlsServer};
use aws_wire::transport::{Client, Config, TransportError};

#[tokio::test]
async fn empty_extra_ca_bundle_is_not_silently_accepted() -> TestResult {
    // Given an empty extra trust bundle.
    let server = TlsServer::start(b"http/1.1").await?;
    std::fs::write(&server.ca, b"")?;
    // When constructing, then missing certificates are an explicit invalid request.
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
    // Given a certificate section with invalid base64.
    let server = TlsServer::start(b"http/1.1").await?;
    std::fs::write(
        &server.ca,
        b"-----BEGIN CERTIFICATE-----\n!\n-----END CERTIFICATE-----\n",
    )?;
    // When constructing, then parsing fails explicitly rather than losing the CA.
    let config = Config {
        extra_ca_bundle: Some(server.ca.clone()),
        ..config()
    };
    assert!(matches!(Client::new(config), Err(TransportError::Tls(_))));
    server.stop().await
}

#[tokio::test]
async fn oversized_ca_bundle_is_rejected_at_the_read_bound() -> TestResult {
    // Given a bundle one byte over the hard 1 MiB initialization limit.
    let server = TlsServer::start(b"http/1.1").await?;
    std::fs::write(&server.ca, vec![b'x'; 1_048_577])?;
    // When constructing, then no unbounded PEM allocation/parser work is admitted.
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
