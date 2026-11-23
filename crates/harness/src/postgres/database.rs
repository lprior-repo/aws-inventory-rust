use sqlx::postgres::{PgConnectOptions, PgConnection};
use sqlx::{ConnectOptions, Connection, Executor};

use super::{URL_LIMIT, bounded, names};
use crate::HarnessError;

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
        let database = Self {
            url: database_url(admin_url, name.as_str())?,
            name,
            admin_url: admin_url.to_owned(),
            cleanup: true,
        };
        let result = bounded(execute(
            admin_url,
            format!("CREATE DATABASE {}", database.name()),
        ))
        .await;
        database.complete_creation(result)
    }

    fn complete_creation(mut self, result: Result<(), HarnessError>) -> Result<Self, HarnessError> {
        match result {
            Ok(()) => Ok(self),
            Err(error) => {
                if matches!(&error, HarnessError::Sqlx(sqlx::Error::Database(error)) if error.code().as_deref() == Some("42P04"))
                {
                    self.cleanup = false;
                }
                Err(HarnessError::invalid(
                    "CREATE DATABASE failed; INVENTORY_POSTGRES_URL must connect as a role with CREATEDB privilege",
                ))
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

async fn execute(admin_url: &str, statement: String) -> Result<(), HarnessError> {
    let mut connection = connect(admin_url).await?;
    connection.execute("SET statement_timeout = '10s'").await?;
    sqlx::raw_sql(sqlx::AssertSqlSafe(statement))
        .execute(&mut connection)
        .await?;
    connection.close().await?;
    Ok(())
}

pub(super) async fn drop_database(admin_url: &str, name: &str) -> Result<(), HarnessError> {
    if names::created_at(name).is_none() {
        return Err(HarnessError::invalid(
            "refusing to drop a database not named by the harness",
        ));
    }
    bounded(execute(
        admin_url,
        format!("DROP DATABASE IF EXISTS {name} WITH (FORCE)"),
    ))
    .await
}
