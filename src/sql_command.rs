mod table;

use crate::command_output::CommandOutput;
use futures_util::future::BoxFuture;
use inventory_engine::sql::{Bounds, ReaderUrl, SqlError, validate};

pub(super) fn run_sql(
    options: crate::cli::SqlArgs,
) -> BoxFuture<'static, anyhow::Result<CommandOutput>> {
    Box::pin(async move {
        execute(options.database, options.sql, options.limit)
            .await
            .map(CommandOutput::Text)
    })
}

#[tracing::instrument(skip_all)]
pub(super) async fn execute(
    database: String,
    statement: String,
    limit: usize,
) -> anyhow::Result<String> {
    let bounds = Bounds {
        rows: limit,
        ..Bounds::default()
    };
    validate(&statement)?;
    bounds.validate()?;
    let reader = ReaderUrl::establish(&database)
        .await
        .map_err(redact_database_error)?;
    let result = inventory_engine::sql::run(&reader, &statement, bounds)
        .await
        .map_err(redact_database_error)?;
    table::render(&result, bounds.bytes)
}
fn redact_database_error(error: SqlError) -> anyhow::Error {
    match error {
        SqlError::Database(_) => anyhow::anyhow!("PostgreSQL operation failed"),
        other => other.into(),
    }
}
