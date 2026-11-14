#[path = "transport_cases/body_bounds.rs"]
mod body_bounds;
#[path = "transport_cases/configuration.rs"]
mod configuration;
#[path = "transport_cases/counters.rs"]
mod counters;
#[path = "transport_cases/deadlines.rs"]
mod deadlines;
#[path = "transport_cases/failures.rs"]
mod failures;
#[path = "transport_cases/goaway.rs"]
mod goaway;
#[path = "transport_cases/http2.rs"]
mod http2;
#[path = "transport_cases/idle.rs"]
mod idle;
#[path = "transport_cases/phases.rs"]
mod phases;
#[path = "transport_cases/retirement.rs"]
mod retirement;
#[path = "transport_cases/tls.rs"]
mod tls;
mod transport_support;
#[path = "transport_cases/trust.rs"]
mod trust;

use aws_wire::transport::{Client, TransportError};
use http::StatusCode;
use std::sync::Arc;
use tokio::task::JoinSet;
use transport_support::{Counts, Mode, Server, TestResult, client, config, count, request};

#[tokio::test]
async fn sequential_requests_reuse_one_connection_across_clones() -> TestResult {
    let server = Server::start(Mode::Good, Arc::default()).await?;
    let client = client()?;
    let _clock = transport_support::clock::Clock::pause();

    for _ in 0..8 {
        let response = client.clone().send(request(&server.url)?).await?;
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.body.as_ref(), b"ok");
    }

    assert_eq!(count(&server.counts.accepts), 1);
    assert_eq!(client.statistics().connections_opened, 1);
    assert_eq!(client.statistics().requests_sent, 8);
    assert_eq!(client.statistics().requests_reused, 7);
    server.stop().await
}

#[tokio::test]
async fn oversized_content_length_body_is_rejected_before_collection() -> TestResult {
    oversized(Mode::LargeKnown).await
}

#[tokio::test]
async fn oversized_chunked_body_is_rejected_during_collection() -> TestResult {
    oversized(Mode::LargeChunked).await
}

async fn oversized(mode: Mode) -> TestResult {
    let server = Server::start(mode, Arc::default()).await?;
    let client = Client::new(aws_wire::transport::Config {
        response_body_limit: 1024,
        ..config()
    })?;

    let result = client.send(request(&server.url)?).await;

    assert!(matches!(
        result,
        Err(TransportError::BodyTooLarge { limit: 1024 })
    ));

    server.stop().await
}

#[tokio::test]
async fn silence_before_headers_times_out_at_deadline() -> TestResult {
    stalled(Mode::BeforeHeaders).await
}

#[tokio::test]
async fn silence_mid_body_times_out_at_deadline() -> TestResult {
    stalled(Mode::MidBody).await
}

async fn stalled(mode: Mode) -> TestResult {
    let mut server = Server::start(mode, Arc::default()).await?;
    let client = client()?;
    let clock = transport_support::clock::Clock::pause();
    let mut send = Box::pin(client.send(request(&server.url)?));
    observe_stall(&mut server, &mut send).await?;
    if matches!(mode, Mode::MidBody) {
        deadlines::observe_body(&client, &mut send).await?;
    }

    let result = clock.expire(send, config().request_deadline).await?;

    assert!(matches!(result, Err(TransportError::Timeout)));
    assert_eq!(client.statistics().requests_sent, 1);
    server.stop().await
}

#[tokio::test]
async fn distinct_endpoints_never_exceed_the_global_connection_cap() -> TestResult {
    let counts = Arc::new(Counts::default());
    let servers = start_servers(&counts).await?;
    let client = Client::new(capped_config(2))?;

    let mut requests = send_all(&client, &servers)?;
    complete_requests(&mut requests, &client).await?;

    assert!(count(&counts.peak) <= 2);
    assert_eq!(client.statistics().requests_sent, 4);
    stop_servers(servers).await?;
    Ok(())
}

async fn complete_requests(
    tasks: &mut JoinSet<Result<aws_wire::transport::Response, TransportError>>,
    client: &Client,
) -> TestResult {
    for _ in 0..4 {
        let response = tasks
            .join_next()
            .await
            .ok_or("missing request result")???;
        assert_eq!(response.body.as_ref(), b"ok");
        assert!(client.statistics().connections_open <= 2);
    }
    Ok(())
}

#[tokio::test]
async fn idle_socket_holding_the_last_permit_is_evicted_for_another_host() -> TestResult {
    idle::retention(false).await
}

async fn observe_stall<F>(server: &mut Server, send: &mut std::pin::Pin<Box<F>>) -> TestResult
where
    F: Future<Output = Result<aws_wire::transport::Response, TransportError>>,
{
    transport_support::watchdog::watch(async {
        tokio::select! {
            result = send => Err(format!("completed before stall: {result:?}").into()),
            ready = server.ready.recv() => {
                assert_eq!(ready, Some(()));
                Ok::<(), transport_support::TestError>(())
            }
        }
    })
    .await??;
    Ok(())
}

fn capped_config(cap: usize) -> aws_wire::transport::Config {
    aws_wire::transport::Config {
        max_connections: cap,
        pool_idle_per_host: cap.min(2),
        ..config()
    }
}

async fn start_servers(counts: &Arc<Counts>) -> Result<Vec<Server>, transport_support::TestError> {
    let mut servers = Vec::with_capacity(4);
    for _ in 0..4 {
        servers.push(Server::start(Mode::Close, Arc::clone(counts)).await?);
    }
    Ok(servers)
}

type Requests = JoinSet<Result<aws_wire::transport::Response, TransportError>>;

fn send_all(client: &Client, servers: &[Server]) -> Result<Requests, http::Error> {
    let mut tasks = JoinSet::new();
    for server in servers {
        let client = client.clone();
        let request = request(&server.url)?;
        tasks.spawn(async move { client.send(request).await });
    }
    Ok(tasks)
}

async fn stop_servers(servers: Vec<Server>) -> TestResult {
    for server in servers {
        server.stop().await?;
    }
    Ok(())
}
