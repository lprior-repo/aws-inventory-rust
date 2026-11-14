use super::support::{OK, Server, TestResult, UNSIGNED, exchange, field, request};
use inventory_harness::proxy::{
    Fault, Probability, Proxy, Rule, Schedule, Scope, ScopeFilter, ScopeName,
};

#[tokio::test]
async fn probabilistic_scopes_replay_across_interleavings_and_match_tcp_responses() -> TestResult {
    let server = Server::start(OK).await?;
    super::clock::within(
        "probabilistic cross-scope replay",
        super::probability::verify(&server),
    )
    .await
}

#[tokio::test]
async fn scheduled_entries_override_probability_at_exact_scope_ordinals_only() -> TestResult {
    let server = Server::start(OK).await?;
    let config = override_config(&server)?;
    let proxy = Proxy::start(config).await?;

    for (key, code) in [
        ("a", "ThrottlingException"),
        ("b", "ThrottlingException"),
        ("a", "ExpiredTokenException"),
        ("a", "ThrottlingException"),
        ("b", "ThrottlingException"),
    ] {
        observed_code(&proxy, key, code).await?;
    }
    assert_scope_ordinals(
        &proxy.snapshot()?,
        &[("a", 0), ("a", 1), ("a", 2), ("b", 0), ("b", 1)],
    );
    super::support::shutdown(proxy).await?;
    Ok(())
}

#[tokio::test]
async fn mapped_access_keys_share_account_scope_and_unsigned_requests_share_fallback() -> TestResult
{
    let server = Server::start(OK).await?;
    let mut config = server.config();
    for key in ["keyA", "keyB"] {
        config
            .accounts
            .push((ScopeName::parse(key)?, ScopeName::parse("123456789012")?));
    }
    let proxy = Proxy::start(config).await?;
    mapped_requests(&proxy).await?;

    let snapshot = proxy.snapshot()?;
    assert_eq!(snapshot.scopes, 2);
    assert_scope_ordinals(&snapshot, &MAPPED_ORDINALS);
    super::support::shutdown(proxy).await?;
    Ok(())
}

#[tokio::test]
async fn scope_capacity_rejects_new_scope_without_evicting_existing_ordinal() -> TestResult {
    let server = Server::start(OK).await?;
    let mut config = server.config();
    config.scope_capacity = 1;
    let proxy = Proxy::start(config).await?;
    assert_eq!(
        exchange(proxy.address(), &request("a", "us-east-1", "s3")).await?,
        OK
    );

    let rejected = exchange(proxy.address(), &request("b", "us-east-1", "s3")).await?;
    assert!(rejected.starts_with(b"HTTP/1.1 503 "));
    assert_eq!(field(&rejected, "x-proxy-error")?, "scope-capacity");
    assert_eq!(
        exchange(proxy.address(), &request("a", "us-east-1", "s3")).await?,
        OK
    );
    assert_retained_scope(&proxy.snapshot()?);
    super::support::shutdown(proxy).await?;
    Ok(())
}

#[tokio::test]
async fn decision_ring_drops_oldest_entries_and_counts_each_loss() -> TestResult {
    let server = Server::start(OK).await?;
    let mut config = server.config();
    config.log_capacity = 2;
    let proxy = Proxy::start(config).await?;
    for _ in 0..5 {
        assert_eq!(exchange(proxy.address(), UNSIGNED).await?, OK);
    }

    let snapshot = proxy.snapshot()?;
    assert_eq!(
        snapshot
            .decisions
            .iter()
            .map(|e| e.ordinal)
            .collect::<Vec<_>>(),
        [3, 4]
    );
    assert_eq!(snapshot.dropped, 3);
    assert_eq!(snapshot.counters.requests, 5);
    super::support::shutdown(proxy).await?;
    Ok(())
}

#[tokio::test]
async fn scope_filter_never_faults_nonmatching_account_region_or_service() -> TestResult {
    let server = Server::start(OK).await?;
    let config = denied_scope(&server)?;
    let proxy = Proxy::start(config).await?;
    for (key, region, service) in [
        ("b", "us-east-1", "s3"),
        ("a", "us-west-2", "s3"),
        ("a", "us-east-1", "ec2"),
    ] {
        assert_eq!(
            exchange(proxy.address(), &request(key, region, service)).await?,
            OK
        );
    }
    let denied = exchange(proxy.address(), &request("a", "us-east-1", "s3")).await?;
    assert_eq!(field(&denied, "x-amzn-ErrorType")?, "AccessDeniedException");
    super::support::shutdown(proxy).await?;
    Ok(())
}

type ConfigResult = Result<inventory_harness::proxy::Config, Box<dyn std::error::Error>>;
const MAPPED_ORDINALS: [(&str, u64); 4] = [
    ("123456789012", 0),
    ("123456789012", 1),
    ("unsigned", 0),
    ("unsigned", 1),
];

fn override_config(server: &Server) -> ConfigResult {
    let mut config = server.config();
    config.rules.push(Rule {
        filter: ScopeFilter::default(),
        probability: Probability::new(1_000_000)?,
        fault: Fault::Throttle,
    });
    config.schedule.push(Schedule {
        scope: Scope::new("a", "us-east-1", "s3")?,
        ordinal: 1,
        fault: Fault::ExpiredCredentials,
    });
    Ok(config)
}

async fn observed_code(proxy: &Proxy, key: &str, code: &str) -> TestResult {
    let bytes = exchange(proxy.address(), &request(key, "us-east-1", "s3")).await?;
    assert_eq!(field(&bytes, "x-amzn-ErrorType")?, code);
    Ok(())
}

fn assert_scope_ordinals(snapshot: &inventory_harness::proxy::Snapshot, expected: &[(&str, u64)]) {
    assert_eq!(
        snapshot
            .decisions
            .iter()
            .map(|entry| {
                let account = match &entry.scope {
                    Scope::Signed { account, .. } => account.as_str(),
                    Scope::Unsigned => "unsigned",
                };
                (account, entry.ordinal)
            })
            .collect::<Vec<_>>(),
        expected
    );
}

async fn mapped_requests(proxy: &Proxy) -> TestResult {
    for key in ["keyA", "keyB"] {
        assert_eq!(
            exchange(proxy.address(), &request(key, "us-east-1", "s3")).await?,
            OK
        );
    }
    assert_eq!(exchange(proxy.address(), UNSIGNED).await?, OK);
    assert_eq!(
        exchange(
            proxy.address(),
            b"GET / HTTP/1.1\r\nHost: x\r\nAuthorization: nonsense\r\nConnection: close\r\n\r\n"
        )
        .await?,
        OK
    );
    Ok(())
}

fn assert_retained_scope(snapshot: &inventory_harness::proxy::Snapshot) {
    assert_eq!(snapshot.scopes, 1);
    assert_eq!(
        snapshot
            .decisions
            .iter()
            .map(|entry| entry.ordinal)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(snapshot.counters.requests, 3);
    assert_eq!(snapshot.counters.forwarded, 2);
}

fn denied_scope(server: &Server) -> ConfigResult {
    let mut config = server.config();
    config.rules.push(Rule {
        filter: ScopeFilter {
            account: Some(ScopeName::parse("a")?),
            region: Some(ScopeName::parse("us-east-1")?),
            service: Some(ScopeName::parse("s3")?),
        },
        probability: Probability::new(1_000_000)?,
        fault: Fault::AccessDenied,
    });
    Ok(config)
}
