//! Pure scheduling core for continuous collection (`serve`, ROADMAP M7.1).
//!
//! The core owns three decisions and nothing else:
//!
//! * **When** a pass over the profiles starts. A pass starts at most once per
//!   interval, measured from the start of the previous pass. A pass that is
//!   still running when the interval elapses is followed at once by the next,
//!   with no catch-up burst for the intervals it overran.
//! * **Which** profile runs next. Profiles run strictly one at a time, so passes
//!   and runs never overlap. Within a pass the stalest profile (never completed,
//!   then oldest completion) goes first. Freshness is seeded from the store, so
//!   the order survives a restart.
//! * **When to stop.** A stop request, or an interrupted run, ends the loop and
//!   starts nothing further. Work the interrupted run left pending is the
//!   store's to resume; a resumed run reports [`RunOutcome::NotOwner`] for every
//!   profile that does not own the pending work, and those are retried once the
//!   owner has made progress.
//!
//! Every effect (reading the clock, reading freshness, sleeping, running a
//! profile, noticing a stop request) is behind [`Environment`], so the loop is
//! exercised with a virtual clock and without any real sleep.

mod scheduler;
mod serve;

pub use scheduler::{Action, CadenceError, MAX_PROFILES, Millis, RunOutcome, Scheduler};
pub use serve::{Environment, ServeReport, Wake, serve};
