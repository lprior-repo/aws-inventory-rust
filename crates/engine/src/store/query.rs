use super::*;

pub(super) async fn install_query_progress(
    connection: &mut SqliteConnection,
) -> Result<(), StoreError> {
    let started = Instant::now();
    let mut remaining = QUERY_PROGRESS_CALLS;
    connection
        .lock_handle()
        .await?
        .set_progress_handler(1_000, move || {
            remaining = remaining.saturating_sub(1);
            remaining != 0 && started.elapsed() <= QUERY_TIMEOUT
        });
    Ok(())
}

pub(super) async fn reset_progress(connection: &mut SqliteConnection) -> Result<(), StoreError> {
    connection.lock_handle().await?.remove_progress_handler();
    Ok(())
}

pub(super) struct QueryOutput {
    pub(super) rows: Vec<Value>,
    pub(super) columns: Option<Vec<String>>,
    pub(super) budget: ByteBudget,
    pub(super) cells: usize,
}

impl QueryOutput {
    pub(super) fn new(row_limit: usize) -> Result<Self, StoreError> {
        let mut rows = Vec::new();
        rows.try_reserve(row_limit)?;
        let mut budget = ByteBudget::new(MAX_QUERY_BYTES);
        budget.add(2)?;
        Ok(Self {
            rows,
            columns: None,
            budget,
            cells: 0,
        })
    }
    pub(super) fn push(mut self, row: SqliteRow, row_limit: usize) -> Result<Self, StoreError> {
        if self.rows.len() >= row_limit {
            return Err(StoreError::Limit(
                "query rows; narrow the query or use streaming export",
            ));
        }
        if self.columns.is_none() {
            self.columns = Some(query_columns(&row)?);
        }
        let columns = self
            .columns
            .as_ref()
            .ok_or(StoreError::Conflict("query columns unavailable"))?;
        self.cells = self
            .cells
            .checked_add(columns.len())
            .ok_or(StoreError::Limit("query cells"))?;
        if self.cells > MAX_QUERY_CELLS {
            return Err(StoreError::Limit("query cells"));
        }
        self.budget.add(3)?;
        self.rows.push(query_row(&row, columns, &mut self.budget)?);
        Ok(self)
    }
}

pub(super) async fn execute_query(
    connection: &mut SqliteConnection,
    sql: &str,
    row_limit: usize,
) -> Result<Value, StoreError> {
    let query = sqlx::query(sqlx::AssertSqlSafe(sql));
    let result = query
        .fetch(connection)
        .map_err(query_error)
        .take(row_limit.saturating_add(1))
        .try_fold(QueryOutput::new(row_limit)?, |output, row| async move {
            tokio::task::coop::consume_budget().await;
            output.push(row, row_limit)
        })
        .await?;
    Ok(Value::Array(result.rows))
}

pub(super) fn query_error(error: sqlx::Error) -> StoreError {
    match &error {
        sqlx::Error::Database(database) if database.code().as_deref() == Some("18") => {
            StoreError::Limit("SQLite value bytes")
        }
        _ => StoreError::Sqlx(error),
    }
}

pub(super) fn query_columns(row: &SqliteRow) -> Result<Vec<String>, StoreError> {
    if row.columns().len() > MAX_QUERY_COLUMNS {
        return Err(StoreError::Limit("query columns"));
    }
    let mut columns = Vec::new();
    let mut distinct = BTreeSet::new();
    columns.try_reserve(row.columns().len())?;
    for column in row.columns() {
        let name = column.name();
        if !distinct.insert(name) {
            return Err(StoreError::QueryDenied(
                "duplicate column names; assign distinct aliases",
            ));
        }
        columns.push(name.to_owned());
    }
    Ok(columns)
}

pub(super) fn query_row(
    row: &SqliteRow,
    columns: &[String],
    budget: &mut ByteBudget,
) -> Result<Value, StoreError> {
    let mut object = Map::new();
    for (index, name) in columns.iter().enumerate() {
        budget.charge(name)?;
        budget.add(2)?;
        let cell = sql_value(row, index, budget)?;
        object.insert(name.clone(), cell);
    }
    Ok(Value::Object(object))
}

pub(super) fn sql_value(
    row: &SqliteRow,
    index: usize,
    budget: &mut ByteBudget,
) -> Result<Value, StoreError> {
    let raw = row.try_get_raw(index)?;
    let value = if raw.is_null() {
        Value::Null
    } else {
        match raw.type_info().name() {
            "INTEGER" | "BOOLEAN" => Value::Number(row.try_get::<i64, _>(index)?.into()),
            "REAL" => Value::Number(
                Number::from_f64(row.try_get(index)?)
                    .ok_or(StoreError::QueryDenied("non-finite numeric result"))?,
            ),
            "TEXT" | "DATE" | "TIME" | "DATETIME" => {
                let text: &str = row.try_get(index)?;
                if text.len() > budget.limit.saturating_sub(budget.used) {
                    return Err(StoreError::Limit("query bytes"));
                }
                Value::String(text.to_owned())
            }
            "BLOB" => {
                return Err(StoreError::QueryDenied(
                    "BLOB output requires an explicit hex() projection",
                ));
            }
            _ => return Err(StoreError::QueryDenied("unsupported SQLite result type")),
        }
    };
    budget.charge(&value)?;
    Ok(value)
}
