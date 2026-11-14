use super::support::{OK, Server, TestResult, UNSIGNED, exchange, field, request};
use inventory_harness::proxy::{Config, Proxy, RateLimit, ScopeFilter};
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn fixed_latency_delays_bytes_until_virtual_deadline() -> TestResult {
    let _clock = super::clock::FrozenClock::new();
    super::clock::within(
        "canned latency deadline",
        super::review_timing::canned_latency(),
    )
    .await
}

fn limited(server: &Server) -> Config {
    let mut config = server.config();
    config.rate_limits.push(RateLimit {
        filter: ScopeFilter::default(),
        requests_per_second: 2,
    });
    config
}

#[tokio::test(start_paused = true)]
async fn rate_bucket_allows_n_then_throttles_and_refills_with_virtual_time() -> TestResult {
    let _clock = super::clock::FrozenClock::new();
    // Given a two-token bucket, when three requests arrive without elapsed time.
    let server = Server::start(OK).await?;
    let proxy = Proxy::start(limited(&server)).await?;
    assert_eq!(exchange(proxy.address(), UNSIGNED).await?, OK);
    assert_eq!(exchange(proxy.address(), UNSIGNED).await?, OK);
    throttled(&proxy, UNSIGNED).await?;
    assert_eq!(proxy.snapshot()?.counters.forwarded, 2);
    // Then half a second refills exactly one token.
    tokio::time::advance(Duration::from_millis(500)).await;
    assert_eq!(exchange(proxy.address(), UNSIGNED).await?, OK);
    throttled(&proxy, UNSIGNED).await?;
    tokio::time::advance(Duration::from_secs(1)).await;
    assert_eq!(exchange(proxy.address(), UNSIGNED).await?, OK);
    assert_eq!(exchange(proxy.address(), UNSIGNED).await?, OK);
    assert_eq!(proxy.snapshot()?.counters.rate_limited, 2);
    super::support::shutdown(proxy).await?;
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn account_region_and_service_each_isolate_rate_buckets() -> TestResult {
    let _clock = super::clock::FrozenClock::new();
    // Given four scope tuples differing in one coordinate, each has two tokens.
    let server = Server::start(OK).await?;
    let proxy = Proxy::start(limited(&server)).await?;
    let scopes = [
        ("a", "us-east-1", "s3"),
        ("b", "us-east-1", "s3"),
        ("a", "us-west-2", "s3"),
        ("a", "us-east-1", "ec2"),
    ];
    for (account, region, service) in scopes {
        let request = request(account, region, service);
        assert_eq!(exchange(proxy.address(), &request).await?, OK);
        assert_eq!(exchange(proxy.address(), &request).await?, OK);
        throttled(&proxy, &request).await?;
    }
    // Then no cross-coordinate bucket consumed another scope's capacity.
    assert_eq!(proxy.snapshot()?.scopes, 4);
    assert_eq!(proxy.snapshot()?.counters.forwarded, 8);
    assert_eq!(proxy.snapshot()?.counters.rate_limited, 4);
    super::support::shutdown(proxy).await?;
    Ok(())
}

async fn throttled(proxy: &Proxy, request: &[u8]) -> TestResult {
    let bytes = exchange(proxy.address(), request).await?;
    assert_eq!(field(&bytes, "x-amzn-ErrorType")?, "ThrottlingException");
    Ok(())
}
