use super::super::worker::CreateOutcome;
use super::{Actions, CleanupFailures, TestDatabase, names::DatabaseName};
use crate::HarnessError;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

const ADMIN: &str = "postgres://postgres@localhost/postgres?sslmode=disable";

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_nonpermission_create_failure_when_returning_error_then_original_error_is_preserved()
-> TestResult {
    let original = HarnessError::Sqlx(sqlx::Error::Protocol("creation connection lost".into()));
    let expected = original.to_string();
    let error = super::super::worker::creation_failure(original);
    assert_eq!(error.to_string(), expected);
    Ok(())
}

struct ExistingDatabase {
    admin: String,
    name: String,
}

impl super::Actions for ExistingDatabase {
    async fn create(&self) -> CreateOutcome {
        CreateOutcome::Created { close: Ok(()) }
    }
    async fn remove(&self) -> Result<(), HarnessError> {
        super::drop_database(&self.admin, &self.name).await
    }
}

#[tokio::test]
async fn given_unreachable_admin_when_drop_fails_then_run_report_names_database_and_fails()
-> TestResult {
    let socket = tokio::net::TcpSocket::new_v4()?;
    socket.bind("127.0.0.1:0".parse()?)?;
    let address = socket.local_addr()?;
    let admin = format!("postgres://postgres@{address}/postgres?sslmode=disable");
    let name = super::names::DatabaseName::at(0);
    let expected_name = name.as_str().to_owned();
    let failures = super::CleanupFailures::default();
    let database = super::TestDatabase::create_owned(
        &admin,
        name,
        &failures,
        ExistingDatabase {
            admin: admin.clone(),
            name: expected_name.clone(),
        },
    )
    .await?;
    drop(database);
    let root = tempfile::tempdir()?;
    let original = std::net::TcpStream::connect(address)
        .err()
        .ok_or("expected connection refusal")?;
    let reason = HarnessError::from(sqlx::Error::Io(original)).to_string();
    let error = crate::head_to_head::write_outcome(
        root.path(),
        Ok(serde_json::json!({
            "comparison_ready": true, "dataset": {"seeded_counts": {"AWS::S3::Bucket": 2}},
            "tools": [{"comparison_timing": {"median_seconds": 1.0}}]
        })),
        &failures,
    )
    .err()
    .ok_or("expected a failed run")?;
    let expected_error = format!("database {expected_name} cleanup failed: {reason}");
    assert_eq!(error.to_string(), expected_error);
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.path().join("report.json"))?)?;
    assert_eq!(
        report.get("comparison_ready"),
        Some(&serde_json::json!(false))
    );
    assert_eq!(
        report.get("cleanup_failures"),
        Some(&serde_json::json!([
            {"database": expected_name, "reason": reason}
        ]))
    );
    assert_eq!(
        report.pointer("/dataset/seeded_counts/AWS::S3::Bucket"),
        Some(&serde_json::json!(2))
    );
    assert_eq!(
        report.pointer("/tools/0/comparison_timing"),
        Some(&serde_json::Value::Null)
    );
    Ok(())
}

struct FailedCreate {
    failed_at: CreateFailure,
    removals: Arc<AtomicUsize>,
}

enum CreateFailure {
    Prepare,
    LostReply { backend: Backend },
}

enum Backend {
    Exited,
    ExitUnconfirmed,
}

impl Actions for FailedCreate {
    async fn create(&self) -> CreateOutcome {
        match self.failed_at {
            CreateFailure::Prepare => CreateOutcome::NotCreated(HarnessError::Sqlx(
                sqlx::Error::Io(std::io::ErrorKind::ConnectionRefused.into()),
            )),
            CreateFailure::LostReply { ref backend } => CreateOutcome::Unconfirmed {
                error: HarnessError::Sqlx(sqlx::Error::Io(
                    std::io::ErrorKind::ConnectionReset.into(),
                )),
                settlement: match backend {
                    Backend::Exited => Ok(()),
                    Backend::ExitUnconfirmed => Err(HarnessError::invalid("backend still visible")),
                },
            },
        }
    }

    async fn remove(&self) -> Result<(), HarnessError> {
        self.removals.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn given_pre_create_failure_when_creating_owned_database_then_returns_error_without_drop()
-> TestResult {
    let removals = Arc::new(AtomicUsize::new(0));
    let failures = CleanupFailures::default();
    let result = TestDatabase::create_owned(
        ADMIN,
        DatabaseName::at(0),
        &failures,
        FailedCreate {
            failed_at: CreateFailure::Prepare,
            removals: removals.clone(),
        },
    )
    .await;
    assert!(matches!(
        result,
        Err(HarnessError::Sqlx(sqlx::Error::Io(error)))
            if error.kind() == std::io::ErrorKind::ConnectionRefused
    ));
    assert_eq!(removals.load(Ordering::SeqCst), 0);
    assert!(failures.failures().is_empty());
    Ok(())
}

#[tokio::test]
async fn given_lost_create_reply_when_backend_exits_then_run_reports_database_without_drop()
-> TestResult {
    let removals = Arc::new(AtomicUsize::new(0));
    let failures = CleanupFailures::default();
    let name = DatabaseName::at(0);
    let expected_name = name.as_str().to_owned();
    let error = TestDatabase::create_owned(
        ADMIN,
        name,
        &failures,
        FailedCreate {
            failed_at: CreateFailure::LostReply {
                backend: Backend::Exited,
            },
            removals: removals.clone(),
        },
    )
    .await
    .err()
    .ok_or("expected unconfirmed creation to fail")?;
    assert!(matches!(
        &error,
        HarnessError::Sqlx(sqlx::Error::Io(error))
            if error.kind() == std::io::ErrorKind::ConnectionReset
    ));
    assert_eq!(removals.load(Ordering::SeqCst), 0);

    let root = tempfile::tempdir()?;
    assert!(crate::head_to_head::write_outcome(root.path(), Err(error), &failures).is_err());
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.path().join("report.json"))?)?;
    let reported = report
        .get("cleanup_failures")
        .and_then(serde_json::Value::as_array)
        .ok_or("expected cleanup failures in report")?;
    assert_eq!(reported.len(), 1);
    assert_eq!(
        reported.first().and_then(|entry| entry.get("database")),
        Some(&serde_json::json!(expected_name))
    );
    assert_eq!(
        report.get("comparison_ready"),
        Some(&serde_json::json!(false))
    );
    Ok(())
}

#[tokio::test]
async fn given_lost_create_reply_when_backend_exit_is_unconfirmed_then_cleanup_is_reported()
-> TestResult {
    let removals = Arc::new(AtomicUsize::new(0));
    let failures = CleanupFailures::default();
    let name = DatabaseName::at(0);
    let expected_name = name.as_str().to_owned();
    let result = TestDatabase::create_owned(
        ADMIN,
        name,
        &failures,
        FailedCreate {
            failed_at: CreateFailure::LostReply {
                backend: Backend::ExitUnconfirmed,
            },
            removals: removals.clone(),
        },
    )
    .await;
    assert!(matches!(
        result,
        Err(HarnessError::Sqlx(sqlx::Error::Io(error)))
            if error.kind() == std::io::ErrorKind::ConnectionReset
    ));
    assert_eq!(removals.load(Ordering::SeqCst), 0);
    let recorded = failures.failures();
    assert_eq!(recorded.len(), 1);
    assert_eq!(
        recorded.first().map(|entry| entry.database.as_str()),
        Some(expected_name.as_str())
    );
    Ok(())
}
