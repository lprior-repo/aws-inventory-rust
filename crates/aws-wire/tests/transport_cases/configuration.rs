use crate::transport_support::{TestResult, config, request};
use aws_wire::transport::{Client, Config, TransportError};
use std::time::Duration;

#[test]
fn invalid_connection_caps_are_rejected_before_pool_construction() {
    // Given out-of-range caps, when constructing, then each is rejected.
    for max_connections in [0, 513] {
        invalid(Config {
            max_connections,
            ..config()
        });
    }
    for pool_idle_per_host in [0, 129] {
        invalid(Config {
            pool_idle_per_host,
            ..config()
        });
    }
}

#[test]
fn invalid_body_limits_are_rejected_before_pool_construction() {
    // Given zero or over-hard-maximum buffers, when constructing, then reject.
    for response_body_limit in [0, 67_108_865] {
        invalid(Config {
            response_body_limit,
            ..config()
        });
    }
}

#[test]
fn invalid_deadlines_are_rejected_before_pool_construction() {
    // Given deadlines outside the documented range, when constructing, then reject.
    for request_deadline in [Duration::ZERO, Duration::from_secs(301)] {
        invalid(Config {
            request_deadline,
            ..config()
        });
    }
}

#[test]
fn invalid_idle_timeouts_are_rejected_before_pool_construction() {
    // Given zero or starvation-prone idle times, when constructing, then reject.
    for idle_timeout in [Duration::ZERO, Duration::from_secs(2)] {
        invalid(Config {
            idle_timeout,
            ..config()
        });
    }
}

fn invalid(config: Config) {
    assert!(matches!(
        Client::new(config),
        Err(TransportError::InvalidRequest(_))
    ));
}

#[tokio::test]
async fn invalid_uri_schemes_are_rejected_before_connecting() -> TestResult {
    // Given a pool accepting plain HTTP for loopback only.
    let client = Client::new(config())?;
    // When URI schemes or tunnel semantics are unsupported, then no socket opens.
    for uri in ["/relative", "ftp://127.0.0.1/a"] {
        assert!(matches!(
            client.send(request(uri)?).await,
            Err(TransportError::InvalidRequest(_))
        ));
    }
    assert_eq!(client.statistics().connections_opened, 0);
    assert_eq!(client.statistics().requests_sent, 0);
    Ok(())
}

#[tokio::test]
async fn tunnel_methods_are_rejected_before_connecting() -> TestResult {
    // Given a tunnel request, when sending, then reject before any socket opens.
    let client = Client::new(config())?;
    let tunnel = http::Request::builder()
        .method("CONNECT")
        .uri("http://127.0.0.1/")
        .body(bytes::Bytes::new())?;
    assert!(matches!(
        client.send(tunnel).await,
        Err(TransportError::InvalidRequest(_))
    ));
    assert_eq!(client.statistics().connections_opened, 0);
    assert_eq!(client.statistics().requests_sent, 0);
    Ok(())
}

#[tokio::test]
async fn body_exactly_at_the_limit_is_returned_intact() -> TestResult {
    // Given a two-byte response and an exactly two-byte limit.
    let server = crate::transport_support::Server::start(
        crate::transport_support::Mode::Good,
        std::sync::Arc::default(),
    )
    .await?;
    let client = Client::new(Config {
        response_body_limit: 2,
        ..config()
    })?;
    // When collected, then the inclusive limit preserves the entire body.
    let response = client.send(request(&server.url)?).await?;
    assert_eq!(response.body.as_ref(), b"ok");
    assert_eq!(response.status, http::StatusCode::OK);
    server.stop().await
}

#[test]
fn inaccessible_extra_ca_bundle_is_a_typed_io_error() -> TestResult {
    // Given a path that cannot name a file because a NUL is forbidden by the OS.
    let config = Config {
        extra_ca_bundle: Some(std::path::PathBuf::from("invalid\0pem")),
        ..config()
    };
    // When roots load, then filesystem errors remain Io rather than generic TLS text.
    assert!(matches!(Client::new(config), Err(TransportError::Io(_))));
    Ok(())
}

#[tokio::test]
async fn smallest_configured_bounds_construct_an_empty_pool() -> TestResult {
    // Given the documented minima, when constructing, then all counters start at zero.
    let config = Config {
        request_deadline: Duration::from_millis(2),
        connect_timeout: Duration::from_millis(2),
        idle_timeout: Duration::from_millis(1),
        response_body_limit: 1,
        max_connections: 1,
        pool_idle_per_host: 1,
        ..config()
    };
    assert_eq!(
        Client::new(config)?.statistics(),
        aws_wire::transport::Statistics::default()
    );
    Ok(())
}

#[tokio::test]
async fn largest_configured_bounds_construct_an_empty_pool() -> TestResult {
    // Given the hard maxima, when constructing, then no socket opens eagerly.
    let config = Config {
        request_deadline: Duration::from_secs(300),
        idle_timeout: Duration::from_secs(150),
        response_body_limit: 67_108_864,
        max_connections: 512,
        pool_idle_per_host: 512,
        ..config()
    };
    assert_eq!(
        Client::new(config)?.statistics(),
        aws_wire::transport::Statistics::default()
    );
    Ok(())
}

#[test]
fn connector_timeout_must_be_positive_and_within_the_request_deadline() {
    // Given zero or an excessive connector bound, construction must fail.
    for connect_timeout in [Duration::ZERO, Duration::from_secs(3)] {
        invalid(Config {
            connect_timeout,
            ..config()
        });
    }
}
