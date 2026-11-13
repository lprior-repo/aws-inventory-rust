//! Transactional schema initialization and managed-view migration.
use super::*;

pub(super) async fn initialize_schema(pool: &SqlitePool) -> Result<(), StoreError> {
    let application: i64 = sqlx::query_scalar("PRAGMA application_id")
        .fetch_one(pool)
        .await?;
    if application != 0 {
        return Err(StoreError::InvalidInput(
            "database belongs to a different application",
        ));
    }
    verify_sqlite_limits(pool).await?;
    // This pool is private to Store::open until migration succeeds. On any
    // failure it is closed, so a connection with temporarily disabled foreign
    // keys cannot escape. Keep it alive on success for shared in-memory stores.
    let mut connection = pool.acquire().await?;
    sqlx::query("PRAGMA foreign_keys=OFF")
        .execute(&mut *connection)
        .await?;
    let mut tx = connection.begin_with("BEGIN IMMEDIATE").await?;
    let version: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(&mut *tx)
        .await?;
    if !(0..=13).contains(&version) {
        return Err(StoreError::SchemaVersion(version));
    }
    if version == 0 {
        apply(&mut tx, include_str!("../../sql/001_inventory.sql")).await?;
    }
    if version < 2 {
        apply(&mut tx, include_str!("../../sql/002_architecture.sql")).await?;
    }
    if version < 3 {
        migrate_sources(&mut tx).await?;
    }
    if version < 4 {
        apply(&mut tx, include_str!("../../sql/004_global_inventory.sql")).await?;
    }
    if version < 5 {
        migrate_cloud_control_views(
            &mut tx,
            include_str!("../../sql/005_cloudcontrol_expansion.sql"),
        )
        .await?;
    }
    if version < 6 {
        migrate_cloud_control_views(
            &mut tx,
            include_str!("../../sql/006_cloudcontrol_parents.sql"),
        )
        .await?;
    }
    if version < 7 {
        apply(&mut tx, include_str!("../../sql/007_freshness.sql")).await?;
    }
    if version < 8 {
        apply(&mut tx, include_str!("../../sql/008_coverage_registry.sql")).await?;
    }
    if version < 9 {
        apply(&mut tx, include_str!("../../sql/009_native_sqs_view.sql")).await?;
    }
    if version < 10 {
        apply(&mut tx, include_str!("../../sql/010_native_sns_views.sql")).await?;
    }
    if version < 11 {
        apply(&mut tx, include_str!("../../sql/011_native_kms_views.sql")).await?;
    }
    if version < 12 {
        apply(
            &mut tx,
            include_str!("../../sql/012_native_step_functions_views.sql"),
        )
        .await?;
    }
    if version < 13 {
        migrate_cloud_control_views(
            &mut tx,
            include_str!("../../sql/013_cloudcontrol_parent_additions.sql"),
        )
        .await?;
    }
    tx.commit().await?;
    sqlx::query("PRAGMA foreign_keys=ON")
        .execute(&mut *connection)
        .await?;
    let foreign_keys: bool = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&mut *connection)
        .await?;
    if !foreign_keys {
        return Err(StoreError::InvalidInput("foreign keys were not restored"));
    }
    Ok(())
}

async fn apply(
    tx: &mut Transaction<'_, Sqlite>,
    migration: &'static str,
) -> Result<(), StoreError> {
    sqlx::raw_sql(migration).execute(&mut **tx).await?;
    Ok(())
}

/// Add reviewed source views without rebuilding observations or replacing user
/// schema objects. A name conflict rolls back the entire migration.
#[tracing::instrument(skip_all)]
pub(super) async fn migrate_cloud_control_views(
    tx: &mut Transaction<'_, Sqlite>,
    migration: &'static str,
) -> Result<(), StoreError> {
    let statements: Vec<&'static str> = migration
        .split_terminator(';')
        .map(str::trim)
        .filter(|sql| !sql.is_empty())
        .take(514)
        .collect();
    if statements.is_empty()
        || statements.len() > 513
        || statements.iter().any(|sql| sql.len() > MAX_SQL_BYTES)
    {
        return Err(StoreError::InvalidInput(
            "invalid expanded Cloud Control views",
        ));
    }
    for statement in statements {
        sqlx::raw_sql(statement).execute(&mut **tx).await?;
    }
    Ok(())
}

