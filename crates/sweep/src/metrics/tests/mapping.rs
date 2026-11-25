use super::*;

#[test]
fn given_retry_and_http2_statistics_when_snapshotted_then_original_counts_are_reported() {
    let executor = inventory_engine::limiter::ExecutorStats {
        attempts: 9,
        retries: 3,
        throttles: 2,
        ..Default::default()
    };
    let transport = aws_wire::transport::Statistics {
        connections_opened: 2,
        requests_sent: 9,
        requests_reused: 7,
        http2_requests: 8,
        bytes_read: 123,
        handshake_ns: 456,
        hosts: 5,
        ..Default::default()
    };
    let mut metrics = SweepMetrics::default();
    complete(&mut metrics, &Accumulators::default(), executor, transport);
    assert_executor(&metrics);
    assert_transport(&metrics);
}

fn assert_executor(metrics: &SweepMetrics) {
    assert_eq!(
        (
            metrics.api_attempts,
            metrics.api_retries,
            metrics.api_throttles
        ),
        (9, 3, 2)
    );
}

fn assert_transport(metrics: &SweepMetrics) {
    assert_eq!(
        (
            metrics.requests_sent,
            metrics.requests_reused,
            metrics.http2_requests
        ),
        (9, 7, 8)
    );
    assert_eq!((metrics.connections_opened, metrics.bytes_read), (2, 123));
    assert_eq!((metrics.handshake_ns, metrics.hosts), (456, 5));
}
