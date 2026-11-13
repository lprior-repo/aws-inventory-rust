//! Immutable shared snapshot handle.

use cmdb_view::Snapshot;
use std::sync::Arc;

/// Router state: one parsed snapshot, shared read-only across tasks.
#[derive(Clone, Debug)]
pub struct AppState {
    snapshot: Arc<Snapshot>,
}

impl AppState {
    /// Wrap a parsed snapshot in shared state.
    pub fn new(snapshot: Snapshot) -> Self {
        Self {
            snapshot: Arc::new(snapshot),
        }
    }

    pub(crate) fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    pub(crate) fn fingerprint(&self) -> &str {
        self.snapshot.fingerprint()
    }
}
