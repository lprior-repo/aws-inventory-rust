use sqlx::postgres::{PgConnectOptions, PgConnection};
use sqlx::{ConnectOptions, Executor};

use super::{URL_LIMIT, bounded, names};
use crate::HarnessError;

// Forced checkpoint waits during DROP exceeded the regular statement budget.
// Give DDL its own server deadline, then allow five seconds for the reply.
const DDL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(65);

#[non_exhaustive]
pub struct TestDatabase {
    name: names::DatabaseName,
    url: String,
    admin_url: String,
    cleanup: bool,
}

impl std::fmt::Debug for TestDatabase {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TestDatabase")
            .field("name", &self.name())
            .finish_non_exhaustive()
    }
}

impl TestDatabase {
    pub async fn create(admin_url: &str) -> Result<Self, HarnessError> {
        Self::create_named(admin_url, names::generate()?).await
    }

    async fn create_named(
        admin_url: &str,
        name: names::DatabaseName,
    ) -> Result<Self, HarnessError> {
        let mut connection = connect_for_ddl(admin_url).await?;
        let database = Self {
            url: database_url(admin_url, name.as_str())?,
            name,
            admin_url: admin_url.to_owned(),
            cleanup: true,
        };
        let result = execute_ddl(
            &mut connection,
            format!("CREATE DATABASE {}", database.name()),
        )
        .await;
        drop(connection);
        database.complete_creation(result)
    }

    fn complete_creation(mut self, result: Result<(), HarnessError>) -> Result<Self, HarnessError> {
        match result {
            Ok(()) => Ok(self),
            Err(error) => {
                if is_database_exists(&error) {
                    self.cleanup = false;
                }
                match &error {
                    HarnessError::Sqlx(sqlx::Error::Database(db)) => Err(HarnessError::invalid(
                        creation_reason(db.code().as_deref(), db.message()),
                    )),
                    _ => Err(error),
                }
            }
        }
    }

    #[cfg(test)]
    pub(super) async fn create_at(admin_url: &str, seconds: u32) -> Result<Self, HarnessError> {
        Self::create_named(admin_url, names::DatabaseName::at(seconds)).await
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Disable automatic deletion for an explicitly retained development run.
    pub fn keep(mut self) {
        self.cleanup = false;
    }

    pub async fn finish(mut self) -> Result<(), HarnessError> {
        drop_database(&self.admin_url, self.name()).await?;
        self.cleanup = false;
        Ok(())
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        if !self.cleanup {
            return;
        }
        if let Err(error) = cleanup_on_thread(&self.admin_url, self.name()) {
            eprintln!(
                "PostgreSQL test database cleanup failed: {error}; run inventory-harness postgres prune"
            );
        }
    }
}

fn is_database_exists(error: &HarnessError) -> bool {
    matches!(error, HarnessError::Sqlx(sqlx::Error::Database(e)) if e.code().as_deref() == Some("42P04"))
}

fn creation_reason(code: Option<&str>, message: &str) -> String {
    match code {
        Some("42501") => {
            "CREATE DATABASE failed; INVENTORY_POSTGRES_URL must connect as a role with CREATEDB privilege".into()
        }
        Some(other) => format!("CREATE DATABASE failed (SQLSTATE {other}): {message}"),
        None => format!("CREATE DATABASE failed: {message}"),
    }
}

fn cleanup_on_thread(admin_url: &str, name: &str) -> Result<(), HarnessError> {
    std::thread::scope(|scope| {
        let thread = std::thread::Builder::new()
            .name("postgres-cleanup".into())
            .spawn_scoped(scope, || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?;
                runtime.block_on(drop_database(admin_url, name))
            })?;
        thread
            .join()
            .map_err(|_| HarnessError::invalid("PostgreSQL cleanup thread failed"))?
    })
}

pub(super) fn options(url: &str) -> Result<PgConnectOptions, HarnessError> {
    if url.len() > URL_LIMIT
        || !(url.starts_with("postgres://") || url.starts_with("postgresql://"))
    {
        return Err(HarnessError::invalid(
            "INVENTORY_POSTGRES_URL must be a PostgreSQL URL of at most 8192 bytes",
        ));
    }
    url.parse::<PgConnectOptions>()
        .map(ConnectOptions::disable_statement_logging)
        .map_err(|_| HarnessError::invalid("invalid INVENTORY_POSTGRES_URL"))
}

