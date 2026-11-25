use crate::HarnessError;
use sqlx::{
    Connection, PgConnection, SqliteConnection, postgres::PgConnectOptions,
    sqlite::SqliteConnectOptions,
};
use std::{collections::BTreeSet, str::FromStr};

const MAX_IDENTITIES: usize = 100_000;
const SELECT_IDENTITIES: &str = "SELECT id FROM resources WHERE kind = $1 AND (region = $2 OR region = 'global') AND (account = $3 OR $3 IS NULL) LIMIT 100001";
const SELECT_ACCOUNTS: &str = "SELECT DISTINCT account FROM resources WHERE kind = $1 AND (region = $2 OR region = 'global') ORDER BY account LIMIT 100001";

pub(super) enum Reader {
    Postgres(PgConnection),
    Sqlite(SqliteConnection),
}

impl Reader {
    pub(super) async fn connect(database: &str) -> Result<Self, HarnessError> {
        if database.starts_with("postgres:") || database.starts_with("postgresql:") {
            let options = PgConnectOptions::from_str(database)?.options([
                ("default_transaction_read_only", "on"),
                ("search_path", "inventory,public"),
                ("statement_timeout", "30000"),
            ]);
            return Ok(Self::Postgres(PgConnection::connect_with(&options).await?));
        }
        if database.starts_with("sqlite:") {
            let options = SqliteConnectOptions::from_str(database)?
                .read_only(true)
                .create_if_missing(false);
            return Ok(Self::Sqlite(
                SqliteConnection::connect_with(&options).await?,
            ));
        }
        Err(HarnessError::invalid(
            "Cloud Control database must be a PostgreSQL or SQLite URL",
        ))
    }

    pub(super) async fn identities(
        &mut self,
        kind: &str,
        region: &str,
        account: Option<&str>,
    ) -> Result<BTreeSet<String>, HarnessError> {
        if account.is_none() {
            let accounts = self.accounts(kind, region).await?;
            if accounts.len() > 1 {
                return Err(HarnessError::invalid(format!(
                    "Cloud Control kind {kind} has multiple inventory accounts: {accounts:?}; specify --account"
                )));
            }
        }
        let rows: Vec<String> = match self {
            Self::Postgres(connection) => {
                sqlx::query_scalar(SELECT_IDENTITIES)
                    .bind(kind)
                    .bind(region)
                    .bind(account)
                    .fetch_all(connection)
                    .await?
            }
            Self::Sqlite(connection) => {
                sqlx::query_scalar(SELECT_IDENTITIES)
                    .bind(kind)
                    .bind(region)
                    .bind(account)
                    .fetch_all(connection)
                    .await?
            }
        };
        if rows.len() > MAX_IDENTITIES {
            return Err(HarnessError::invalid(
                "Cloud Control inventory identity bound exceeded",
            ));
        }
        Ok(rows.into_iter().collect())
    }

    async fn accounts(&mut self, kind: &str, region: &str) -> Result<Vec<String>, HarnessError> {
        Ok(match self {
            Self::Postgres(connection) => {
                sqlx::query_scalar(SELECT_ACCOUNTS)
                    .bind(kind)
                    .bind(region)
                    .fetch_all(connection)
                    .await?
            }
            Self::Sqlite(connection) => {
                sqlx::query_scalar(SELECT_ACCOUNTS)
                    .bind(kind)
                    .bind(region)
                    .fetch_all(connection)
                    .await?
            }
        })
    }

    pub(super) async fn close(self) -> Result<(), HarnessError> {
        match self {
            Self::Postgres(connection) => connection.close().await?,
            Self::Sqlite(connection) => connection.close().await?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod scoping_tests;
