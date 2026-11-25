mod series;
#[cfg(test)]
mod tests;
mod transport;
use super::{
    args::ProxySpec,
    collector::{self, Finished},
};
use futures_util::{StreamExt, TryStreamExt, stream};
use inventory_harness::proxy::{Config, Proxy, RateLimit, ScopeFilter};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{net::TcpListener, task::JoinHandle, time::Instant};
use tokio_util::sync::CancellationToken;
const SEED: u64 = 0xB5_04_50_20;
pub(crate) type Extra = Vec<(&'static str, Option<u64>)>;
pub(super) fn seed_header() -> String {
    format!("{SEED:#x}")
}
/// A collection owns a fresh fault sequence and bucket, then drains its front.
pub(super) async fn collect<T>(
    upstream: &str,
    ca: &Path,
    spec: ProxySpec,
    work: &Path,
    operation: impl AsyncFnOnce(&Front) -> Result<T, String>,
) -> Result<T, String> {
    tokio::fs::create_dir_all(work).await.map_err(text)?;
    let front = Front::start(upstream, ca, spec, work).await?;
    let result = operation(&front).await;
    let cleanup = front.shutdown().await.map(|_| ());
    super::combine(result, cleanup)
}
pub(crate) struct Front {
    proxy: Option<Proxy>,
    endpoint: String,
    ca: PathBuf,
    cancel: CancellationToken,
    listener: Option<JoinHandle<Result<transport::Report, String>>>,
    spec: ProxySpec,
}
impl Front {
    pub(crate) async fn start(
        upstream: &str,
        ca: &Path,
        spec: ProxySpec,
        work: &Path,
    ) -> Result<Self, String> {
        let acceptor = transport::certificate(work).await?;
        let listener = TcpListener::bind("127.0.0.1:0").await.map_err(text)?;
        let endpoint = format!("https://{}", listener.local_addr().map_err(text)?);
        let mut config = Config::new(upstream, SEED);
        config.ca_bundle = Some(ca.into());
        config.latency = vec![(
            ScopeFilter::default(),
            Duration::from_millis(spec.latency_ms),
        )];
        config.rate_limits = vec![RateLimit {
            filter: ScopeFilter::default(),
            requests_per_second: spec.rps,
        }];
        config.log_capacity = 1;
        let proxy = Proxy::start(config).await.map_err(text)?;
        let cancel = CancellationToken::new();
        let listener = tokio::spawn(transport::serve(
            listener,
            acceptor,
            proxy.address(),
            cancel.clone(),
        ));
        Ok(Self {
            proxy: Some(proxy),
            endpoint,
            ca: work.join("proxy-ca.pem"),
            cancel,
            listener: Some(listener),
            spec,
        })
    }
    pub(crate) fn endpoint(&self) -> &str {
        &self.endpoint
    }
    pub(crate) fn ca_bundle(&self) -> &Path {
        &self.ca
    }
    pub(crate) async fn sweep(
        &self,
        command: tokio::process::Command,
        log: &Path,
        shutdown: &CancellationToken,
    ) -> Result<(Finished, Extra), String> {
        let proxy = self.proxy.as_ref().ok_or("proxy missing")?;
        let cancel = CancellationToken::new();
        let collect = async {
            let outcome = collector::sweep(command, log, shutdown).await;
            cancel.cancel();
            outcome
        };
        let (finished, samples) = tokio::join!(collect, record(proxy, cancel.clone()));
        let samples = samples?;
        tokio::fs::write(log.with_extension("proxy.tsv"), render(&samples))
            .await
            .map_err(text)?;
        Ok((finished?, series::measurements(&samples, self.spec.rps)?))
    }
    pub(crate) async fn shutdown(mut self) -> Result<transport::Report, String> {
        self.cancel.cancel();
        let listener = match self.listener.take() {
            Some(owner) => owner.await.map_err(text).and_then(|result| result),
            None => Err("proxy listener missing".into()),
        };
        let proxy = self
            .proxy
            .take()
            .ok_or("proxy missing")?
            .shutdown()
            .await
            .map_err(text);
        let report = proxy?;
        let mut listener = listener?;
        listener.connection_failures = listener
            .connection_failures
            .checked_add(report.failed)
            .ok_or("connection failure counter overflow")?;
        if report.aborted != 0 || report.panicked != 0 {
            return Err(format!("proxy drain failed: {report:?}"));
        }
        Ok(listener)
    }
}
impl Drop for Front {
    fn drop(&mut self) {
        self.cancel.cancel();
        if let Some(owner) = &self.listener {
            owner.abort();
        }
    }
}
fn capture(proxy: &Proxy, started: Instant) -> Result<series::Sample, String> {
    let snapshot = proxy.snapshot().map_err(text)?;
    let elapsed = u64::try_from(started.elapsed().as_millis()).map_err(text)?;
    Ok((
        elapsed,
        snapshot.counters.responses_written,
        snapshot.counters.rate_limited,
    ))
}
async fn record(proxy: &Proxy, cancel: CancellationToken) -> Result<Vec<series::Sample>, String> {
    let started = Instant::now();
    let (_, responses, throttles) = capture(proxy, started)?;
    let ticks = stream::unfold(
        (tokio::time::interval(Duration::from_millis(100)), cancel),
        |(mut interval, cancel)| async move {
            tokio::select! {
                biased;
                () = cancel.cancelled() => None,
                _ = interval.tick() => Some(((), (interval, cancel))),
            }
        },
    );
    let mut samples = ticks
        .take(36_001)
        .map(|()| capture(proxy, started))
        .try_fold(
            vec![(0, responses, throttles)],
            |mut samples, sample| async move {
                append(&mut samples, sample)?;
                Ok(samples)
            },
        )
        .await?;
    append(&mut samples, capture(proxy, started)?)?;
    Ok(samples)
}
fn append(samples: &mut Vec<series::Sample>, sample: series::Sample) -> Result<(), String> {
    if let Some(last) = samples.last_mut() {
        if last.0 == sample.0 {
            *last = sample;
            return Ok(());
        }
        if last.0 > sample.0 {
            return Err("proxy sample clock moved backwards".into());
        }
    }
    samples.try_reserve(1).map_err(text)?;
    samples.push(sample);
    Ok(())
}
fn render(samples: &[series::Sample]) -> String {
    samples.iter().fold(
        "elapsed_ms\tresponses\tthrottles\n".into(),
        |mut output, (time, responses, throttles)| {
            output.push_str(&format!("{time}\t{responses}\t{throttles}\n"));
            output
        },
    )
}
fn text(error: impl std::fmt::Display) -> String {
    error.to_string()
}
