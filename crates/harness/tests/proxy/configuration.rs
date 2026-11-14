use super::support::{OK, Server, TestResult};
use inventory_harness::proxy::{
    Config, Fault, MAX_DELAY, MAX_LOG_ENTRIES, MAX_RATE, MAX_RULES, MAX_SCOPE_NAME, Probability,
    Proxy, ProxyError, RateLimit, Rule, Schedule, Scope, ScopeFilter, ScopeName,
};

fn rejected(result: Result<Proxy, ProxyError>, expected: &'static str) -> TestResult {
    match result {
        Err(ProxyError::Configuration(reason)) => assert_eq!(reason, expected),
        Err(error) => return Err(format!("unexpected error: {error}").into()),
        Ok(_) => return Err("invalid configuration accepted".into()),
    }
    Ok(())
}

#[test]
fn probability_and_scope_names_enforce_exact_boundaries_without_panics() -> TestResult {
    // Given boundary and hostile primitive values, when parsed at the boundary.
    assert!(matches!(
        Probability::new(1_000_001),
        Err(ProxyError::Configuration("probability exceeds one million"))
    ));
    for invalid in ["", "space key", "slash/key", "é", "<xml>"] {
        assert!(matches!(
            ScopeName::parse(invalid),
            Err(ProxyError::Configuration("scope component"))
        ));
    }
    // Then the maximum accepted size is exact, and one extra byte is rejected.
    assert_eq!(
        ScopeName::parse(&"a".repeat(MAX_SCOPE_NAME))?
            .as_str()
            .len(),
        MAX_SCOPE_NAME
    );
    assert!(matches!(
        ScopeName::parse(&"a".repeat(MAX_SCOPE_NAME.saturating_add(1))),
        Err(ProxyError::Configuration("scope component"))
    ));
    Ok(())
}

#[tokio::test]
async fn startup_rejects_out_of_bound_tables_rates_and_durations_before_binding() -> TestResult {
    // Given a valid endpoint, when each startup resource bound is violated.
    let server = Server::start(OK).await?;
    let mut configs = Vec::new();
    let mut zero = server.config();
    zero.log_capacity = 0;
    configs.push((zero, "configuration table capacity"));
    let mut over = server.config();
    over.log_capacity = MAX_LOG_ENTRIES.saturating_add(1);
    configs.push((over, "configuration table capacity"));
    let mut scopes = server.config();
    scopes.scope_capacity = 0;
    configs.push((scopes, "configuration table capacity"));
    invalid_rules_and_durations(&server, &mut configs)?;
    invalid_rates(&server, &mut configs);
    // Then each failure is typed and identifies its violated boundary.
    for (config, reason) in configs {
        rejected(Proxy::start(config).await, reason)?;
    }
    Ok(())
}

#[tokio::test]
async fn startup_rejects_duplicate_scheduled_ordinals_and_invalid_upstream_authorities()
-> TestResult {
    // Given duplicated exact overrides, when startup validates immutable plans.
    let server = Server::start(OK).await?;
    let mut duplicate = server.config();
    let scheduled = Schedule {
        scope: Scope::new("a", "r", "s")?,
        ordinal: 0,
        fault: Fault::Throttle,
    };
    duplicate.schedule = vec![scheduled.clone(), scheduled];
    rejected(Proxy::start(duplicate).await, "duplicate configuration key")?;
    // Then malformed upstreams and HTTPS without explicit trust are also rejected.
    for (endpoint, reason) in [
        ("ftp://host:80", "upstream scheme"),
        ("http://host", "upstream port"),
        ("http://user@host:80", "upstream host"),
        ("http://host:0", "zero upstream port"),
        ("https://host:443", "HTTPS requires CA; HTTP forbids CA"),
    ] {
        rejected(Proxy::start(Config::new(endpoint, 0)).await, reason)?;
    }
    Ok(())
}

type InvalidCases = Vec<(Config, &'static str)>;
fn invalid_rules_and_durations(server: &Server, configs: &mut InvalidCases) -> TestResult {
    let mut rules = server.config();
    rules.rules = vec![
        Rule {
            filter: ScopeFilter::default(),
            probability: Probability::new(0)?,
            fault: Fault::Reset
        };
        MAX_RULES.saturating_add(1)
    ];
    configs.push((rules, "configuration table capacity"));
    let mut refusal = server.config();
    refusal.refusal_duration = std::time::Duration::ZERO;
    configs.push((refusal, "duration or rate"));
    let mut delay = server.config();
    delay.latency.push((
        ScopeFilter::default(),
        MAX_DELAY.saturating_add(std::time::Duration::from_nanos(1)),
    ));
    configs.push((delay, "duration or rate"));
    Ok(())
}

fn invalid_rates(server: &Server, configs: &mut InvalidCases) {
    for rate in [0, MAX_RATE.saturating_add(1)] {
        let mut config = server.config();
        config.rate_limits.push(RateLimit {
            filter: ScopeFilter::default(),
            requests_per_second: rate,
        });
        configs.push((config, "duration or rate"));
    }
}
