use inventory_core::inventory::{
    Column, InventoryError, MAX_DERIVED_SQL_NAME_BYTES, MAX_SQL_NAME_BYTES, ScalarType, SqlName,
    shorten_sql_name,
};
use inventory_core::record::FieldId;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_bounded_name_when_shortened_then_owned_buffer_is_preserved() -> TestResult {
    let full = format!("a_{}", "b".repeat(61));
    let pointer = full.as_ptr();
    let shortened = shorten_sql_name(full)?;
    assert_eq!(shortened.len(), MAX_SQL_NAME_BYTES);
    assert_eq!(shortened.as_ptr(), pointer);
    assert_eq!(SqlName::parse(&shortened)?.as_str(), shortened);
    Ok(())
}

#[test]
fn given_64_and_128_byte_names_when_shortened_then_full_name_hash_is_used() -> TestResult {
    for full in [
        format!("a_{}", "b".repeat(62)),
        format!("a_{}", "b".repeat(126)),
    ] {
        let hash = blake3::hash(full.as_bytes()).to_hex();
        let expected = format!(
            "{}_{}",
            full.get(..54).ok_or("prefix absent")?,
            hash.get(..8).ok_or("hash absent")?
        );
        let pointer = full.as_ptr();
        let shortened = shorten_sql_name(full)?;
        assert_eq!(shortened, expected);
        assert_eq!(shortened.len(), MAX_SQL_NAME_BYTES);
        assert_eq!(shortened.as_ptr(), pointer);
        SqlName::parse(&shortened)?;
    }
    Ok(())
}

#[test]
fn given_shared_prefix_when_long_names_differ_then_short_names_are_distinct() -> TestResult {
    let prefix = format!("a_{}", "b".repeat(52));
    let first = shorten_sql_name(format!("{prefix}cccccccccc"))?;
    let second = shorten_sql_name(format!("{prefix}dddddddddd"))?;
    assert_eq!(first.get(..54), second.get(..54));
    assert_ne!(first, second);
    let boundary = shorten_sql_name(format!("a_{}_{}", "b".repeat(51), "c".repeat(10)))?;
    assert!(boundary.contains("__"));
    SqlName::parse(&boundary)?;
    Ok(())
}

#[test]
fn given_unbounded_or_non_ascii_name_when_shortened_then_it_is_rejected() {
    assert!(matches!(
        shorten_sql_name("a".repeat(129)),
        Err(InventoryError::SqlName)
    ));
    assert!(matches!(
        shorten_sql_name("a_é".to_owned()),
        Err(InventoryError::SqlName)
    ));
    assert!(matches!(
        SqlName::parse(&"a".repeat(64)),
        Err(InventoryError::SqlName)
    ));
    assert_eq!(MAX_DERIVED_SQL_NAME_BYTES, 128);
}

#[test]
fn given_deep_large_field_paths_when_projected_then_names_fit_and_remain_distinct() -> TestResult {
    let first = vec![FieldId::new(u32::MAX)?; 8];
    let mut second = first.clone();
    *second.last_mut().ok_or("field absent")? = FieldId::new(4_294_967_294)?;
    let first = Column::new(first, ScalarType::Text)?;
    let second = Column::new(second, ScalarType::Text)?;
    assert_eq!(first.name().len(), MAX_SQL_NAME_BYTES);
    assert_eq!(first.name().get(..54), second.name().get(..54));
    assert_ne!(first.name(), second.name());
    SqlName::parse(first.name())?;
    SqlName::parse(second.name())?;
    Ok(())
}
