use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct NeverCalled(Arc<AtomicUsize>);

impl Actions for NeverCalled {
    async fn create(&self) -> CreateOutcome {
        self.0.fetch_add(1, Ordering::SeqCst);
        CreateOutcome::NotCreated(HarnessError::invalid("create must not run"))
    }

    async fn remove(&self) -> Result<(), HarnessError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn given_runtime_build_failure_when_serving_then_waiter_gets_error_and_no_cleanup_is_recorded()
-> TestResult {
    let calls = Arc::new(AtomicUsize::new(0));
    let failures = CleanupFailures::default();
    let (created, ready) = oneshot::channel();
    let (_keep, command) = mpsc::sync_channel(1);
    let (done, finished) = oneshot::channel();
    serve(
        Err(std::io::Error::other("runtime unavailable")),
        NeverCalled(calls.clone()),
        "runtime-database",
        &failures,
        created,
        command,
        done,
    );
    let error = ready
        .await?
        .err()
        .ok_or("runtime failure must reach the creation waiter")?;
    assert!(error.to_string().contains("runtime unavailable"));
    finished.await??;
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(failures.failures().is_empty());
    Ok(())
}
