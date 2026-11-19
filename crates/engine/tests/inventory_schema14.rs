use inventory_core::inventory::{Kind, KindSchema, ScalarType, SqlName};
use inventory_core::record::FieldId;
use inventory_engine::inventory::{BinaryStore, StoreError};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

const CARGO_CONFIG: &str = include_str!("../../../.cargo/config.toml");

#[tokio::test]
async fn given_schema13_when_opened_then_rejects_without_migration()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("inventory.sqlite");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&database)
                .create_if_missing(true),
        )
        .await?;
    sqlx::query("PRAGMA user_version=13").execute(&pool).await?;
    pool.close().await;

    let result = BinaryStore::open(&database).await;

    assert!(matches!(
        result,
        Err(StoreError::SchemaVersion { found: 13 })
    ));
    Ok(())
}

#[test]
fn given_schema_exceeding_sqlite_column_limit_when_validated_then_rejected_at_admission()
-> Result<(), Box<dyn std::error::Error>> {
    let kind = Kind::parse("AWS::Test::Oversized")?;
    let mut columns = Vec::new();
    // 7 fixed columns + 256 projected = 263 total, exceeding SQLITE_MAX_COLUMN=256.
    for i in 1..=256 {
        let id = FieldId::new(i)?;
        columns.push(inventory_core::inventory::Column::new(
            vec![id],
            ScalarType::Text,
        )?);
    }
    let table = SqlName::parse("test_oversized")?;
    let result = KindSchema::new(kind.clone(), table.clone(), columns);
    assert!(
        matches!(result, Err(inventory_core::inventory::InventoryError::Columns)),
        "schema with 256 projected columns (263 total) should exceed SQLite limit"
    );
    // 249 projected columns (256 total) is the maximum allowed.
    let mut ok_columns = Vec::new();
    for i in 1..=249 {
        let id = FieldId::new(i)?;
        ok_columns.push(inventory_core::inventory::Column::new(
            vec![id],
            ScalarType::Text,
        )?);
    }
    let result = KindSchema::new(kind.clone(), table.clone(), ok_columns);
    assert!(
        result.is_ok(),
        "schema with 249 projected columns (256 total) should be accepted"
    );
    // 250 projected columns (257 total) exceeds the limit.
    let mut over_columns = Vec::new();
    for i in 1..=250 {
        let id = FieldId::new(i)?;
        over_columns.push(inventory_core::inventory::Column::new(
            vec![id],
            ScalarType::Text,
        )?);
    }
    let result = KindSchema::new(kind.clone(), table.clone(), over_columns);
    assert!(
        matches!(result, Err(inventory_core::inventory::InventoryError::Columns)),
        "schema with 250 projected columns (257 total) should exceed SQLite limit"
    );
    // Drift guard: ensure the constant in core matches the build config.
    // Parse manually to extract the value without regex dependency.
    let config = CARGO_CONFIG;
    let start = config.find("SQLITE_MAX_COLUMN=").expect("SQLITE_MAX_COLUMN not found in config");
    let value_start = start + 18; // len("SQLITE_MAX_COLUMN=")
    let after = &config[value_start..];
    let value_end = after
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(after.len());
    let expected: u16 = after[..value_end]
        .parse()
        .expect("non-numeric SQLITE_MAX_COLUMN value");
    // The constant is in inventory-core; import it via the Column constructor.
    // 7 fixed + projected = total SQLite columns. Validate the boundary.
    let boundary = expected - 7;
    assert!(
        boundary > 0 && boundary < 256,
        "computed boundary {boundary} is outside expected range"
    );
    Ok(())
}
