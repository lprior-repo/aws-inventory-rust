use super::*;

pub(super) async fn verify_sqlite_limits(pool: &SqlitePool) -> Result<(), StoreError> {
    let row = sqlx::query(
        "SELECT sqlite_compileoption_used('MAX_LENGTH=8388608') AS length_ok,
        sqlite_compileoption_used('MAX_SQL_LENGTH=65536') AS sql_ok,
        sqlite_compileoption_used('MAX_COLUMN=256') AS column_ok,
        sqlite_compileoption_used('MAX_EXPR_DEPTH=128') AS depth_ok",
    )
    .fetch_one(pool)
    .await?;
    for column in ["length_ok", "sql_ok", "column_ok", "depth_ok"] {
        if !row.try_get::<bool, _>(column)? {
            return Err(StoreError::InvalidInput(
                "SQLite compile bounds missing; build with repository .cargo/config.toml",
            ));
        }
    }
    Ok(())
}
