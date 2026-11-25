use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Clone, Debug)]
pub struct CleanupFailure {
    pub database: String,
    pub reason: String,
}

#[derive(Clone, Default)]
pub struct CleanupFailures(Arc<Mutex<Vec<CleanupFailure>>>);

impl CleanupFailures {
    pub fn failures(&self) -> Vec<CleanupFailure> {
        self.lock().clone()
    }

    pub(crate) fn record(&self, database: &str, reason: impl ToString) {
        self.lock().push(CleanupFailure {
            database: database.into(),
            reason: reason.to_string(),
        });
    }

    fn lock(&self) -> MutexGuard<'_, Vec<CleanupFailure>> {
        match self.0.lock() {
            Ok(failures) => failures,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}
