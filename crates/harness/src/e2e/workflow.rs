use crate::HarnessError;
use futures_util::{StreamExt, TryStreamExt};
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Stage {
    Seed,
    Sweep,
    Counts,
    Resweep,
}

impl Stage {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Seed => "seed",
            Self::Sweep => "sweep",
            Self::Counts => "verify-counts",
            Self::Resweep => "resweep",
        }
    }
}

pub(super) trait Session: Sized {
    async fn stage(&mut self, stage: Stage) -> Result<(), HarnessError>;
    async fn finish(self, interrupted: bool) -> Result<(), HarnessError>;
}

pub(super) async fn drive(mut session: impl Session) -> Result<(), HarnessError> {
    let (result, interrupted) = tokio::select! {
        result = stages(&mut session) => (result, false),
        signal = tokio::signal::ctrl_c() => (signal.and_then(|()| Err(std::io::Error::other("e2e run interrupted"))).map_err(HarnessError::from), true),
    };
    let started = Instant::now();
    let stopped = session.finish(interrupted).await;
    report("stop", started, &stopped);
    match (result, stopped) {
        (Err(error), Err(cleanup)) => Err(HarnessError::invalid(format!(
            "{error}; cleanup: {cleanup}"
        ))),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Ok(()), Ok(())) => Ok(()),
    }
}

async fn stages(session: &mut impl Session) -> Result<(), HarnessError> {
    futures_util::stream::iter([Stage::Seed, Stage::Sweep, Stage::Counts, Stage::Resweep])
        .map(Ok::<_, HarnessError>)
        .try_fold(session, |session, stage| async move {
            let started = Instant::now();
            let result = session.stage(stage).await;
            report(stage.name(), started, &result);
            result?;
            Ok(session)
        })
        .await
        .map(|_| ())
}

pub(super) fn report<T>(name: &str, started: Instant, result: &Result<T, HarnessError>) {
    let outcome = if result.is_ok() { "ok" } else { "failed" };
    println!("stage {name} {} {outcome}", started.elapsed().as_millis());
}
