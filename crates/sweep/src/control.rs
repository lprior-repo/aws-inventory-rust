use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::mpsc::{Sender, error::TrySendError};
use tokio_util::sync::CancellationToken;

use crate::{ScopeOutcome, SweepSummary};

#[derive(Clone, Debug, Default)]
pub struct SweepControl {
    pub cancel: CancellationToken,
    pub progress: Option<Sender<ProgressEvent>>,
    /// Optional receipt, constructed from the invocation's catalog before running.
    pub attempts: Option<std::sync::Arc<crate::AttemptedOperations>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProgressEvent {
    Started {
        sweep: String,
        account: String,
        regions: u32,
        root_scopes: u64,
    },
    Scope {
        outcome: ScopeOutcome,
    },
    Page {
        rows: u64,
        resource_writes: u64,
    },
    Queue {
        pending: u64,
        in_flight: u32,
    },
    Finished {
        summary: Box<SweepSummary>,
    },
}

pub(crate) struct Progress {
    sender: Option<Sender<ProgressEvent>>,
    dropped: AtomicU64,
}

impl Progress {
    pub(crate) fn new(sender: Option<Sender<ProgressEvent>>) -> Self {
        Self {
            sender,
            dropped: AtomicU64::new(0),
        }
    }
    pub(crate) fn enabled(&self) -> bool {
        self.sender.is_some()
    }

    pub(crate) fn emit(&self, event: impl FnOnce() -> ProgressEvent) {
        let Some(sender) = &self.sender else { return };
        match sender.try_send(event()) {
            Ok(()) | Err(TrySendError::Closed(_)) => {}
            Err(TrySendError::Full(_)) => {
                match self
                    .dropped
                    .try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                        value.checked_add(1)
                    }) {
                    Ok(_) => {}
                    Err(_) => tracing::warn!("progress drop counter saturated"),
                }
            }
        }
    }
    pub(crate) fn queue(&self, pending: Option<u64>, in_flight: usize) {
        if self.sender.is_none() {
            return;
        }
        let Some(pending) = pending else { return };
        let Ok(in_flight) = u32::try_from(in_flight) else {
            tracing::error!(in_flight, "scheduler exceeds progress counter range");
            return;
        };
        self.emit(|| ProgressEvent::Queue {
            pending: pending.saturating_sub(u64::from(in_flight)),
            in_flight,
        });
    }

    pub(crate) fn finish(&self, mut summary: SweepSummary) -> SweepSummary {
        summary.metrics.progress_dropped = self.dropped.load(Ordering::Relaxed);
        self.emit(|| ProgressEvent::Finished {
            summary: Box::new(summary),
        });
        summary.metrics.progress_dropped = self.dropped.load(Ordering::Relaxed);
        summary
    }

    pub(crate) fn scope(&self, summary: &SweepSummary) {
        if !self.enabled() {
            return;
        }
        if let Some(outcome) = scope_outcome(&summary.outcomes) {
            self.emit(|| ProgressEvent::Scope { outcome });
        }
    }
}

fn scope_outcome(outcomes: &crate::ScopeOutcomes) -> Option<ScopeOutcome> {
    [
        (outcomes.complete, ScopeOutcome::Complete),
        (outcomes.partial, ScopeOutcome::Partial),
        (outcomes.inaccessible, ScopeOutcome::Inaccessible),
        (outcomes.unsupported, ScopeOutcome::Unsupported),
        (outcomes.failed, ScopeOutcome::Failed),
        (outcomes.unobserved, ScopeOutcome::Unobserved),
    ]
    .into_iter()
    .find_map(|(count, outcome)| (count != 0).then_some(outcome))
}

#[cfg(test)]
mod tests;
