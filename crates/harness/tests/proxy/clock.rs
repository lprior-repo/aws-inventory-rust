//! Keep paused Tokio clocks from automatically leaping over a TCP exchange.
//! Real sockets wake from the OS, not virtual time; an owned runnable checkpoint
//! task makes advance() the only clock driver while each timing scenario runs.
use std::{
    error::Error,
    future::Future,
    time::{Duration, Instant},
};

const WATCHDOG: Duration = Duration::from_secs(3);
const SCENARIO_BOUND: Duration = Duration::from_secs(10);

pub struct FrozenClock(tokio::task::JoinHandle<()>);
impl FrozenClock {
    pub fn new() -> Self {
        Self(tokio::spawn(async {
            let started = Instant::now();
            while started.elapsed() < SCENARIO_BOUND {
                tokio::task::yield_now().await;
            }
        }))
    }
}
impl Drop for FrozenClock {
    fn drop(&mut self) {
        self.0.abort();
    }
}

pub async fn within<T, E>(
    label: &str,
    future: impl Future<Output = Result<T, E>>,
) -> Result<T, Box<dyn Error>>
where
    E: Into<Box<dyn Error>>,
{
    tokio::select! {
        result = future => result.map_err(Into::into),
        error = watchdog(label) => Err(error),
    }
}

async fn watchdog(label: &str) -> Box<dyn Error> {
    let started = Instant::now();
    while started.elapsed() < WATCHDOG {
        tokio::task::yield_now().await;
    }
    format!("watchdog expired: {label}").into()
}

pub async fn until(
    label: &str,
    mut condition: impl FnMut() -> Result<bool, Box<dyn Error>>,
) -> super::support::TestResult {
    within(label, async {
        while !condition()? {
            tokio::task::yield_now().await;
        }
        Ok::<(), Box<dyn Error>>(())
    })
    .await
}
