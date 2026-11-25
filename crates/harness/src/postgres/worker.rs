use super::CleanupFailures;
use crate::HarnessError;
use std::{sync::mpsc, thread::JoinHandle};
use tokio::sync::oneshot;

pub(crate) enum CreateOutcome {
    Created {
        close: Result<(), HarnessError>,
    },
    NotCreated(HarnessError),
    Unconfirmed {
        error: HarnessError,
        settlement: Result<(), HarnessError>,
    },
}

pub(crate) trait Actions: Send + 'static {
    async fn create(&self) -> CreateOutcome;
    async fn remove(&self) -> Result<(), HarnessError>;
}

pub(crate) struct Worker {
    keep: Option<mpsc::SyncSender<()>>,
    finished: Option<oneshot::Receiver<Result<(), HarnessError>>>,
    thread: Option<JoinHandle<()>>,
    name: String,
    failures: CleanupFailures,
}

impl Worker {
    pub(crate) fn start(
        name: String,
        failures: CleanupFailures,
        actions: impl Actions,
    ) -> Result<(Self, oneshot::Receiver<Result<(), HarnessError>>), HarnessError> {
        let (created, ready) = oneshot::channel();
        let (keep, command) = mpsc::sync_channel(1);
        let (done, finished) = oneshot::channel();
        let worker_name = name.clone();
        let worker_failures = failures.clone();
        let thread = std::thread::Builder::new()
            .name("postgres-owner".into())
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build();
                serve(
                    runtime,
                    actions,
                    &worker_name,
                    &worker_failures,
                    created,
                    command,
                    done,
                );
            })?;
        Ok((
            Self {
                keep: Some(keep),
                finished: Some(finished),
                thread: Some(thread),
                name,
                failures,
            },
            ready,
        ))
    }

    pub(crate) async fn finish(mut self) -> Result<(), HarnessError> {
        drop(self.keep.take());
        let finished = self
            .finished
            .take()
            .ok_or_else(|| HarnessError::invalid("database cleanup result missing"))?;
        let result = finished.await.map_err(|_| {
            HarnessError::invalid("database worker stopped without a cleanup result")
        })?;
        self.join()?;
        result
    }

    pub(crate) fn keep(mut self) -> Result<(), HarnessError> {
        let sender = self
            .keep
            .take()
            .ok_or_else(|| HarnessError::invalid("database owner already finished"))?;
        sender
            .send(())
            .map_err(|_| HarnessError::invalid("database worker stopped before retention"))?;
        self.join()
    }

    fn join(&mut self) -> Result<(), HarnessError> {
        match self.thread.take() {
            Some(thread) => thread
                .join()
                .map_err(|_| HarnessError::invalid("PostgreSQL database worker failed")),
            None => Ok(()),
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        drop(self.keep.take());
        if let Err(error) = self.join() {
            self.failures.record(&self.name, &error);
            eprintln!(
                "PostgreSQL database {} cleanup failed: {error}; run inventory-harness postgres prune",
                self.name
            );
        }
    }
}

fn serve(
    runtime: std::io::Result<tokio::runtime::Runtime>,
    actions: impl Actions,
    name: &str,
    failures: &CleanupFailures,
    created: oneshot::Sender<Result<(), HarnessError>>,
    keep: mpsc::Receiver<()>,
    done: oneshot::Sender<Result<(), HarnessError>>,
) {
    match runtime {
        Ok(runtime) => runtime.block_on(run(actions, name, failures, created, keep, done)),
        Err(error) => {
            deliver(created, Err(error.into()));
            deliver(done, Ok(()));
        }
    }
}

async fn run(
    actions: impl Actions,
    name: &str,
    failures: &CleanupFailures,
    created: oneshot::Sender<Result<(), HarnessError>>,
    keep: mpsc::Receiver<()>,
    done: oneshot::Sender<Result<(), HarnessError>>,
) {
    let close = match actions.create().await {
        CreateOutcome::Created { close } => close,
        CreateOutcome::NotCreated(error) => {
            deliver(created, Err(creation_failure(error)));
            deliver(done, Ok(()));
            return;
        }
        CreateOutcome::Unconfirmed { error, settlement } => {
            let reason = match settlement {
                Ok(()) => format!(
                    "CREATE DATABASE outcome is unknown: {error}; verify ownership before removing"
                ),
                Err(settlement) => format!(
                    "CREATE DATABASE outcome is unknown: {error}; creating backend exit not confirmed: {settlement}; verify ownership before removing"
                ),
            };
            failures.record(name, &reason);
            eprintln!("PostgreSQL database {name}: {reason}");
            deliver(created, Err(creation_failure(error)));
            deliver(done, Err(HarnessError::invalid(reason)));
            return;
        }
    };
    deliver(created, close.map_err(creation_failure));
    let retain = keep.recv().is_ok();
    let result = if retain {
        Ok(())
    } else {
        actions.remove().await
    };
    if let Err(error) = &result {
        failures.record(name, error);
        eprintln!(
            "PostgreSQL database {name} cleanup failed: {error}; run inventory-harness postgres prune"
        );
    }
    deliver(done, result);
}

fn deliver<T>(sender: oneshot::Sender<T>, value: T) {
    match sender.send(value) {
        Ok(()) => (),
        Err(value) => drop(value),
    }
}

pub(super) fn creation_failure(error: HarnessError) -> HarnessError {
    match &error {
        HarnessError::Sqlx(sqlx::Error::Database(database))
            if database.code().as_deref() == Some("42501") =>
        {
            HarnessError::invalid(
                "CREATE DATABASE failed; INVENTORY_POSTGRES_URL must connect as a role with CREATEDB privilege",
            )
        }
        HarnessError::Sqlx(sqlx::Error::Database(database))
            if database.code().as_deref() == Some("42P04") =>
        {
            HarnessError::invalid(format!(
                "CREATE DATABASE failed (SQLSTATE 42P04): {}",
                database.message()
            ))
        }
        _ => error,
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod runtime_tests;
