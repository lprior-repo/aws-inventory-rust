mod table;

use crate::command_output::CommandOutput;
use futures_util::future::BoxFuture;
use inventory_engine::sql::{Bounds, ReaderUrl, validate};

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
    let reader = ReaderUrl::establish(&database).await?;
    let result = inventory_engine::sql::run(&reader, &statement, bounds).await?;
    table::render(&result, bounds.bytes)
}
