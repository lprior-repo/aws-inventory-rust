//! Deterministic behaviour of the `serve` scheduling core on a virtual clock.
//! No test sleeps: the simulated environment advances its own clock.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, VecDeque};
use std::time::Duration;

use inventory_engine::cadence::{
    Action, CadenceError, Environment, MAX_PROFILES, Millis, RunOutcome, Scheduler, ServeReport,
    Wake, serve,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Event {
    Start { profile: usize, at: u64 },
    End { profile: usize, at: u64 },
    Sleep { from: u64, until: u64 },
}

/// What the simulated profile run does.
#[derive(Clone, Copy, Debug)]
struct Script {
    duration: u64,
    outcome: RunOutcome,
}

/// Virtual-clock environment. Runs and sleeps advance `now` and nothing else.
struct Sim {
    now: Cell<u64>,
    events: RefCell<Vec<Event>>,
    scripts: RefCell<VecDeque<Script>>,
    /// Freshness the store reports at start; `None` means nothing was recorded.
    seed: RefCell<Option<Vec<Option<Millis>>>>,
    default_script: Script,
    running: Cell<bool>,
    overlaps: Cell<u32>,
    stop_at: Cell<Option<u64>>,
    stop_after_runs: Cell<Option<usize>>,
    runs: Cell<usize>,
}

impl Sim {
    fn new(default_duration: u64) -> Self {
        Self {
            now: Cell::new(0),
            events: RefCell::new(Vec::new()),
            scripts: RefCell::new(VecDeque::new()),
            seed: RefCell::new(None),
            default_script: Script {
                duration: default_duration,
                outcome: RunOutcome::Completed,
            },
            running: Cell::new(false),
            overlaps: Cell::new(0),
            stop_at: Cell::new(None),
            stop_after_runs: Cell::new(None),
            runs: Cell::new(0),
        }
    }

    fn script(&self, script: Script) {
        self.scripts.borrow_mut().push_back(script);
    }

    fn seed_freshness(&self, last_completed: Vec<Option<Millis>>) {
        *self.seed.borrow_mut() = Some(last_completed);
    }

    fn events(&self) -> Vec<Event> {
        self.events.borrow().clone()
    }

    fn starts(&self) -> Vec<(usize, u64)> {
        self.events()
            .into_iter()
            .filter_map(|event| match event {
                Event::Start { profile, at } => Some((profile, at)),
                _ => None,
            })
            .collect()
    }
}

impl Environment for Sim {
    fn now(&self) -> Millis {
        Millis::new(self.now.get())
    }

    async fn last_completed(&self, profiles: usize) -> Vec<Option<Millis>> {
        self.seed
            .borrow()
            .clone()
            .unwrap_or_else(|| vec![None; profiles])
    }

    fn stop_requested(&self) -> bool {
        self.stop_after_runs
            .get()
            .is_some_and(|limit| self.runs.get() >= limit)
    }

    async fn sleep_until(&self, until: Millis) -> Wake {
        let from = self.now.get();
        self.events.borrow_mut().push(Event::Sleep {
            from,
            until: until.get(),
        });
        match self.stop_at.get() {
            Some(stop) if stop <= until.get() => {
                self.now.set(stop.max(from));
                Wake::Stop
            }
            _ => {
                self.now.set(until.get());
                Wake::Elapsed
            }
        }
    }

    async fn run_profile(&self, profile: usize) -> RunOutcome {
        if self.running.replace(true) {
            self.overlaps.set(self.overlaps.get().saturating_add(1));
        }
        let script = self
            .scripts
            .borrow_mut()
            .pop_front()
            .unwrap_or(self.default_script);
        let start = self.now.get();
        self.events
            .borrow_mut()
            .push(Event::Start { profile, at: start });
        self.now.set(start.saturating_add(script.duration));
        self.events.borrow_mut().push(Event::End {
            profile,
            at: self.now.get(),
        });
        self.runs.set(self.runs.get().saturating_add(1));
        self.running.set(false);
        script.outcome
    }
}

fn ms(value: u64) -> Duration {
    Duration::from_millis(value)
}

/// The simulated operator asks to stop at virtual millisecond `at`.
fn stop_after(sim: &Sim, at: u64) {
    sim.stop_at.set(Some(at));
}

#[tokio::test]
async fn given_a_short_pass_when_serving_then_it_idles_until_the_next_interval() -> TestResult {
    let sim = Sim::new(10);
    stop_after(&sim, 250);
    let report = serve(&sim, ms(100), 3).await?;

    // Passes start at 0, 100 and 200; each pass is three 10 ms runs.
    let pass_starts: Vec<u64> = sim
        .starts()
        .into_iter()
        .filter(|(profile, _)| *profile == 0)
        .map(|(_, at)| at)
        .collect();
    assert_eq!(pass_starts, vec![0, 100, 200]);
    assert_eq!(report.passes, 3);
    assert_eq!(report.completed, 9);
    // Idle time is a sleep and nothing runs inside it.
    for event in sim.events() {
        if let Event::Sleep { from, until } = event {
            for (_, at) in sim.starts() {
                assert!(
                    at <= from || at >= until,
                    "a run started at {at} inside the idle window {from}..{until}"
                );
            }
        }
    }
    Ok(())
}

#[tokio::test]
async fn given_a_pass_that_overruns_the_interval_when_it_ends_then_the_next_starts_at_once()
-> TestResult {
    // Three runs of 60 ms: the pass lasts 180 ms against a 100 ms interval.
    let sim = Sim::new(60);
    sim.stop_after_runs.set(Some(6));
    let report = serve(&sim, ms(100), 3).await?;

    assert_eq!(report.passes, 2);
    assert!(
        sim.events()
            .iter()
            .all(|event| !matches!(event, Event::Sleep { .. })),
        "an overrunning pass must not be followed by an idle wait"
    );
    // The second pass starts the instant the first ends (180), not at 100 or 200.
    assert_eq!(
        sim.starts(),
        vec![(0, 0), (1, 60), (2, 120), (0, 180), (1, 240), (2, 300)]
    );
    Ok(())
}

#[tokio::test]
async fn given_a_long_overrun_when_it_ends_then_missed_intervals_are_not_replayed() -> TestResult {
    // One 450 ms run against a 100 ms interval overran four intervals.
    let sim = Sim::new(10);
    sim.script(Script {
        duration: 450,
        outcome: RunOutcome::Completed,
    });
    sim.stop_after_runs.set(Some(3));
    serve(&sim, ms(100), 1).await?;

    // Next pass starts once at 450, the one after waits a full interval from there.
    assert_eq!(sim.starts(), vec![(0, 0), (0, 450), (0, 550)]);
    assert_eq!(
        sim.events()
            .iter()
            .filter(|event| matches!(event, Event::Sleep { .. }))
            .count(),
        1
    );
    Ok(())
}

#[tokio::test]
async fn given_any_schedule_when_serving_then_runs_never_overlap() -> TestResult {
    let sim = Sim::new(37);
    sim.stop_after_runs.set(Some(25));
    serve(&sim, ms(100), 7).await?;
    assert_eq!(sim.overlaps.get(), 0);
    // Every run ends before the next begins.
    let mut last_end = 0;
    for event in sim.events() {
        match event {
            Event::Start { at, .. } => assert!(at >= last_end),
            Event::End { at, .. } => last_end = at,
            Event::Sleep { .. } => {}
        }
    }
    Ok(())
}

#[tokio::test]
async fn given_a_failed_profile_when_the_next_pass_starts_then_the_stalest_goes_first() -> TestResult
{
    let sim = Sim::new(10);
    sim.script(Script {
        duration: 10,
        outcome: RunOutcome::Completed,
    });
    sim.script(Script {
        duration: 10,
        outcome: RunOutcome::Failed,
    });
    sim.stop_after_runs.set(Some(6));
    let report = serve(&sim, ms(100), 3).await?;

    // Pass 1 fails profile 1; in pass 2 it has never completed, so it is first,
    // then profile 0 (completed at 10) before profile 2 (completed at 30).
    let order: Vec<usize> = sim
        .starts()
        .into_iter()
        .map(|(profile, _)| profile)
        .collect();
    assert_eq!(order, vec![0, 1, 2, 1, 0, 2]);
    assert_eq!(report.failed, 1);
    Ok(())
}

#[tokio::test]
async fn given_a_stop_during_idle_when_serving_then_nothing_further_starts() -> TestResult {
    let sim = Sim::new(10);
    stop_after(&sim, 40);
    let report = serve(&sim, ms(100), 2).await?;
    assert_eq!(sim.starts(), vec![(0, 0), (1, 10)]);
    assert_eq!(
        report,
        ServeReport {
            passes: 1,
            completed: 2,
            ..ServeReport::default()
        }
    );
    assert_eq!(sim.now.get(), 40);
    Ok(())
}

#[tokio::test]
async fn given_a_stop_request_between_runs_when_serving_then_the_pass_is_not_continued()
-> TestResult {
    let sim = Sim::new(10);
    sim.stop_after_runs.set(Some(1));
    let report = serve(&sim, ms(100), 3).await?;
    assert_eq!(sim.starts(), vec![(0, 0)]);
    assert_eq!(report.completed, 1);
    Ok(())
}

#[tokio::test]
async fn given_an_interrupted_run_when_serving_then_the_loop_ends_without_idling() -> TestResult {
    let sim = Sim::new(10);
    sim.script(Script {
        duration: 10,
        outcome: RunOutcome::Completed,
    });
    sim.script(Script {
        duration: 4,
        outcome: RunOutcome::Interrupted,
    });
    let report = serve(&sim, ms(100), 3).await?;
    assert_eq!(sim.starts(), vec![(0, 0), (1, 10)]);
    assert_eq!(report.interrupted, 1);
    assert!(
        sim.events()
            .iter()
            .all(|event| !matches!(event, Event::Sleep { .. }))
    );
    Ok(())
}

/// A pending sweep belongs to one profile. A restart must lose none of its work.
#[tokio::test]
async fn given_a_killed_run_when_restarted_then_the_owner_resumes_and_nothing_is_lost() -> TestResult
{
    const SCOPES: usize = 5;

    struct Store {
        /// Owner of the pending sweep, if any.
        pending_owner: Cell<Option<usize>>,
        /// Scopes of the pending sweep not yet finished.
        remaining: RefCell<BTreeSet<usize>>,
        /// Every scope run, in order, with its profile.
        executed: RefCell<Vec<(usize, usize)>>,
        /// Kill the process after this many scope runs.
        kill_after: Cell<Option<usize>>,
        stopping: Cell<bool>,
        sweeps_completed: RefCell<Vec<usize>>,
    }

    struct Restartable<'store> {
        store: &'store Store,
        now: Cell<u64>,
        passes_wanted: Cell<u32>,
    }

    impl Environment for Restartable<'_> {
        fn now(&self) -> Millis {
            Millis::new(self.now.get())
        }
        async fn last_completed(&self, profiles: usize) -> Vec<Option<Millis>> {
            vec![None; profiles]
        }
        fn stop_requested(&self) -> bool {
            self.store.stopping.get() || self.passes_wanted.get() == 0
        }
        async fn sleep_until(&self, until: Millis) -> Wake {
            self.now.set(until.get());
            self.passes_wanted
                .set(self.passes_wanted.get().saturating_sub(1));
            Wake::Elapsed
        }
        async fn run_profile(&self, profile: usize) -> RunOutcome {
            let store = self.store;
            match store.pending_owner.get() {
                Some(owner) if owner != profile => return RunOutcome::NotOwner,
                Some(_) => {}
                None => {
                    store.pending_owner.set(Some(profile));
                    *store.remaining.borrow_mut() = (0..SCOPES).collect();
                }
            }
            loop {
                let next = store.remaining.borrow().first().copied();
                let Some(scope) = next else { break };
                if store.kill_after.get() == Some(store.executed.borrow().len()) {
                    store.kill_after.set(None);
                    store.stopping.set(true);
                    return RunOutcome::Interrupted;
                }
                store.executed.borrow_mut().push((profile, scope));
                store.remaining.borrow_mut().remove(&scope);
            }
            store.pending_owner.set(None);
            store.sweeps_completed.borrow_mut().push(profile);
            RunOutcome::Completed
        }
    }

    let store = Store {
        pending_owner: Cell::new(None),
        remaining: RefCell::new(BTreeSet::new()),
        executed: RefCell::new(Vec::new()),
        kill_after: Cell::new(Some(SCOPES + 2)),
        stopping: Cell::new(false),
        sweeps_completed: RefCell::new(Vec::new()),
    };

    // First process: profile 0 completes (5 scopes), profile 1 is killed after 2.
    let first = Restartable {
        store: &store,
        now: Cell::new(0),
        passes_wanted: Cell::new(3),
    };
    let report = serve(&first, ms(100), 3).await?;
    assert_eq!(report.interrupted, 1);
    assert_eq!(store.pending_owner.get(), Some(1));
    assert_eq!(store.remaining.borrow().len(), SCOPES - 2);

    // Second process: starts at profile order [0, 1, 2]. Profile 0 is not the
    // owner and is deferred, profile 1 resumes, profile 2 sweeps, then 0 is retried.
    store.stopping.set(false);
    let second = Restartable {
        store: &store,
        now: Cell::new(0),
        passes_wanted: Cell::new(1),
    };
    let report = serve(&second, ms(100), 3).await?;
    assert_eq!(report.not_owner, 1);
    assert_eq!(*store.sweeps_completed.borrow(), vec![0, 1, 2, 0]);

    // Profile 1's interrupted sweep ran each of its scopes exactly once across both processes.
    let executed = store.executed.borrow();
    let mut profile_one: Vec<usize> = executed
        .iter()
        .filter(|(profile, _)| *profile == 1)
        .map(|(_, scope)| *scope)
        .take(SCOPES)
        .collect();
    profile_one.sort_unstable();
    assert_eq!(profile_one, (0..SCOPES).collect::<Vec<_>>());
    Ok(())
}

