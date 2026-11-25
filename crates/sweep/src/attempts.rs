use inventory_core::catalog::runtime::RuntimeCatalog;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
struct Entry {
    service: Box<str>,
    operation: Box<str>,
    sent: AtomicBool,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct AttemptedOperations {
    entries: Box<[Entry]>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum AttemptError {
    #[error("attempted operation absent from receipt table")]
    UnknownOperation,
    #[error("attempt receipt index invalid")]
    InvalidIndex,
}

impl AttemptedOperations {
    pub fn new(catalog: &RuntimeCatalog<'_>) -> Self {
        let mut entries: Vec<_> = catalog
            .plans()
            .iter()
            .map(|plan| Entry {
                service: plan.service_name().into(),
                operation: plan.operation_name().into(),
                sent: AtomicBool::new(false),
            })
            .collect();
        entries.sort_by(|a, b| (&a.service, &a.operation).cmp(&(&b.service, &b.operation)));
        entries.dedup_by(|a, b| a.service == b.service && a.operation == b.operation);
        Self {
            entries: entries.into_boxed_slice(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries
            .iter()
            .filter(|entry| entry.sent.load(Ordering::Relaxed))
            .map(|entry| (entry.service.as_ref(), entry.operation.as_ref()))
    }

    pub(crate) fn record(&self, service: &str, operation: &str) -> Result<(), AttemptError> {
        let index = self
            .entries
            .binary_search_by(|entry| {
                (entry.service.as_ref(), entry.operation.as_ref()).cmp(&(service, operation))
            })
            .map_err(|_| AttemptError::UnknownOperation)?;
        let entry = self.entries.get(index).ok_or(AttemptError::InvalidIndex)?;
        entry.sent.store(true, Ordering::Relaxed);
        Ok(())
    }
}
