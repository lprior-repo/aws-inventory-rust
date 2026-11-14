//! Every meaningful content change affects the digest.
use super::support::{TestResult, expected, fields, hash};
use inventory_core::record::Value;

#[test]
fn scalar_value_change_changes_hash() -> TestResult {
    // Given records differing in exactly one integer.
    let original = fields([(1, Value::Int(1))])?;
    let changed = fields([(1, Value::Int(2))])?;
    // When hashing, then each matches its independent input and differs.
    assert_eq!(hash(&original)?, expected(&[1, 0xBF, 1, 1, 0xFF]));
    assert_eq!(hash(&changed)?, expected(&[1, 0xBF, 1, 2, 0xFF]));
    assert_ne!(hash(&original)?, hash(&changed)?);
    Ok(())
}

#[test]
fn field_key_change_changes_hash() -> TestResult {
    // Given the same scalar under different field IDs.
    let original = fields([(1, Value::Int(1))])?;
    let changed = fields([(2, Value::Int(1))])?;
    // When hashing, then key bytes contribute as well as value bytes.
    assert_eq!(hash(&changed)?, expected(&[1, 0xBF, 2, 1, 0xFF]));
    assert_ne!(hash(&original)?, hash(&changed)?);
    Ok(())
}

#[test]
fn integer_to_text_change_changes_hash() -> TestResult {
    // Given equal-looking but differently typed values.
    let original = fields([(1, Value::Int(1))])?;
    let changed = fields([(1, Value::Text("1".into()))])?;
    // When hashing, then text retains its CBOR type and content.
    assert_eq!(hash(&changed)?, expected(&[1, 0xBF, 1, 0x61, b'1', 0xFF]));
    assert_ne!(hash(&original)?, hash(&changed)?);
    Ok(())
}

#[test]
fn list_reordering_changes_hash() -> TestResult {
    // Given the same list items in opposite orders.
    let original = fields([(1, Value::List(vec![Value::Int(1), Value::Int(2)]))])?;
    let changed = fields([(1, Value::List(vec![Value::Int(2), Value::Int(1)]))])?;
    // When hashing, then list order is preserved.
    assert_eq!(
        hash(&original)?,
        expected(&[1, 0xBF, 1, 0x9F, 1, 2, 0xFF, 0xFF])
    );
    assert_ne!(hash(&original)?, hash(&changed)?);
    Ok(())
}

#[test]
fn list_nesting_changes_hash() -> TestResult {
    // Given [[1], [2]] versus [[1, 2]], with the same flattened leaves.
    let separate = vec![
        Value::List(vec![Value::Int(1)]),
        Value::List(vec![Value::Int(2)]),
    ];
    let combined = vec![Value::List(vec![Value::Int(1), Value::Int(2)])];
    let original = fields([(1, Value::List(separate))])?;
    let changed = fields([(1, Value::List(combined))])?;
    // When hashing, then nested container boundaries cannot disappear.
    let input = [1, 0xBF, 1, 0x9F, 0x9F, 1, 0xFF, 0x9F, 2, 0xFF, 0xFF, 0xFF];
    assert_eq!(hash(&original)?, expected(&input));
    assert_ne!(hash(&original)?, hash(&changed)?);
    Ok(())
}