#[tokio::test]
async fn given_a_pending_sweep_owned_by_nobody_configured_when_serving_then_the_pass_ends()
-> TestResult {
    struct Blocked {
        now: Cell<u64>,
        attempts: Cell<u32>,
    }
    impl Environment for Blocked {
        fn now(&self) -> Millis {
            Millis::new(self.now.get())
        }
        async fn last_completed(&self, profiles: usize) -> Vec<Option<Millis>> {
            vec![None; profiles]
        }
        fn stop_requested(&self) -> bool {
            self.now.get() > 0
        }
        async fn sleep_until(&self, until: Millis) -> Wake {
            self.now.set(until.get());
            Wake::Elapsed
        }
        async fn run_profile(&self, _profile: usize) -> RunOutcome {
            self.attempts.set(self.attempts.get().saturating_add(1));
            RunOutcome::NotOwner
        }
    }
    let env = Blocked {
        now: Cell::new(0),
        attempts: Cell::new(0),
    };
    let report = serve(&env, ms(100), 3).await?;
    // With no progress by anyone, each profile is tried once; no retry storm.
    assert_eq!(env.attempts.get(), 3);
    assert_eq!(report.not_owner, 3);
    Ok(())
}

#[test]
fn given_invalid_configuration_when_building_the_scheduler_then_it_is_refused() {
    let now = Millis::new(0);
    assert_eq!(
        Scheduler::new(Duration::ZERO, 1, now).err(),
        Some(CadenceError::ZeroInterval)
    );
    assert_eq!(
        Scheduler::new(Duration::from_micros(999), 1, now).err(),
        Some(CadenceError::ZeroInterval)
    );
    assert_eq!(
        Scheduler::new(ms(1), 0, now).err(),
        Some(CadenceError::NoProfiles)
    );
    assert_eq!(
        Scheduler::new(ms(1), MAX_PROFILES.saturating_add(1), now).err(),
        Some(CadenceError::TooManyProfiles)
    );
    assert!(Scheduler::new(ms(1), MAX_PROFILES, now).is_ok());
}

