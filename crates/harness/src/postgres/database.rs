use sqlx::postgres::{PgConnectOptions, PgConnection};
use sqlx::{ConnectOptions, Executor};

use super::{
    CleanupFailures, URL_LIMIT, bounded, creation, names,
    worker::{Actions, Worker},
};
use crate::HarnessError;
use std::future::Future;

const DDL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(65);

#[non_exhaustive]
pub struct TestDatabase {
    name: names::DatabaseName,
    url: String,
    owner: Worker,
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
        Self::create_tracked(admin_url, &CleanupFailures::default()).await
    }

    pub async fn create_tracked(
        admin_url: &str,
        failures: &CleanupFailures,
    ) -> Result<Self, HarnessError> {
        Self::create_named(admin_url, names::generate()?, failures).await
    }

    async fn create_named(
        admin_url: &str,
        name: names::DatabaseName,
        failures: &CleanupFailures,
    ) -> Result<Self, HarnessError> {
        let actions = creation::Postgres {
            admin: admin_url.into(),
            name: name.as_str().into(),
        };
        Self::create_owned(admin_url, name, failures, actions).await
    }

    async fn create_owned(
        admin_url: &str,
        name: names::DatabaseName,
        failures: &CleanupFailures,
        actions: impl Actions,
    ) -> Result<Self, HarnessError> {
        let url = database_url(admin_url, name.as_str())?;
        let (owner, ready) = Worker::start(name.as_str().into(), failures.clone(), actions)?;
        let database = Self { name, url, owner };
        bounded_ddl(async {
            ready
                .await
                .map_err(|_| HarnessError::invalid("database worker stopped during creation"))?
        })
        .await?;
        Ok(database)
    }

    #[cfg(test)]
    pub(crate) async fn create_with_actions(
        admin: &str,
        failures: &CleanupFailures,
        actions: impl Actions,
    ) -> Result<Self, HarnessError> {
        Self::create_owned(admin, names::generate()?, failures, actions).await
    }

    #[cfg(test)]
    pub(super) async fn create_at(admin_url: &str, seconds: u32) -> Result<Self, HarnessError> {
        Self::create_named(
            admin_url,
            names::DatabaseName::at(seconds),
            &CleanupFailures::default(),
        )
        .await
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn keep(self) {
        if let Err(error) = self.owner.keep() {
            eprintln!("PostgreSQL database retention failed: {error}");
        }
    }

    pub async fn finish(self) -> Result<(), HarnessError> {
        self.owner.finish().await
    }
}

#[cfg(test)]
fn creation_reason(code: Option<&str>, message: &str) -> String {
    match code {
        Some("42501") => {
            "CREATE DATABASE failed; INVENTORY_POSTGRES_URL must connect as a role with CREATEDB privilege".into()
        }
        Some(other) => format!("CREATE DATABASE failed (SQLSTATE {other}): {message}"),
        None => format!("CREATE DATABASE failed: {message}"),
    }
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
    if names::created_at(name).is_none() {
        return Err(HarnessError::invalid(
            "refusing to address a database not named by the harness",
        ));
    }
    options(url)?;
    let retargeted = retarget(url, name)?;
    if options(&retargeted)?.get_database() != Some(name) {
        return Err(HarnessError::invalid(
            "INVENTORY_POSTGRES_URL cannot be retargeted to the fixture database",
        ));
    }
    Ok(retargeted)
}

fn retarget(admin: &str, name: &str) -> Result<String, HarnessError> {
    let mut parsed = url::Url::parse(admin)
        .map_err(|_| HarnessError::invalid("invalid INVENTORY_POSTGRES_URL"))?;
    parsed.set_path(name);
    if parsed.query_pairs().any(|(key, _)| key == "dbname") {
        let kept: Vec<(String, String)> = parsed
            .query_pairs()
            .filter(|(key, _)| key != "dbname")
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect();
        if kept.is_empty() {
            parsed.set_query(None);
        } else {
            parsed.query_pairs_mut().clear().extend_pairs(kept);
        }
    }
    Ok(parsed.into())
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

pub(super) async fn bounded_ddl<T>(
    future: impl Future<Output = Result<T, HarnessError>>,
) -> Result<T, HarnessError> {
    ddl_bounded(future).await
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
    use super::{CleanupFailures, TestDatabase, connect, connect_for_ddl, creation_reason, names};
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
        let original =
            TestDatabase::create_named(&admin, name.clone(), &CleanupFailures::default()).await?;
        let duplicate = TestDatabase::create_named(&admin, name, &CleanupFailures::default()).await;
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

#[cfg(test)]
mod tests;
