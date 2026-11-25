#[path = "probe_native/report.rs"]
mod report;
#[path = "probe_native/runtime.rs"]
mod runtime;

use std::{io, time::Duration};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
async fn given_seeded_thin_slice_when_native_probe_runs_then_every_callable_has_evidence()
-> TestResult {
    let native = tokio::task::spawn_blocking(|| {
        runtime::Native::start().map_err(|error| io::Error::other(error.to_string()))
    })
    .await??;
    let text = tokio::time::timeout(Duration::from_secs(1200), async {
        native.seed().await?;
        native.probe().await
    })
    .await??;
    report::verify(&text).await?;
    native.stop()
}
