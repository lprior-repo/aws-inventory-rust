use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

struct DelayedCreate {
    started: mpsc::SyncSender<()>,
    release: mpsc::Receiver<()>,
    live: Arc<AtomicBool>,
    removals: Arc<AtomicUsize>,
}

impl Actions for DelayedCreate {
    async fn create(&self) -> CreateOutcome {
        if let Err(error) = self.started.send(()) {
            return CreateOutcome::NotCreated(HarnessError::invalid(error.to_string()));
        }
        if let Err(error) = self.release.recv_timeout(std::time::Duration::from_secs(5)) {
            return CreateOutcome::NotCreated(HarnessError::invalid(error.to_string()));
        }
        self.live.store(true, Ordering::SeqCst);
        CreateOutcome::Created { close: Ok(()) }
    }

    async fn remove(&self) -> Result<(), HarnessError> {
        self.removals.fetch_add(1, Ordering::SeqCst);
        self.live.store(false, Ordering::SeqCst);
        Ok(())
    }
}

fn delayed() -> (DelayedCreate, mpsc::Receiver<()>, mpsc::SyncSender<()>) {
    let (started, observed) = mpsc::sync_channel(1);
    let (release, gate) = mpsc::sync_channel(1);
    (
        DelayedCreate {
            started,
            release: gate,
            live: Arc::new(AtomicBool::new(false)),
            removals: Arc::new(AtomicUsize::new(0)),
        },
        observed,
        release,
    )
}

#[tokio::test]
async fn given_delayed_create_when_wait_times_out_then_cleanup_runs_after_creation_settles()
-> TestResult {
    let (actions, started, release) = delayed();
    let live = actions.live.clone();
    let removals = actions.removals.clone();
    let failures = CleanupFailures::default();
    let (worker, ready) = Worker::start("delayed-database".into(), failures.clone(), actions)?;
    started.recv_timeout(std::time::Duration::from_secs(5))?;
    tokio::time::pause();
    let timed_out = {
        let wait = super::super::database::bounded_ddl(async {
            ready
                .await
                .map_err(|_| HarnessError::invalid("worker stopped before delivering readiness"))?
        });
        tokio::pin!(wait);
        assert!(futures_util::poll!(&mut wait).is_pending());
        tokio::time::advance(std::time::Duration::from_secs(64)).await;
        assert!(futures_util::poll!(&mut wait).is_pending());
        tokio::time::advance(std::time::Duration::from_secs(2)).await;
        tokio::time::timeout(std::time::Duration::from_secs(1), wait).await
    };
    let cleanup = std::thread::spawn(move || drop(worker));
    release.send(())?;
    cleanup.join().map_err(|_| "cleanup thread failed")?;
    assert!(matches!(timed_out, Ok(Err(HarnessError::Invalid(_)))));
    assert!(!live.load(Ordering::SeqCst));
    assert_eq!(removals.load(Ordering::SeqCst), 1);
    assert_eq!(failures.failures().len(), 0);
    Ok(())
}

#[tokio::test]
async fn given_delayed_create_when_wait_is_cancelled_then_cleanup_runs_after_creation_settles()
-> TestResult {
    let (actions, started, release) = delayed();
    let live = actions.live.clone();
    let removals = actions.removals.clone();
    let failures = CleanupFailures::default();
    let (worker, ready) = Worker::start("cancelled-database".into(), failures.clone(), actions)?;
    started.recv_timeout(std::time::Duration::from_secs(5))?;
    drop(ready);
    let cleanup = std::thread::spawn(move || drop(worker));
    release.send(())?;
    cleanup.join().map_err(|_| "cleanup thread failed")?;
    assert!(!live.load(Ordering::SeqCst));
    assert_eq!(removals.load(Ordering::SeqCst), 1);
    assert_eq!(failures.failures().len(), 0);
    Ok(())
}

struct CountsRemovals {
    close_error: Option<&'static str>,
    attempts: Arc<AtomicUsize>,
}

