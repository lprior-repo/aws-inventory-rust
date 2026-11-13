use std::collections::BTreeSet;

use serde_json::Value;

use super::catalog;

#[test]
fn catalog_kinds_and_derived_views_match_the_sqlite_schema()
-> Result<(), Box<dyn std::error::Error>> {
    let catalog = catalog();
    let tables = catalog
        .get("tables")
        .and_then(Value::as_array)
        .ok_or("missing native table catalog")?;
    assert_eq!(tables.len(), 32);
    assert_eq!(
        catalog
            .get("native_resource_kind_count")
            .and_then(Value::as_u64),
        Some(32)
    );
    assert_eq!(
        catalog.get("schema_version").and_then(Value::as_u64),
        Some(6)
    );
    assert_eq!(
        catalog.get("sqlite_schema_version").and_then(Value::as_u64),
        Some(6)
    );
    let migrations = [
        include_str!("../../crates/engine/sql/001_inventory.sql"),
        include_str!("../../crates/engine/sql/002_architecture.sql"),
        include_str!("../../crates/engine/sql/003_sources.sql"),
        include_str!("../../crates/engine/sql/004_global_inventory.sql"),
    ];
    let mut native_names = BTreeSet::new();
    for table in tables {
        let name = table
            .get("table")
            .and_then(Value::as_str)
            .ok_or("missing native table name")?;
        assert!(native_names.insert(name), "duplicate native kind: {name}");
        let declaration = format!("CREATE VIEW {name} AS");
        assert!(
            migrations.iter().any(|sql| sql.contains(&declaration)),
            "catalog table has no SQLite view: {name}"
        );
        let operations = table
            .get("operations")
            .and_then(Value::as_array)
            .ok_or("missing operation list")?;
        assert!(!operations.is_empty());
        assert!(
            operations
                .iter()
                .all(|operation| operation.as_str().is_some_and(|value| !value.is_empty()))
        );
    }
    let derived = catalog
        .get("derived_sql_views")
        .and_then(Value::as_array)
        .ok_or("missing derived view catalog")?;
    let mut derived_names = BTreeSet::new();
    for view in derived {
        let name = view
            .get("view")
            .and_then(Value::as_str)
            .ok_or("missing derived view name")?;
        assert!(derived_names.insert(name), "duplicate derived view: {name}");
        assert!(
            !native_names.contains(name),
            "derived view counted as a native kind: {name}"
        );
        let declaration = format!("CREATE VIEW {name} AS");
        assert!(
            migrations.iter().any(|sql| sql.contains(&declaration)),
            "derived view has no SQLite definition: {name}"
        );
    }
    Ok(())
}
