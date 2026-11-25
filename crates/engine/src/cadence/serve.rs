use std::time::Duration;

use super::scheduler::{Action, CadenceError, Millis, RunOutcome, Scheduler};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wake {
    Elapsed,
    Stop,
}

pub trait Environment {
    fn now(&self) -> Millis;
    fn last_completed(&self, profiles: usize) -> impl Future<Output = Vec<Option<Millis>>>;
    fn stop_requested(&self) -> bool;
    fn sleep_until(&self, until: Millis) -> impl Future<Output = Wake>;
    fn run_profile(&self, profile: usize) -> impl Future<Output = RunOutcome>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ServeReport {
    pub passes: u64,
    pub completed: u64,
    pub failed: u64,
    pub not_owner: u64,
    pub interrupted: u64,
}

impl ServeReport {
    fn count(&mut self, outcome: RunOutcome) {
        let counter = match outcome {
            RunOutcome::Completed => &mut self.completed,
            RunOutcome::Failed => &mut self.failed,
            RunOutcome::NotOwner => &mut self.not_owner,
            RunOutcome::Interrupted => &mut self.interrupted,
        };
        *counter = counter.saturating_add(1);
    }
}

pub async fn serve<E: Environment>(
    environment: &E,
    interval: Duration,
    profiles: usize,
) -> Result<ServeReport, CadenceError> {
    let seed = environment.last_completed(profiles).await;
    if seed.len() != profiles {
        return Err(CadenceError::FreshnessMismatch);
    }
    let mut scheduler = Scheduler::seeded(interval, seed, environment.now())?;
    let mut report = ServeReport::default();
    loop {
        if environment.stop_requested() {
            scheduler.request_stop();
        }
        match scheduler.next_action(environment.now()) {
            Action::Stop => {
                report.passes = scheduler.passes();
                return Ok(report);
            }
            Action::Sleep { until } => {
                if environment.sleep_until(until).await == Wake::Stop {
                    scheduler.request_stop();
                }
            }
            Action::Run { profile } => {
                let outcome = environment.run_profile(profile).await;
                report.count(outcome);
                scheduler.record(profile, outcome, environment.now())?;
            }
        }
    }
}
