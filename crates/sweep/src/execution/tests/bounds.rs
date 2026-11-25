use super::*;
use descriptors::SoftLimit;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn config(concurrency: usize) -> SweepConfig {
    SweepConfig {
        region: None,
        max_pages: 1,
        concurrency,
        resume: false,
        endpoint: None,
        ca_bundle: None,
        idle_timeout: None,
        pool_idle_per_host: None,
        slice_timeout_secs: None,
        max_rps: None,
        max_page_size: None,
    }
}

#[test]
fn given_concurrency_outside_the_limiter_range_when_validated_then_it_is_rejected() {
    // Given a concurrency of zero and one past the limiter's maximum.
    for concurrency in [0, 4_097] {
        // When the configuration is validated.
        let result = config(concurrency).validate();
        // Then it is rejected with the accepted range.
        assert!(matches!(
            result,
            Err(SweepError::InvalidConfig("concurrency must be in 1..=4096"))
        ));
    }
}

#[test]
fn given_a_rate_cap_outside_the_limiter_range_when_validated_then_it_is_rejected() {
    // Given a cap of zero and one past the limiter's maximum.
    for cap in [0, 100_001] {
        let config = SweepConfig {
            max_rps: Some(cap),
            ..config(1)
        };
        // When the configuration is validated.
        let result = config.validate();
        // Then it is rejected with the accepted range.
        assert!(matches!(
            result,
            Err(SweepError::InvalidConfig("max-rps must be in 1..=100000"))
        ));
    }
}

#[test]
fn given_a_rate_cap_at_either_bound_when_validated_then_it_is_accepted() -> TestResult {
    // Given the smallest and the largest accepted cap.
    for cap in [1, 100_000] {
        let config = SweepConfig {
            max_rps: Some(cap),
            ..config(1)
        };
        // When the configuration is validated. Then it is accepted.
        config.validate()?;
    }
    Ok(())
}

#[test]
fn given_a_page_size_outside_the_allowed_range_when_validated_then_it_is_rejected() {
    // Given a page size of zero and one past the limiter's maximum.
    for size in [0, 1_000_001] {
        let config = SweepConfig {
            max_page_size: Some(size),
            ..config(1)
        };
        // When the configuration is validated.
        let result = config.validate();
        // Then it is rejected with the accepted range.
        assert!(matches!(
            result,
            Err(SweepError::InvalidConfig(
                "max-page-size must be in 1..=1000000"
            ))
        ));
    }
}

#[test]
fn given_a_page_size_at_either_bound_when_validated_then_it_is_accepted() -> TestResult {
    // Given the smallest and the largest accepted page size.
    for size in [1, 1_000_000] {
        let config = SweepConfig {
            max_page_size: Some(size),
            ..config(1)
        };
        // When the configuration is validated. Then it is accepted.
        config.validate()?;
    }
    Ok(())
}

#[tokio::test]
async fn given_the_largest_concurrency_when_resources_are_built_then_every_layer_accepts_it()
-> TestResult {
    // Given the largest accepted concurrency and no open-file limit.
    let config = config(4_096);
    config.validate()?;
    // When the transport and executor settings are derived from it.
    let transport = transport_config(&config, Some(SoftLimit::Unlimited))?;
    let settings = executor_config(config.concurrency)?;
    // Then one connection per page is allowed and the transport, executor and lease accept it.
    assert_eq!(transport.max_connections, 4_096);
    let _client = Client::new(transport)?;
    let _executor = ApiExecutor::new(settings)?;
    assert!(lease_budget(settings)? > 0);
    Ok(())
}

#[test]
fn given_a_1024_file_limit_when_concurrency_is_4096_then_connections_stop_at_768() -> TestResult {
    // Given 4,096 pages in flight and a soft limit of 1,024 open files.
    let config = config(4_096);
    // When the transport settings are derived.
    let transport = transport_config(&config, Some(SoftLimit::Files(1_024)))?;
    // Then 256 descriptors are kept back and the idle pool stays at eight.
    assert_eq!(transport.max_connections, 768);
    assert_eq!(transport.idle_timeout, Duration::from_secs(1));
    assert_eq!(transport.pool_idle_per_host, 8);
    Ok(())
}

#[test]
fn given_one_page_at_a_time_when_configured_then_the_idle_pool_fits_the_connection_bound()
-> TestResult {
    // Given one page in flight and an unreadable open-file limit.
    let config = config(1);
    // When the transport settings are derived.
    let transport = transport_config(&config, None)?;
    // Then one connection is allowed and at most one is kept idle.
    assert_eq!(transport.max_connections, 1);
    assert_eq!(transport.pool_idle_per_host, 1);
    Ok(())
}

#[test]
fn given_transport_overrides_when_configured_then_the_pool_uses_them() -> TestResult {
    let config = SweepConfig {
        idle_timeout: Some(Duration::from_secs(10)),
        pool_idle_per_host: Some(64),
        ..config(64)
    };
    let transport = transport_config(&config, Some(SoftLimit::Unlimited))?;
    assert_eq!(transport.idle_timeout, Duration::from_secs(10));
    assert_eq!(transport.pool_idle_per_host, 64);
    assert_eq!(Client::new(transport)?.statistics(), Default::default());
    Ok(())
}

#[test]
fn given_invalid_idle_timeout_when_configured_then_the_error_names_the_option_value_and_range() {
    for milliseconds in [0, 10_001] {
        let config = SweepConfig {
            idle_timeout: Some(Duration::from_millis(milliseconds)),
            ..config(64)
        };
        let result = transport_config(&config, Some(SoftLimit::Unlimited));
        assert_eq!(
            result.err().map(|error| error.to_string()),
            Some(format!(
                "invalid sweep configuration: --idle-timeout-ms (INVENTORY_SWEEP_IDLE_TIMEOUT_MS) \
                 was {milliseconds}; allowed range is 1..=10000 ms"
            ))
        );
    }
}

#[test]
fn given_invalid_pool_size_when_configured_then_the_error_names_the_descriptor_bound() {
    for given in [0, 17] {
        let config = SweepConfig {
            pool_idle_per_host: Some(given),
            ..config(64)
        };
        let result = transport_config(&config, Some(SoftLimit::Files(272)));
        assert_eq!(
            result.err().map(|error| error.to_string()),
            Some(format!(
                "invalid sweep configuration: --pool-idle-per-host \
                 (INVENTORY_SWEEP_POOL_IDLE_PER_HOST) was {given}; allowed range is 1..=16"
            ))
        );
    }
}
