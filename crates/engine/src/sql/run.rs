use super::{Bounds, Budget, ReaderUrl, SqlError, SqlValue, reader::Backend, validate, value};
use futures_util::{StreamExt, TryStreamExt};
use sqlx::{
    Column, Connection, Executor, PgConnection, Postgres, Row, SqlSafeStr, Statement, Transaction,
    postgres::PgRow,
};
use std::{sync::Arc, time::Duration};

#[derive(Debug, PartialEq)]
pub struct SqlResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<SqlValue>>,
}

#[tracing::instrument(skip_all)]
pub async fn run(reader: &ReaderUrl, sql: &str, bounds: Bounds) -> Result<SqlResult, SqlError> {
    validate(sql)?;
    bounds.validate()?;
    let (mut connection, backend) = reader.connect().await?;
    let mut guard = TerminateOnDrop {
        reader,
        runtime: tokio::runtime::Handle::current(),
        backend,
        armed: true,
    };
    let server_millis = bounds.millis;
    #[cfg(test)]
    let server_millis = reader.server_millis.unwrap_or(server_millis);
    let result = execute(
        &mut connection,
        sql,
        bounds,
        server_millis,
        reader.namespace.schema(),
    )
    .await;
    if matches!(result, Err(SqlError::Timeout)) {
        reader.terminate(&backend).await?;
        guard.armed = false;
        return result;
    }
    guard.armed = false;
    let result = result?;
    super::reader::close(connection).await?;
    Ok(result)
}

struct TerminateOnDrop<'a> {
    reader: &'a ReaderUrl,
    runtime: tokio::runtime::Handle,
    backend: Backend,
    armed: bool,
}

impl Drop for TerminateOnDrop<'_> {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let reader = self.reader.clone();
        let backend = self.backend;
        self.runtime.spawn(async move {
            if let Err(error) = reader.terminate(&backend).await {
                tracing::warn!(pid = backend.pid, %error, "failed to terminate abandoned reader backend");
            }
        });
    }
}

#[tracing::instrument(skip_all)]
async fn execute(
    connection: &mut PgConnection,
    sql: &str,
    bounds: Bounds,
    server_millis: u64,
    schema: &str,
) -> Result<SqlResult, SqlError> {
    let mut tx = tokio::time::timeout(
        Duration::from_secs(1),
        connection.begin_with("BEGIN TRANSACTION READ ONLY"),
    )
    .await
    .map_err(|_| SqlError::Timeout)??;
    let result = tokio::time::timeout(
        Duration::from_millis(bounds.millis.saturating_add(1_000)),
        query_transaction(&mut tx, sql, bounds, server_millis, schema),
    )
    .await
    .map_err(|_| SqlError::Timeout)??;
    tokio::time::timeout(Duration::from_secs(1), tx.rollback())
        .await
        .map_err(|_| SqlError::Timeout)??;
    Ok(result)
}

#[tracing::instrument(skip_all)]
async fn query_transaction(
    tx: &mut Transaction<'_, Postgres>,
    sql: &str,
    bounds: Bounds,
    server_millis: u64,
    schema: &str,
) -> Result<SqlResult, SqlError> {
    sqlx::query("SELECT set_config('statement_timeout', $1, true)")
        .bind(format!("{server_millis}ms"))
        .execute(&mut **tx)
        .await?;
    sqlx::query("SELECT set_config('search_path', $1, true)")
        .bind(schema)
        .execute(&mut **tx)
        .await?;
    fetch(tx, sql, bounds).await
}

#[tracing::instrument(skip_all)]
async fn fetch(
    connection: &mut PgConnection,
    sql: &str,
    bounds: Bounds,
) -> Result<SqlResult, SqlError> {
    let sql: Arc<str> = Arc::from(sql);
    let statement = connection
        .prepare(sqlx::AssertSqlSafe(Arc::clone(&sql)).into_sql_str())
        .await
        .map_err(query_error)?;
    let mut budget = Budget::new(bounds)?;
    budget.charge_columns(statement.columns().iter().map(Column::name))?;
    let mut columns = Vec::new();
    columns
        .try_reserve_exact(statement.columns().len())
        .map_err(|_| SqlError::Allocation)?;
    columns.extend(
        statement
            .columns()
            .iter()
            .map(|column| column.name().to_owned()),
    );
    let mut rows = Vec::new();
    rows.try_reserve_exact(bounds.rows)
        .map_err(|_| SqlError::Allocation)?;
    let output = SqlResult { columns, rows };
    sqlx::query(sqlx::AssertSqlSafe(sql))
        .fetch(connection)
        .map_err(query_error)
        .take(bounds.rows.saturating_add(1))
        .try_fold(
            (output, budget),
            |(mut output, mut budget), row| async move {
                tokio::task::coop::consume_budget().await;
                output.rows.push(decode_row(&row, &mut budget)?);
                Ok((output, budget))
            },
        )
        .await
        .map(|(output, _)| output)
}

fn decode_row(row: &PgRow, budget: &mut Budget) -> Result<Vec<SqlValue>, SqlError> {
    budget.charge_row(row.columns().len())?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(row.columns().len())
        .map_err(|_| SqlError::Allocation)?;
    row.columns()
        .iter()
        .enumerate()
        .try_for_each(|(index, _)| {
            values.push(value::decode(row, index, budget)?);
            Ok::<_, SqlError>(())
        })?;
    Ok(values)
}

fn query_error(error: sqlx::Error) -> SqlError {
    match &error {
        sqlx::Error::Database(database) if database.code().as_deref() == Some("57014") => {
            SqlError::StatementTimeout
        }
        _ => SqlError::Database(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::test_peer::{Peer, Stall};

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[tokio::test]
    async fn given_peer_stalling_rollback_when_executing_then_step_timeout_is_returned()
    -> TestResult {
        let mut peer = Peer::start(Stall::Rollback).await?;
        let result = tokio::time::timeout(Duration::from_secs(30), async {
            let mut connection = PgConnection::connect_with(&peer.options).await?;
            let result = execute(
                &mut connection,
                "SELECT 1 AS n",
                Bounds::default(),
                Bounds::default().millis,
                "inventory",
            )
            .await;
            (&mut peer.stalled).await?;
            Ok::<_, Box<dyn std::error::Error>>(result)
        })
        .await;
        let cleanup = peer.finish().await;
        let error = result??.err().ok_or("expected an error")?;
        cleanup?;
        assert_eq!(
            error.to_string(),
            "SQL connection or transaction step exceeded time bound"
        );
        assert!(matches!(error, SqlError::Timeout));
        Ok(())
    }
}

#[cfg(test)]
#[path = "run/watchdog_tests.rs"]
mod watchdog_tests;
