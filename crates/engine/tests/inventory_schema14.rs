use inventory_core::inventory::{Kind, KindSchema, MAX_SQLITE_COLUMNS, ScalarType, SqlName};
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
        matches!(
            result,
            Err(inventory_core::inventory::InventoryError::Columns)
        ),
        "schema with 256 projected columns (263 total) should exceed SQLite limit"
    );
    let mut ok_columns = Vec::new();
    for i in 1..=249 {
        let id = FieldId::new(i)?;
        ok_columns.push(inventory_core::inventory::Column::new(
            vec![id],
            ScalarType::Text,
        )?);
    }
    let result = KindSchema::new(kind.clone(), table.clone(), ok_columns);
    result?;
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
        matches!(
            result,
            Err(inventory_core::inventory::InventoryError::Columns)
        ),
        "schema with 250 projected columns (257 total) should exceed SQLite limit"
    );
    let config = CARGO_CONFIG;
    let config_value = config
        .split("SQLITE_MAX_COLUMN=")
        .nth(1)
        .ok_or("SQLITE_MAX_COLUMN not found in config")?;
    let digits: String = config_value
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    let expected: u16 = digits
        .parse()
        .map_err(|_| "non-numeric SQLITE_MAX_COLUMN value")?;
    assert_eq!(
        usize::from(expected),
        MAX_SQLITE_COLUMNS,
        "MAX_SQLITE_COLUMNS ({}) must equal SQLITE_MAX_COLUMN build config ({})",
        MAX_SQLITE_COLUMNS,
        expected
    );
    Ok(())
}