#[test]
fn given_the_scheduler_when_an_outcome_is_for_the_wrong_profile_then_it_is_refused() -> TestResult {
    let mut scheduler = Scheduler::new(ms(100), 2, Millis::new(0))?;
    assert_eq!(
        scheduler.record(0, RunOutcome::Completed, Millis::new(0)),
        Err(CadenceError::UnexpectedOutcome)
    );
    assert_eq!(
        scheduler.next_action(Millis::new(0)),
        Action::Run { profile: 0 }
    );
    // Repeating the query does not advance the pass.
    assert_eq!(
        scheduler.next_action(Millis::new(5)),
        Action::Run { profile: 0 }
    );
    assert_eq!(
        scheduler.record(1, RunOutcome::Completed, Millis::new(5)),
        Err(CadenceError::UnexpectedOutcome)
    );
    scheduler.record(0, RunOutcome::Completed, Millis::new(5))?;
    assert_eq!(scheduler.last_completed(), &[Some(Millis::new(5)), None]);
    Ok(())
}

#[test]
fn given_a_stopped_scheduler_when_asked_then_it_only_stops() -> TestResult {
    let mut scheduler = Scheduler::new(ms(100), 2, Millis::new(0))?;
    scheduler.request_stop();
    assert_eq!(scheduler.next_action(Millis::new(10_000)), Action::Stop);
    Ok(())
}

