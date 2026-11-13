//! collection responsibilities for emulator_snapshot.
use super::*;

pub(super) async fn consume_events(
    store: &Store,
    receiver: tokio::sync::mpsc::Receiver<CollectionEvent>,
    started_at: i64,
) -> TestResult {
    stream::unfold(receiver, |mut receiver| async move {
        receiver.recv().await.map(|event| (event, receiver))
    })
    .map(Ok::<_, Box<dyn std::error::Error + Send + Sync>>)
    .try_fold((0_usize, 0_usize), |(count, bytes), event| async move {
        let count = count
            .checked_add(1)
            .ok_or_else(|| fail("event count overflow"))?;
        let bytes = bytes
            .checked_add(serde_json::to_vec(&event)?.len())
            .ok_or_else(|| fail("event byte count overflow"))?;
        if count > MAX_EVENTS || bytes > MAX_EVENT_BYTES {
            return Err(fail("collector exceeded the snapshot event budget"));
        }
        match event {
            CollectionEvent::Page(page) => {
                page.validate()?;
                let current_time = now_millis()?;
                if page
                    .resources
                    .iter()
                    .map(|resource| resource.observed_at)
                    .chain(page.edges.iter().map(|edge| edge.observed_at))
                    .any(|time| time < started_at || time > current_time)
                {
                    return Err(fail(
                        "local observation time is outside the actual collection interval",
                    ));
                }
                store.apply_page(SCAN_ID, page).await?;
            }
            CollectionEvent::Failure(failure) => store.record_failure(SCAN_ID, &failure).await?,
        }
        Ok((count, bytes))
    })
    .await?;
    Ok(())
}

pub(super) async fn collect_case(
    store: &Store,
    scan_options: AwsScanOptions,
    started_at: i64,
) -> TestResult {
    let executor = Arc::new(ApiExecutor::new(ExecutorConfig {
        global_concurrency: 32,
        per_bucket_concurrency: 4,
        requests_per_second: 100,
        max_attempts: 2,
        ..ExecutorConfig::default()
    })?);
    let (sender, receiver) = tokio::sync::mpsc::channel(32);
    let collect = inventory_aws::scan(scan_options, Arc::clone(&executor), sender);
    let consume = consume_events(store, receiver, started_at);
    let work = async { tokio::join!(collect, consume) };
    tokio::pin!(work);
    let outcome = tokio::select! {
        result = &mut work => result,
        () = tokio::time::sleep(CASE_TIMEOUT) => {
            executor.cancel();
            let drained = tokio::time::timeout(DRAIN_TIMEOUT, &mut work).await.is_ok();
            return Err(fail(format!("snapshot collection exceeded its deadline; cancelled and drained={drained}")));
        }
    };
    outcome.0?;
    outcome.1?;
    if executor.stats().in_flight != 0 {
        return Err(fail("collector returned with an outstanding API permit"));
    }
    Ok(())
}
