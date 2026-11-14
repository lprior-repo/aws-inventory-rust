use super::support::{OK, Server, TestResult, UNSIGNED, exchange};
use inventory_harness::proxy::{Fault, Proxy};
use std::{io, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::test(start_paused = true)]
async fn stall_sends_nothing_until_shutdown_then_closes_cleanly() -> TestResult {
    let _clock = super::clock::FrozenClock::new();
    super::clock::within(
        "stalled cancellation",
        super::review_lifecycle::shutdown_stall(),
    )
    .await
}

#[tokio::test]
async fn reset_is_observed_as_connection_reset_not_eof() -> TestResult {
    // Given zero-linger reset after the complete request.
    let server = Server::start(OK).await?;
    let proxy = Proxy::start(server.fault(Fault::Reset)).await?;
    let mut client = TcpStream::connect(proxy.address()).await?;
    client.write_all(UNSIGNED).await?;
    // When reading, then the OS reports an abortive close.
    let outcome = super::clock::within("zero-linger reset read", client.read(&mut [0])).await;
    assert_eq!(
        outcome
            .err()
            .and_then(|error| error.downcast::<io::Error>().ok())
            .map(|error| error.kind()),
        Some(io::ErrorKind::ConnectionReset)
    );
    super::support::shutdown(proxy).await?;
    Ok(())
}

#[tokio::test]
async fn truncation_keeps_declared_length_but_ends_after_seeded_strict_prefixes() -> TestResult {
    super::clock::within("seeded truncation replay", super::seeded::truncation()).await
}

#[tokio::test]
async fn malformed_body_preserves_length_with_seeded_invalid_json_xml_and_cbor() -> TestResult {
    super::clock::within("seeded corruption replay", super::seeded::malformed()).await
}

#[tokio::test]
async fn explicit_refusal_rejects_connects_until_resume_acknowledges_rebind() -> TestResult {
    // Given a running fault-free proxy, when its listener is refused.
    let server = Server::start(OK).await?;
    let proxy = Proxy::start(server.config()).await?;
    super::clock::within("explicit refusal acknowledgement", proxy.refuse()).await?;
    let outcome = TcpStream::connect(proxy.address()).await;
    // Then actual connect fails, not just a post-connect close.
    assert_eq!(
        outcome.err().map(|e| e.kind()),
        Some(io::ErrorKind::ConnectionRefused)
    );
    super::clock::within("resume acknowledgement", proxy.resume()).await?;
    assert_eq!(exchange(proxy.address(), UNSIGNED).await?, OK);
    super::support::shutdown(proxy).await?;
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn scheduled_refusal_drops_trigger_then_rebinds_after_its_exact_window() -> TestResult {
    let _clock = super::clock::FrozenClock::new();
    // Given ordinal zero refuses for two virtual seconds.
    let server = Server::start(OK).await?;
    let mut config = server.fault(Fault::RefusedConnection);
    config.refusal_duration = Duration::from_secs(2);
    let proxy = Proxy::start(config).await?;
    assert_eq!(exchange(proxy.address(), UNSIGNED).await?, b"");
    // When still in the window, then the listening socket really is absent.
    super::wire::refused(proxy.address()).await?;
    tokio::time::advance(Duration::from_secs(1)).await;
    super::wire::refused(proxy.address()).await?;
    tokio::time::advance(Duration::from_secs(1)).await;
    super::wire::rebound(proxy.address()).await?;
    assert_eq!(exchange(proxy.address(), UNSIGNED).await?, OK);
    super::support::shutdown(proxy).await?;
    Ok(())
}