/// SQLite's generalized table-rebuild migration, preserving user views,
/// indexes, and triggers. No sqlite_schema edits or legacy ALTER semantics.
pub(super) async fn migrate_sources(tx: &mut Transaction<'_, Sqlite>) -> Result<(), StoreError> {
    verify_managed_columns(tx).await?;
    let sizes = sqlx::query(
        "SELECT count(*) AS objects, coalesce(sum(length(CAST(sql AS BLOB))),0) AS bytes,
        coalesce(max(length(CAST(sql AS BLOB))),0) AS largest
        FROM sqlite_schema WHERE sql IS NOT NULL AND (
        type IN ('view','trigger') OR (type='index' AND tbl_name IN (
            'scope_checkpoints','resource_observations','edge_observations')))",
    )
    .fetch_one(&mut **tx)
    .await?;
    if sizes.try_get::<i64, _>("objects")? > 1_024
        || sizes.try_get::<i64, _>("bytes")? > 4_194_304
        || sizes.try_get::<i64, _>("largest")? > 65_536
    {
        return Err(StoreError::Limit("migration schema objects"));
    }
    let objects = sqlx::query(
        "SELECT type, name, sql FROM sqlite_schema WHERE sql IS NOT NULL AND (
        type IN ('view','trigger') OR (type='index' AND tbl_name IN (
            'scope_checkpoints','resource_observations','edge_observations')))
        ORDER BY type DESC, name LIMIT 1024",
    )
    .fetch_all(&mut **tx)
    .await?;
    // Dropping a view also drops its INSTEAD OF triggers, so remove triggers
    // first. Restoration uses the opposite order from the schema query above.
    for (kind, keyword) in [("trigger", "TRIGGER"), ("view", "VIEW")] {
        for object in &objects {
            if object.try_get::<&str, _>("type")? != kind {
                continue;
            }
            let name: &str = object.try_get("name")?;
            // DDL identifiers cannot be bound. Only the fixed keywords above
            // and a double-quote-escaped schema identifier enter this SQL.
            let quoted = name.replace('"', "\"\"");
            let drop_sql = format!("DROP {keyword} \"{quoted}\"");
            sqlx::query(sqlx::AssertSqlSafe(drop_sql.as_str()))
                .execute(&mut **tx)
                .await?;
        }
    }
    sqlx::raw_sql(include_str!("../../sql/003_sources.sql"))
        .execute(&mut **tx)
        .await?;
    for object in &objects {
        let kind: &str = object.try_get("type")?;
        let name: &str = object.try_get("name")?;
        if kind == "view" && name == "aws_ecs_service" {
            continue; // Migration 3 installs the expanded managed view.
        }
        let definition: &str = object.try_get("sql")?;
        // Replay only already-persisted, bounded CREATE definitions selected
        // from SQLite's schema. No CLI expression or bound value becomes DDL.
        sqlx::raw_sql(sqlx::AssertSqlSafe(definition))
            .execute(&mut **tx)
            .await?;
    }
    // SQLite checks the supplied SQL buffer against MAX_SQL_LENGTH, even when
    // it contains many small statements. This generator permits semicolons
    // only as statement terminators (names and JSON paths are restricted).
    let views: Vec<&'static str> = include_str!("../../sql/003_cloudcontrol_views.sql")
        .split_terminator(';')
        .map(str::trim)
        .filter(|statement| !statement.is_empty())
        .take(513)
        .collect();
    if views.len() != 200 || views.iter().any(|sql| sql.len() > MAX_SQL_BYTES) {
        return Err(StoreError::InvalidInput(
            "invalid generated Cloud Control views",
        ));
    }
    for view in views {
        sqlx::raw_sql(view).execute(&mut **tx).await?;
    }
    let broken: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pragma_foreign_key_check)")
        .fetch_one(&mut **tx)
        .await?;
    if broken {
        return Err(StoreError::Conflict("migration foreign key check failed"));
    }
    Ok(())
}

pub(super) async fn verify_managed_columns(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), StoreError> {
    // These tables are application-owned. Rebuilding a table with an unknown
    // extra column would silently discard its values, even if no view/index
    // refers to it. Reject extensions before any schema object is dropped.
    let expected: [(&str, &[&str]); 3] = [
        (
            "scope_checkpoints",
            &[
                "scan_id",
                "scope_key",
                "account_id",
                "region",
                "collector",
                "parent_id",
                "source",
                "last_page",
                "next_token",
                "complete",
                "failed",
            ],
        ),
        (
            "resource_observations",
            &[
                "scan_id",
                "scope_key",
                "page_number",
                "resource_id",
                "account_id",
                "region",
                "kind",
                "name",
                "source",
                "source_priority",
                "observed_at",
                "resource_json",
            ],
        ),
        (
            "edge_observations",
            &[
                "scan_id",
                "scope_key",
                "page_number",
                "from_id",
                "to_id",
                "relation",
                "source",
                "source_priority",
                "observed_at",
                "edge_json",
            ],
        ),
    ];
    for (table, names) in expected {
        let actual: Vec<String> =
            sqlx::query_scalar("SELECT name FROM pragma_table_xinfo(?) ORDER BY cid LIMIT 257")
                .bind(table)
                .fetch_all(&mut **tx)
                .await?;
        if actual.iter().map(String::as_str).ne(names.iter().copied()) {
            return Err(StoreError::InvalidInput(
                "managed table columns differ from the supported migration schema",
            ));
        }
    }
    Ok(())
}

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
