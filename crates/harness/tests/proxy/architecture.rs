use super::{
    clock::{FrozenClock, within},
    support::{OK, Server, TestResult, UNSIGNED, exchange, field, request, shutdown},
};
use inventory_harness::proxy::{
    Config, Proxy, ProxyError, RateLimit, Scope, ScopeFilter, Snapshot,
};
use std::{io::Write, path::Path};
use tokio::net::TcpStream;

#[tokio::test(start_paused = true)]
async fn unsigned_and_signed_placeholder_scopes_keep_separate_ordinals_and_rates() -> TestResult {
    let _clock = FrozenClock::new();
    within(
        "distinct unsigned identity and rate buckets",
        isolated_scopes(),
    )
    .await
}

async fn isolated_scopes() -> TestResult {
    let server = Server::start(OK).await?;
    let mut config = server.config();
    config.rate_limits.push(RateLimit {
        filter: ScopeFilter::default(),
        requests_per_second: 1,
    });
    let proxy = Proxy::start(config).await?;
    let signed = request("unsigned", "unsigned", "unsigned");

    assert_eq!(exchange(proxy.address(), UNSIGNED).await?, OK);
    assert_eq!(exchange(proxy.address(), &signed).await?, OK);
    assert_throttled(&proxy, UNSIGNED).await?;
    assert_throttled(&proxy, &signed).await?;

    assert_distinct_scopes(&proxy.snapshot()?)?;
    shutdown(proxy).await?;
    Ok(())
}

async fn assert_throttled(proxy: &Proxy, request: &[u8]) -> TestResult {
    let bytes = exchange(proxy.address(), request).await?;
    assert!(bytes.starts_with(b"HTTP/1.1 400 "));
    assert_eq!(field(&bytes, "x-amzn-ErrorType")?, "ThrottlingException");
    Ok(())
}

fn assert_distinct_scopes(snapshot: &Snapshot) -> TestResult {
    let signed = Scope::new("unsigned", "unsigned", "unsigned")?;
    let actual: Vec<_> = snapshot
        .decisions
        .iter()
        .map(|entry| (entry.scope.clone(), entry.ordinal))
        .collect();
    let expected = [
        (signed.clone(), 0),
        (signed, 1),
        (Scope::Unsigned, 0),
        (Scope::Unsigned, 1),
    ];
    assert_eq!(actual, expected);
    assert_eq!(snapshot.scopes, 2);
    assert_eq!(snapshot.counters.forwarded, 2);
    assert_eq!(snapshot.counters.rate_limited, 2);
    assert_domain_entropy(snapshot);
    Ok(())
}

fn assert_domain_entropy(snapshot: &Snapshot) {
    let entropy: Vec<_> = snapshot
        .decisions
        .iter()
        .filter(|entry| entry.ordinal == 0)
        .map(|entry| entry.decision.entropy)
        .collect();
    assert_eq!(entropy.len(), 2);
    assert_ne!(entropy.first(), entropy.last());
}

#[tokio::test]
async fn valid_ca_with_malformed_tls_authority_fails_before_returning_a_bound_address() -> TestResult
{
    within("validate TLS authority before binding", tls_authorities()).await
}

async fn tls_authorities() -> TestResult {
    let mut ca = tempfile::NamedTempFile::new()?;
    ca.write_all(include_bytes!("ca.pem"))?;
    valid_tls_authority(ca.path()).await?;

    for authority in ["https://-:443", "https://....:443"] {
        let result = Proxy::start(tls_config(authority, ca.path())).await;

        assert_eq!(result.as_ref().ok().map(Proxy::address), None);
        assert!(matches!(
            result,
            Err(ProxyError::Configuration("TLS server name"))
        ));
    }
    Ok(())
}

fn tls_config(authority: &str, ca: &Path) -> Config {
    let mut config = Config::new(authority.to_owned(), 7);
    config.ca_bundle = Some(ca.to_owned());
    config
}

async fn valid_tls_authority(ca: &Path) -> TestResult {
    let proxy = Proxy::start(tls_config("https://127.0.0.1:443", ca)).await?;
    let client = TcpStream::connect(proxy.address()).await?;
    assert_eq!(client.peer_addr()?, proxy.address());
    drop(client);
    shutdown(proxy).await?;
    Ok(())
}