pub(super) fn database_url(url: &str, name: &str) -> Result<String, HarnessError> {
    Ok(options(url)?.database(name).to_url_lossy().into())
}

pub(super) async fn connect(url: &str) -> Result<PgConnection, HarnessError> {
    bounded(async { Ok(options(url)?.connect().await?) }).await
}

async fn connect_for_ddl(admin_url: &str) -> Result<PgConnection, HarnessError> {
    let mut connection = connect(admin_url).await?;
    bounded(async { Ok(connection.execute("SET statement_timeout = '60s'").await?) }).await?;
    Ok(connection)
}

async fn execute_ddl(connection: &mut PgConnection, statement: String) -> Result<(), HarnessError> {
    ddl_bounded(async {
        sqlx::raw_sql(sqlx::AssertSqlSafe(statement))
            .execute(connection)
            .await?;
        Ok(())
    })
    .await
}

async fn ddl_bounded<T>(
    future: impl Future<Output = Result<T, HarnessError>>,
) -> Result<T, HarnessError> {
    tokio::time::timeout(DDL_TIMEOUT, future)
        .await
        .map_err(|_| {
            HarnessError::invalid(format!(
                "PostgreSQL CREATE or DROP DATABASE exceeded {} seconds",
                DDL_TIMEOUT.as_secs()
            ))
        })?
}

pub(super) async fn drop_database(admin_url: &str, name: &str) -> Result<(), HarnessError> {
    if names::created_at(name).is_none() {
        return Err(HarnessError::invalid(
            "refusing to drop a database not named by the harness",
        ));
    }
    let mut connection = connect_for_ddl(admin_url).await?;
    execute_ddl(
        &mut connection,
        format!("DROP DATABASE IF EXISTS {name} WITH (FORCE)"),
    )
    .await
}

#[cfg(test)]
mod creation_tests {
    use super::{TestDatabase, connect, connect_for_ddl, creation_reason, names};
    use crate::{HarnessError, postgres::admin_url};

    #[test]
    fn code_42501_gives_createdb_text() -> Result<(), Box<dyn std::error::Error>> {
        let text = creation_reason(Some("42501"), "permission denied for database \"test\"");
        assert_eq!(
            text,
            "CREATE DATABASE failed; INVENTORY_POSTGRES_URL must connect as a role with CREATEDB privilege"
        );
        Ok(())
    }

    #[test]
    fn other_sqlstate_includes_code_and_message() -> Result<(), Box<dyn std::error::Error>> {
        let text = creation_reason(Some("42P04"), "database \"x\" already exists");
        assert!(text.contains("42P04"));
        assert!(text.contains("database \"x\" already exists"));
        Ok(())
    }

    #[test]
    fn none_code_uses_message_only() -> Result<(), Box<dyn std::error::Error>> {
        let text = creation_reason(None, "connection timed out");
        assert!(text.contains("connection timed out"));
        assert!(!text.contains("CREATEDB"));
        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires INVENTORY_POSTGRES_URL"]
    async fn duplicate_create_does_not_remove_the_existing_database() -> Result<(), HarnessError> {
        let admin = admin_url()?;
        let name = names::generate()?;
        let original = TestDatabase::create_named(&admin, name.clone()).await?;
        let duplicate = TestDatabase::create_named(&admin, name).await;
        let Err(error) = duplicate else {
            return Err(HarnessError::invalid(
                "duplicate database creation unexpectedly succeeded",
            ));
        };
        assert!(
            error
                .to_string()
                .starts_with("CREATE DATABASE failed (SQLSTATE 42P04):")
        );
        let mut connection = connect(original.url()).await?;
        let value: i32 = sqlx::query_scalar("SELECT 42")
            .fetch_one(&mut connection)
            .await?;
        assert_eq!(value, 42);
        drop(connection);
        original.finish().await
    }

    #[tokio::test]
    #[ignore = "requires INVENTORY_POSTGRES_URL"]
    async fn ddl_connection_sets_a_60_second_statement_timeout() -> Result<(), HarnessError> {
        let admin = admin_url()?;
        let mut connection = connect_for_ddl(&admin).await?;
        let value: String = sqlx::query_scalar("SHOW statement_timeout")
            .fetch_one(&mut connection)
            .await?;
        assert_eq!(value, "1min");
        Ok(())
    }
}

#[cfg(test)]
mod ddl_tests;
