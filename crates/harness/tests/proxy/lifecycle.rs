use super::support::{OK, Server, TestResult, UNSIGNED, reached};
use inventory_harness::proxy::{Fault, MAX_CONNECTIONS, Probability, Proxy, Rule, ScopeFilter};
use std::io;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

fn stalled(
    server: &Server,
) -> Result<inventory_harness::proxy::Config, Box<dyn std::error::Error>> {
    let mut config = server.config();
    config.rules.push(Rule {
        filter: ScopeFilter::default(),
        probability: Probability::new(1_000_000)?,
        fault: Fault::Stall,
    });
    Ok(config)
}

#[tokio::test]
async fn shutdown_cancels_stalled_region_and_report_accounts_for_every_admitted_socket()
-> TestResult {
    // Given three consumed stalled requests owned by the proxy region.
    let server = Server::start(OK).await?;
    let proxy = Proxy::start(stalled(&server)?).await?;
    let address = proxy.address();
    const STALLED_CLIENTS: usize = 3;
    let _clients = stalled_clients(&proxy, STALLED_CLIENTS).await?;
    reached(&proxy, u64::try_from(STALLED_CLIENTS)?).await?;
    // When shutdown completes, then all three are classified and intake is closed.
    let report = super::support::shutdown(proxy).await?;
    super::review_lifecycle::assert_report(&report, (3, 0, 0, 3));
    assert_eq!(
        TcpStream::connect(address).await.err().map(|e| e.kind()),
        Some(io::ErrorKind::ConnectionRefused)
    );
    Ok(())
}

#[tokio::test]
async fn dropping_handle_aborts_owner_and_children_and_releases_listening_port() -> TestResult {
    // Given a live stalled session, when the only handle is dropped.
    let server = Server::start(OK).await?;
    let proxy = Proxy::start(stalled(&server)?).await?;
    let address = proxy.address();
    let mut client = TcpStream::connect(address).await?;
    client.write_all(UNSIGNED).await?;
    reached(&proxy, 1).await?;
    drop(proxy);
    use tokio::io::AsyncReadExt;
    assert_eq!(
        super::clock::within("drop closes admitted client", client.read(&mut [0])).await?,
        0
    );
    // Then abort-on-drop released the listener without an explicit shutdown.
    assert_eq!(
        TcpStream::connect(address).await.err().map(|e| e.kind()),
        Some(io::ErrorKind::ConnectionRefused)
    );
    Ok(())
}

#[tokio::test]
async fn connection_cap_leaves_excess_clients_in_kernel_backlog_without_extra_tasks() -> TestResult
{
    // Given more simultaneous stalled clients than admission permits.
    const EXTRA_CLIENTS: usize = 4;
    let server = Server::start(OK).await?;
    let proxy = Proxy::start(stalled(&server)?).await?;
    let count = MAX_CONNECTIONS
        .checked_add(EXTRA_CLIENTS)
        .ok_or("client overflow")?;
    let _clients = stalled_clients(&proxy, count).await?;
    reached(&proxy, u64::try_from(MAX_CONNECTIONS)?).await?;
    super::clock::within("connection cap intake checkpoint", proxy.refuse()).await?;
    // Then accepted and active sessions never grow past the named cap.
    let counters = proxy.snapshot()?.counters;
    let admitted = u64::try_from(MAX_CONNECTIONS)?;
    assert_eq!(counters.accepted, admitted);
    assert_eq!(counters.requests, admitted);
    let report = super::support::shutdown(proxy).await?;
    assert_eq!(report.accepted, u64::try_from(MAX_CONNECTIONS)?);
    assert_eq!(report.cancelled, u64::try_from(MAX_CONNECTIONS)?);
    Ok(())
}

#[tokio::test]
async fn failed_rebind_surfaces_typed_io_error_and_can_be_retried_after_port_is_released()
-> TestResult {
    // Given explicit refusal, another owner can occupy the now-closed listener.
    let server = Server::start(OK).await?;
    let proxy = Proxy::start(server.config()).await?;
    super::clock::within("explicit refusal acknowledgement", proxy.refuse()).await?;
    let blocker = TcpListener::bind(proxy.address()).await?;
    // When resume cannot bind, then the handle returns the real typed I/O error.
    match super::support::resume_outcome(&proxy).await? {
        Err(inventory_harness::proxy::ProxyError::Io(error)) => {
            assert_eq!(error.kind(), io::ErrorKind::AddrInUse)
        }
        other => return Err(format!("expected address-in-use, got {other:?}").into()),
    }
    drop(blocker);
    super::clock::within("resume after port release", proxy.resume()).await?;
    assert_eq!(
        super::support::exchange(proxy.address(), UNSIGNED).await?,
        OK
    );
    super::support::shutdown(proxy).await?;
    Ok(())
}

async fn stalled_clients(
    proxy: &Proxy,
    count: usize,
) -> Result<Vec<TcpStream>, Box<dyn std::error::Error>> {
    let mut clients = Vec::with_capacity(count);
    for _ in 0..count {
        let mut client = TcpStream::connect(proxy.address()).await?;
        client.write_all(UNSIGNED).await?;
        clients.push(client);
    }
    Ok(clients)
}
