use crate::HarnessError;
use futures_util::{StreamExt, TryStreamExt};
use std::{future::Future, time::Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Stage {
    Baseline,
    Seed,
    Sweep,
    Counts,
    Resweep,
}

impl Stage {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
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

pub(super) async fn drive(session: impl Session) -> Result<(), HarnessError> {
    drive_until(session, tokio::signal::ctrl_c()).await
}

async fn drive_until(
    mut session: impl Session,
    interrupt: impl Future<Output = std::io::Result<()>>,
) -> Result<(), HarnessError> {
    let (result, interrupted) = tokio::select! {
        result = stages(&mut session) => (result, false),
        signal = interrupt => (signal.and_then(|()| Err(std::io::Error::other("e2e run interrupted"))).map_err(HarnessError::from), true),
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
    futures_util::stream::iter([
        Stage::Baseline,
        Stage::Seed,
        Stage::Sweep,
        Stage::Counts,
        Stage::Resweep,
    ])
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

#[cfg(test)]
mod tests {
    use super::*;

    struct InterruptedSession<'a> {
        finished: &'a mut Option<bool>,
    }

    impl Session for InterruptedSession<'_> {
        async fn stage(&mut self, _stage: Stage) -> Result<(), HarnessError> {
            std::future::pending().await
        }

        async fn finish(self, interrupted: bool) -> Result<(), HarnessError> {
            *self.finished = Some(interrupted);
            Ok(())
        }
    }

    #[tokio::test]
    async fn given_pending_stage_when_interrupted_then_cleanup_receives_interrupted()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut finished = None;
        let error = drive_until(
            InterruptedSession {
                finished: &mut finished,
            },
            std::future::ready(Ok(())),
        )
        .await
        .err()
        .ok_or("interrupted run succeeded")?;
        assert_eq!(error.to_string(), "e2e run interrupted");
        assert_eq!(finished, Some(true));
        Ok(())
    }
}