/// Freshness of each profile as the first run's completions left it: the
/// time its last sweep ended. This is what a store records for a restart.
fn freshness_of(sim: &Sim, profiles: usize) -> Vec<Option<Millis>> {
    let mut last = vec![None; profiles];
    for event in sim.events() {
        if let Event::End { profile, at } = event
            && let Some(slot) = last.get_mut(profile)
        {
            *slot = Some(Millis::new(at));
        }
    }
    last
}

#[tokio::test]
async fn given_freshness_from_the_store_when_serving_then_the_stalest_profile_runs_first()
-> TestResult {
    let sim = Sim::new(10);
    sim.seed_freshness(vec![Some(Millis::new(300)), None, Some(Millis::new(100))]);
    sim.stop_after_runs.set(Some(3));
    serve(&sim, ms(100), 3).await?;
    let order: Vec<usize> = sim.starts().into_iter().map(|(profile, _)| profile).collect();
    assert_eq!(order, vec![1, 2, 0]);
    Ok(())
}

#[tokio::test]
async fn given_freshness_for_another_profile_count_when_serving_then_it_is_refused() -> TestResult {
    let sim = Sim::new(10);
    sim.seed_freshness(vec![None, None]);
    assert_eq!(
        serve(&sim, ms(100), 3).await.err(),
        Some(CadenceError::FreshnessMismatch)
    );
    assert!(sim.starts().is_empty());
    Ok(())
}

#[tokio::test]
async fn given_a_stop_and_a_restart_when_serving_then_no_profile_is_skipped_and_the_stalest_runs_first()
-> TestResult {
    // First process: profiles 0 and 1 complete, then the operator stops it.
    let first = Sim::new(10);
    first.stop_after_runs.set(Some(2));
    serve(&first, ms(100), 3).await?;
    let freshness = freshness_of(&first, 3);
    assert_eq!(freshness, vec![Some(Millis::new(10)), Some(Millis::new(20)), None]);

    // Restart from the stored freshness: never-run profile 2 first, then 0, then 1.
    let second = Sim::new(10);
    second.seed_freshness(freshness);
    second.stop_after_runs.set(Some(3));
    serve(&second, ms(100), 3).await?;
    let order: Vec<usize> = second.starts().into_iter().map(|(profile, _)| profile).collect();
    assert_eq!(order, vec![2, 0, 1]);
    Ok(())
}
