use super::support::{TestResult, expected, fields, hash};
use inventory_core::record::Value;

#[test]
fn scalar_value_change_changes_hash() -> TestResult {
    let original = fields([(1, Value::Int(1))])?;
    let changed = fields([(1, Value::Int(2))])?;

    assert_eq!(hash(&original)?, expected(&[1, 0xBF, 1, 1, 0xFF]));
    assert_eq!(hash(&changed)?, expected(&[1, 0xBF, 1, 2, 0xFF]));
    assert_ne!(hash(&original)?, hash(&changed)?);
    Ok(())
}

#[test]
fn field_key_change_changes_hash() -> TestResult {
    let original = fields([(1, Value::Int(1))])?;
    let changed = fields([(2, Value::Int(1))])?;

    assert_eq!(hash(&changed)?, expected(&[1, 0xBF, 2, 1, 0xFF]));
    assert_ne!(hash(&original)?, hash(&changed)?);
    Ok(())
}

#[test]
fn integer_to_text_change_changes_hash() -> TestResult {
    let original = fields([(1, Value::Int(1))])?;
    let changed = fields([(1, Value::Text("1".into()))])?;

    assert_eq!(hash(&changed)?, expected(&[1, 0xBF, 1, 0x61, b'1', 0xFF]));
    assert_ne!(hash(&original)?, hash(&changed)?);
    Ok(())
}

#[test]
fn list_reordering_changes_hash() -> TestResult {
    let original = fields([(1, Value::List(vec![Value::Int(1), Value::Int(2)]))])?;
    let changed = fields([(1, Value::List(vec![Value::Int(2), Value::Int(1)]))])?;

    assert_eq!(
        hash(&original)?,
        expected(&[1, 0xBF, 1, 0x9F, 1, 2, 0xFF, 0xFF])
    );
    assert_ne!(hash(&original)?, hash(&changed)?);
    Ok(())
}

#[test]
fn list_nesting_changes_hash() -> TestResult {
    let separate = vec![
        Value::List(vec![Value::Int(1)]),
        Value::List(vec![Value::Int(2)]),
    ];
    let combined = vec![Value::List(vec![Value::Int(1), Value::Int(2)])];
    let original = fields([(1, Value::List(separate))])?;
    let changed = fields([(1, Value::List(combined))])?;

    let input = [1, 0xBF, 1, 0x9F, 0x9F, 1, 0xFF, 0x9F, 2, 0xFF, 0xFF, 0xFF];
    assert_eq!(hash(&original)?, expected(&input));
    assert_ne!(hash(&original)?, hash(&changed)?);
    Ok(())
}
