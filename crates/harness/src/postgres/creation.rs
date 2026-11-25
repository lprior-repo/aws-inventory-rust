use super::{
    bounded, database,
    worker::{Actions, CreateOutcome},
};
use crate::HarnessError;
use futures_util::{StreamExt, TryStreamExt};
use sqlx::{
    Connection, Executor,
    postgres::{PgConnection, PgDatabaseError, PgSeverity},
};
use std::time::Duration;

const BACKEND_EXITED_SQL: &str =
    "SELECT NOT EXISTS(SELECT 1 FROM pg_catalog.pg_stat_activity WHERE pid = $1)";

pub(super) struct Postgres {
    pub admin: String,
    pub name: String,
}

impl Actions for Postgres {
    async fn create(&self) -> CreateOutcome {
        let (mut connection, pid) = match bounded(prepare(&self.admin)).await {
            Ok(prepared) => prepared,
            Err(error) => return CreateOutcome::NotCreated(error),
        };
        let statement = format!("CREATE DATABASE {}", self.name);
        let result = database::bounded_ddl(async {
            sqlx::raw_sql(sqlx::AssertSqlSafe(statement.as_str()))
                .execute(&mut connection)
                .await?;
            Ok(())
        })
        .await;
        match result {
            Ok(()) => {
                let close = bounded(async { Ok(connection.close().await?) }).await;
                CreateOutcome::Created { close }
            }
            Err(error) if statement_rejected(&error) => CreateOutcome::NotCreated(error),
            Err(error) => {
                drop(connection);
                CreateOutcome::Unconfirmed {
                    error,
                    settlement: bounded(settle(&self.admin, pid)).await,
                }
            }
        }
    }

    async fn remove(&self) -> Result<(), HarnessError> {
        database::drop_database(&self.admin, &self.name).await
    }
}

fn statement_rejected(error: &HarnessError) -> bool {
    let HarnessError::Sqlx(sqlx::Error::Database(error)) = error else {
        return false;
    };
    error
        .try_downcast_ref::<PgDatabaseError>()
        .is_some_and(|error| server_rejected(error.severity(), error.code()))
}

fn server_rejected(severity: PgSeverity, code: &str) -> bool {
    let completion_unknown = code.starts_with("08") || code.starts_with("57") || code == "40003";
    severity == PgSeverity::Error && !completion_unknown
}

async fn prepare(admin: &str) -> Result<(PgConnection, i32), HarnessError> {
    let mut connection = database::connect(admin).await?;
    connection.execute("SET statement_timeout = '60s'").await?;
    let pid = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut connection)
        .await?;
    Ok((connection, pid))
}

async fn settle(admin: &str, pid: i32) -> Result<(), HarnessError> {
    let connection = database::connect(admin).await?;
    let (connection, exited) = futures_util::stream::iter(0..100)
        .map(Ok::<_, HarnessError>)
        .try_fold(
            (connection, false),
            |(mut connection, exited), _| async move {
                if exited {
                    return Ok::<_, HarnessError>((connection, true));
                }
                let exited: bool = sqlx::query_scalar(BACKEND_EXITED_SQL)
                    .bind(pid)
                    .fetch_one(&mut connection)
                    .await?;
                if !exited {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Ok((connection, exited))
            },
        )
        .await?;
    connection.close().await?;
    match exited {
        true => Ok(()),
        false => Err(HarnessError::invalid(
            "CREATE DATABASE backend did not exit within 100 attempts",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqliteConnection;

    #[tokio::test]
    async fn given_disabled_activity_when_checking_settlement_then_only_backend_exit_settles()
    -> Result<(), HarnessError> {
        let mut connection = SqliteConnection::connect("sqlite::memory:").await?;
        connection
            .execute("ATTACH ':memory:' AS pg_catalog")
            .await?;
        connection
            .execute(
                "CREATE TABLE pg_catalog.pg_stat_activity (pid INTEGER, state TEXT, query TEXT)",
            )
            .await?;
        connection
            .execute("INSERT INTO pg_catalog.pg_stat_activity VALUES (1, 'disabled', '')")
            .await?;

        let query = || sqlx::query_scalar::<_, bool>(BACKEND_EXITED_SQL).bind(1);
        assert!(!query().fetch_one(&mut connection).await?);
        connection
            .execute("UPDATE pg_catalog.pg_stat_activity SET state = 'idle'")
            .await?;
        assert!(!query().fetch_one(&mut connection).await?);
        connection
            .execute("UPDATE pg_catalog.pg_stat_activity SET pid = 2")
            .await?;
        assert!(query().fetch_one(&mut connection).await?);
        Ok(())
    }

    #[test]
    fn given_duplicate_or_unique_sqlstate_when_classifying_then_server_rejected_create() {
        assert!(server_rejected(PgSeverity::Error, "42P04"));
        assert!(server_rejected(PgSeverity::Error, "23505"));
    }

    #[test]
    fn given_connection_cancel_shutdown_or_unknown_outcome_when_classifying_then_not_rejected() {
        for code in ["08000", "08006", "40003", "57014", "57P01"] {
            assert!(!server_rejected(PgSeverity::Error, code), "{code}");
        }
    }

    #[test]
    fn given_fatal_severity_when_classifying_then_not_rejected() {
        assert!(!server_rejected(PgSeverity::Fatal, "42P04"));
    }
}
