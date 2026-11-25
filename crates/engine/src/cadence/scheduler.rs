use std::time::Duration;

use thiserror::Error;

pub const MAX_PROFILES: usize = 64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Millis(u64);

impl Millis {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn saturating_add(self, delta: u64) -> Self {
        Self(self.0.saturating_add(delta))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunOutcome {
    Completed,
    Failed,
    NotOwner,
    Interrupted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Run { profile: usize },
    Sleep { until: Millis },
    Stop,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum CadenceError {
    #[error("the sweep interval must be at least one millisecond")]
    ZeroInterval,
    #[error("the sweep interval exceeds the representable range")]
    IntervalTooLong,
    #[error("at least one profile is required")]
    NoProfiles,
    #[error("at most {MAX_PROFILES} profiles are supported")]
    TooManyProfiles,
    #[error("the stored freshness does not cover every profile")]
    FreshnessMismatch,
    #[error("a run outcome arrived for a profile that is not running")]
    UnexpectedOutcome,
}

#[derive(Debug)]
struct Pass {
    order: Vec<usize>,
    cursor: usize,
    next_due: Millis,
    deferred: Vec<usize>,
    progressed: bool,
}

impl Pass {
    fn current(&self) -> Option<usize> {
        self.order.get(self.cursor).copied()
    }

    fn requeue_deferred(&mut self) -> bool {
        if !self.progressed || self.deferred.is_empty() {
            return false;
        }
        self.order = std::mem::take(&mut self.deferred);
        self.cursor = 0;
        self.progressed = false;
        true
    }
}

#[derive(Debug)]
enum Phase {
    Idle { due: Millis },
    Pass(Pass),
    Stopped,
}

#[derive(Debug)]
pub struct Scheduler {
    interval_ms: u64,
    last_completed: Vec<Option<Millis>>,
    passes: u64,
    phase: Phase,
}

impl Scheduler {
    pub fn new(interval: Duration, profiles: usize, now: Millis) -> Result<Self, CadenceError> {
        check_profiles(profiles)?;
        Self::seeded(interval, vec![None; profiles], now)
    }

    pub fn seeded(
        interval: Duration,
        last_completed: Vec<Option<Millis>>,
        now: Millis,
    ) -> Result<Self, CadenceError> {
        let interval_ms = interval_millis(interval)?;
        check_profiles(last_completed.len())?;
        Ok(Self {
            interval_ms,
            last_completed,
            passes: 0,
            phase: Phase::Idle { due: now },
        })
    }

    pub fn next_action(&mut self, now: Millis) -> Action {
        loop {
            if let Some(action) = self.step(now) {
                return action;
            }
        }
    }

    pub fn record(
        &mut self,
        profile: usize,
        outcome: RunOutcome,
        now: Millis,
    ) -> Result<(), CadenceError> {
        let Phase::Pass(pass) = &mut self.phase else {
            return Err(CadenceError::UnexpectedOutcome);
        };
        if pass.current() != Some(profile) {
            return Err(CadenceError::UnexpectedOutcome);
        }
        pass.cursor = pass.cursor.saturating_add(1);
        match outcome {
            RunOutcome::Completed => {
                pass.progressed = true;
                if let Some(slot) = self.last_completed.get_mut(profile) {
                    *slot = Some(now);
                }
            }
            RunOutcome::Failed => pass.progressed = true,
            RunOutcome::NotOwner => pass.deferred.push(profile),
            RunOutcome::Interrupted => self.phase = Phase::Stopped,
        }
        Ok(())
    }

    pub fn request_stop(&mut self) {
        self.phase = Phase::Stopped;
    }

    pub fn last_completed(&self) -> &[Option<Millis>] {
        &self.last_completed
    }

    pub fn passes(&self) -> u64 {
        self.passes
    }

    fn step(&mut self, now: Millis) -> Option<Action> {
        match &mut self.phase {
            Phase::Stopped => Some(Action::Stop),
            Phase::Idle { due } if now < *due => Some(Action::Sleep { until: *due }),
            Phase::Idle { .. } => {
                self.start_pass(now);
                None
            }
            Phase::Pass(pass) => match pass.current() {
                Some(profile) => Some(Action::Run { profile }),
                None => {
                    if !pass.requeue_deferred() {
                        self.phase = Phase::Idle { due: pass.next_due };
                    }
                    None
                }
            },
        }
    }

    fn start_pass(&mut self, now: Millis) {
        self.passes = self.passes.saturating_add(1);
        self.phase = Phase::Pass(Pass {
            order: self.stalest_first(),
            cursor: 0,
            next_due: now.saturating_add(self.interval_ms),
            deferred: Vec::new(),
            progressed: false,
        });
    }

    fn stalest_first(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.last_completed.len()).collect();
        order
            .sort_by_key(|&profile| (self.last_completed.get(profile).copied().flatten(), profile));
        order
    }
}

fn interval_millis(interval: Duration) -> Result<u64, CadenceError> {
    match u64::try_from(interval.as_millis()) {
        Ok(0) => Err(CadenceError::ZeroInterval),
        Ok(millis) => Ok(millis),
        Err(_) => Err(CadenceError::IntervalTooLong),
    }
}

fn check_profiles(profiles: usize) -> Result<(), CadenceError> {
    match profiles {
        0 => Err(CadenceError::NoProfiles),
        n if n > MAX_PROFILES => Err(CadenceError::TooManyProfiles),
        _ => Ok(()),
    }
}