impl Actions for CountsRemovals {
    async fn create(&self) -> CreateOutcome {
        CreateOutcome::Created {
            close: match self.close_error {
                Some(reason) => Err(HarnessError::invalid(reason)),
                None => Ok(()),
            },
        }
    }
    async fn remove(&self) -> Result<(), HarnessError> {
        self.attempts.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn given_successful_create_when_owner_finishes_then_database_is_removed() -> TestResult {
    let attempts = Arc::new(AtomicUsize::new(0));
    let failures = CleanupFailures::default();
    let (worker, ready) = Worker::start(
        "finished-database".into(),
        failures.clone(),
        CountsRemovals {
            close_error: None,
            attempts: attempts.clone(),
        },
    )?;
    ready.await??;
    assert_eq!(attempts.load(Ordering::SeqCst), 0);
    worker.finish().await?;
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    assert!(failures.failures().is_empty());
    Ok(())
}

#[tokio::test]
async fn given_successful_create_when_owner_keeps_then_database_is_not_removed() -> TestResult {
    let attempts = Arc::new(AtomicUsize::new(0));
    let failures = CleanupFailures::default();
    let (worker, ready) = Worker::start(
        "retained-database".into(),
        failures.clone(),
        CountsRemovals {
            close_error: None,
            attempts: attempts.clone(),
        },
    )?;
    ready.await??;
    worker.keep()?;
    assert_eq!(attempts.load(Ordering::SeqCst), 0);
    assert!(failures.failures().is_empty());
    Ok(())
}

#[tokio::test]
async fn given_acknowledged_create_when_connection_close_fails_then_database_is_removed()
-> TestResult {
    let attempts = Arc::new(AtomicUsize::new(0));
    let failures = CleanupFailures::default();
    let (worker, ready) = Worker::start(
        "acknowledged-database".into(),
        failures.clone(),
        CountsRemovals {
            close_error: Some("connection close failed"),
            attempts: attempts.clone(),
        },
    )?;
    assert!(matches!(ready.await?, Err(HarnessError::Invalid(_))));
    drop(worker);
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    assert!(failures.failures().is_empty());
    Ok(())
}

struct FailedRemove(Arc<AtomicUsize>);
impl Actions for FailedRemove {
    async fn create(&self) -> CreateOutcome {
        CreateOutcome::Created { close: Ok(()) }
    }
    async fn remove(&self) -> Result<(), HarnessError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err(HarnessError::invalid("drop refused"))
    }
}

#[tokio::test]
async fn given_failed_remove_when_finishing_owner_then_only_one_drop_is_attempted() -> TestResult {
    let attempts = Arc::new(AtomicUsize::new(0));
    let failures = CleanupFailures::default();
    let (worker, ready) = Worker::start(
        "failed-database".into(),
        failures.clone(),
        FailedRemove(attempts.clone()),
    )?;
    ready.await??;
    let error = worker.finish().await.err().ok_or("expected an error")?;
    assert_eq!(error.to_string(), "drop refused");
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    let recorded = failures.failures();
    assert_eq!(recorded.len(), 1);
    assert_eq!(
        recorded
            .first()
            .map(|failure| (failure.database.as_str(), failure.reason.as_str())),
        Some(("failed-database", "drop refused"))
    );
    Ok(())
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
struct SqlState {
    code: &'static str,
    message: &'static str,
}

impl sqlx::error::DatabaseError for SqlState {
    fn message(&self) -> &str {
        self.message
    }
    fn code(&self) -> Option<std::borrow::Cow<'_, str>> {
        Some(self.code.into())
    }
    fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        self
    }
    fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
        self
    }
    fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
        self
    }
    fn kind(&self) -> sqlx::error::ErrorKind {
        sqlx::error::ErrorKind::Other
    }
}

#[test]
fn given_create_sqlstates_when_reporting_failure_then_only_permission_mentions_createdb()
-> TestResult {
    for (code, expected) in [
        (
            "42501",
            "CREATE DATABASE failed; INVENTORY_POSTGRES_URL must connect as a role with CREATEDB privilege",
        ),
        (
            "42P04",
            "CREATE DATABASE failed (SQLSTATE 42P04): original creation error",
        ),
        (
            "XX000",
            "error returned from database: original creation error",
        ),
        (
            "08001",
            "error returned from database: original creation error",
        ),
    ] {
        let error = HarnessError::from(sqlx::Error::Database(Box::new(SqlState {
            code,
            message: "original creation error",
        })));
        assert_eq!(creation_failure(error).to_string(), expected);
    }
    Ok(())
}

struct RejectedCreate {
    code: &'static str,
    attempts: Arc<AtomicUsize>,
}

impl Actions for RejectedCreate {
    async fn create(&self) -> CreateOutcome {
        CreateOutcome::NotCreated(
            sqlx::Error::Database(Box::new(SqlState {
                code: self.code,
                message: "database already exists",
            }))
            .into(),
        )
    }
    async fn remove(&self) -> Result<(), HarnessError> {
        self.attempts.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn given_duplicate_create_when_owner_is_dropped_then_existing_database_is_not_dropped()
-> TestResult {
    let attempts = Arc::new(AtomicUsize::new(0));
    let failures = CleanupFailures::default();
    let (worker, ready) = Worker::start(
        "existing-database".into(),
        failures.clone(),
        RejectedCreate {
            code: "42P04",
            attempts: attempts.clone(),
        },
    )?;
    let error = ready.await?.err().ok_or("expected duplicate error")?;
    assert_eq!(
        error.to_string(),
        "CREATE DATABASE failed (SQLSTATE 42P04): database already exists"
    );
    drop(worker);
    assert_eq!(attempts.load(Ordering::SeqCst), 0);
    assert_eq!(failures.failures().len(), 0);
    Ok(())
}

#[tokio::test]
async fn given_catalog_unique_violation_when_owner_finishes_then_database_is_not_dropped()
-> TestResult {
    let attempts = Arc::new(AtomicUsize::new(0));
    let failures = CleanupFailures::default();
    let (worker, ready) = Worker::start(
        "concurrent-database".into(),
        failures.clone(),
        RejectedCreate {
            code: "23505",
            attempts: attempts.clone(),
        },
    )?;
    assert!(matches!(
        ready.await?,
        Err(HarnessError::Sqlx(sqlx::Error::Database(error)))
            if error.code().as_deref() == Some("23505")
    ));
    worker.finish().await?;
    assert_eq!(attempts.load(Ordering::SeqCst), 0);
    assert!(failures.failures().is_empty());
    Ok(())
}
