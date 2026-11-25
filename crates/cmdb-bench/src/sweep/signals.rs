use tokio::signal::unix::{Signal, SignalKind, signal};
use tokio_util::sync::CancellationToken;
pub(super) struct Shutdown {
    interrupt: Signal,
    terminate: Signal,
    cancel: CancellationToken,
}
impl Shutdown {
    pub(super) fn new() -> Result<Self, String> {
        Ok(Self {
            interrupt: signal(SignalKind::interrupt()).map_err(text)?,
            terminate: signal(SignalKind::terminate()).map_err(text)?,
            cancel: CancellationToken::new(),
        })
    }
    pub(super) fn token(&self) -> CancellationToken {
        self.cancel.clone()
    }
    pub(super) async fn run<T>(
        &mut self,
        operation: impl Future<Output = Result<T, String>>,
    ) -> Result<T, String> {
        tokio::pin!(operation);
        let interrupted = tokio::select! {
            biased;
            _ = self.interrupt.recv() => "SIGINT",
            _ = self.terminate.recv() => "SIGTERM",
            result = &mut operation => return result,
        };
        self.cancel.cancel();
        match operation.await {
            Ok(_) => Err(format!("{interrupted}: benchmark cancelled after cleanup")),
            Err(error) => Err(format!("{interrupted}: {error}")),
        }
    }
}
fn text(error: impl std::fmt::Display) -> String {
    error.to_string()
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::sweep::collector;
    use futures_util::{StreamExt, stream};
    #[tokio::test]
    async fn interrupt_and_terminate_cancel_and_reap_the_real_collector() -> Result<(), String> {
        exercise("INT").await?;
        exercise("TERM").await
    }
    async fn exercise(signal: &str) -> Result<(), String> {
        let directory = tempfile::tempdir().map_err(text)?;
        let pid = directory.path().join("pid");
        let mut shutdown = Shutdown::new()?;
        let cancel = shutdown.token();
        let mut command = tokio::process::Command::new("/bin/sh");
        command
            .arg("-c")
            .arg("printf '%s' \"$$\" > \"$1\"; exec sleep 3600")
            .arg("collector")
            .arg(&pid);
        let log = directory.path().join("log");
        let operation = collector::sweep(command, &log, &cancel);
        let send = async {
            let ready = stream::iter(0..100_000)
                .then(|_| async {
                    tokio::task::yield_now().await;
                    tokio::fs::read_to_string(&pid)
                        .await
                        .ok()
                        .filter(|text| !text.is_empty())
                })
                .filter_map(|value| async move { value });
            tokio::pin!(ready);
            let ready = ready.next().await.ok_or("collector not ready")?;
            let status = tokio::process::Command::new("/bin/kill")
                .args(["-s", signal, &std::process::id().to_string()])
                .status()
                .await
                .map_err(text)?;
            if !status.success() {
                return Err("send signal failed".to_string());
            }
            Ok::<_, String>(ready)
        };
        let (result, sent) = tokio::join!(shutdown.run(operation), send);
        let pid = sent?;
        let error = result.err().ok_or("signal did not fail operation")?;
        assert!(error.contains(signal), "{error}");
        assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
        Ok(())
    }
}
