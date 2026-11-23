use super::*;
mod mapping;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_parenthesized_process_name_when_parsed_then_cpu_fields_keep_their_positions() {
    let stat = "123 (worker (copy) name) R 1 2 3 4 5 6 7 8 9 10 17 23 999";
    assert_eq!(process::cpu(stat), Some((170, 230)));
    assert_eq!(
        process::peak_rss("Name:\tworker\nVmHWM:\t12345 kB\n"),
        Some(12345)
    );
}

#[test]
fn given_missing_malformed_or_overflowing_proc_fields_when_parsed_then_metrics_are_unavailable() {
    for stat in [
        "",
        "123 (worker) R 1",
        "123 (worker) R 1 2 3 4 5 6 7 8 9 10 18446744073709551615 1",
    ] {
        assert_eq!(process::cpu(stat), None);
    }
    for status in ["", "VmHWM: -1 kB", "VmHWM: 10 MB", "VmHWM: 10 kB extra"] {
        assert_eq!(process::peak_rss(status), None);
    }
}

#[test]
fn given_concurrent_pages_when_accumulated_then_every_counter_is_preserved() -> TestResult {
    let counters = Accumulators::default();
    std::thread::scope(|scope| {
        let workers = (0..8)
            .map(|_| scope.spawn(|| accumulate(&counters)))
            .collect::<Vec<_>>();
        for worker in workers {
            worker.join().map_err(|_| "metric worker failed")?;
        }
        Ok::<_, Box<dyn std::error::Error>>(())
    })?;
    let mut metrics = SweepMetrics::default();
    counters.snapshot(&mut metrics);
    assert_eq!((metrics.decode_ns, metrics.store_commit_ns), (1792, 2816));
    assert_eq!(
        (metrics.claims, metrics.pages, metrics.store_commits),
        (256, 256, 256)
    );
    Accumulators::default().snapshot(&mut metrics);
    assert_eq!(
        (metrics.decode_ns, metrics.claims, metrics.pages),
        (0, 0, 0)
    );
    Ok(())
}

fn accumulate(counters: &Accumulators) {
    for _ in 0..32 {
        add(&counters.decode_ns, 7);
        add(&counters.store_commit_ns, 11);
        add(&counters.store_commits, 1);
        counters.claimed(true);
    }
}

#[test]
fn given_failed_commit_or_empty_claim_when_recorded_then_success_counters_stay_zero() {
    let counters = Accumulators::default();
    counters.committed(Instant::now(), false);
    counters.claimed(false);
    let mut metrics = SweepMetrics::default();
    counters.snapshot(&mut metrics);
    assert_eq!(
        (metrics.pages, metrics.store_commits, metrics.claims),
        (0, 0, 0)
    );
}

#[test]
fn given_counter_at_capacity_when_incremented_then_it_does_not_wrap() {
    let counter = AtomicU64::new(u64::MAX);
    add(&counter, 1);
    assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
}
