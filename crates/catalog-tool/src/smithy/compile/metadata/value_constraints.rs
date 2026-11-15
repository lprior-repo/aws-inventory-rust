use super::{CompileError, Json};
use inventory_core::{catalog::CatalogKey, record::{FieldId, Key, Value}};
use std::collections::BTreeMap;

const LENGTH: &str = "smithy.api#length";
const PATTERN: &str = "smithy.api#pattern";
const MAX_PATTERN_BYTES: usize = 16_384;

pub(super) fn field(traits: &BTreeMap<String, Json>) -> Result<(CatalogKey, Value), CompileError> {
    let mut fields = BTreeMap::new();
    if let Some(length) = traits.get(LENGTH) {
        let object = length.as_object().ok_or_else(|| invalid(LENGTH))?;
        if object.keys().any(|key| !matches!(key.as_str(), "min" | "max")) {
            return Err(invalid(LENGTH));
        }
        let min = bound(length, "min")?;
        let max = bound(length, "max")?;
        if matches!((min, max), (Some(min), Some(max)) if min > max) {
            return Err(invalid(LENGTH));
        }
        for (id, value) in [(1, min), (2, max)] {
            if let Some(value) = value {
                fields.insert(Key::Field(FieldId::new(id)?), Value::Int(value));
            }
        }
    }
    if let Some(pattern) = traits.get(PATTERN) {
        let pattern = pattern.as_str().filter(|text| text.len() <= MAX_PATTERN_BYTES)
            .ok_or_else(|| invalid(PATTERN))?;
        fields.insert(Key::Field(FieldId::new(3)?), Value::Text(pattern.to_owned()));
    }
    Ok((CatalogKey::ValueConstraints, if fields.is_empty() { Value::Null } else { Value::Map(fields) }))
}

fn bound(length: &Json, key: &str) -> Result<Option<i64>, CompileError> {
    length.get(key).map(|value| value.as_i64().filter(|value| *value >= 0)
        .ok_or_else(|| invalid(LENGTH))).transpose()
}

fn invalid(trait_name: &str) -> CompileError {
    CompileError::Trait(trait_name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn given_source_length_and_pattern_when_compiled_then_exact_traits_are_retained() -> TestResult {
        let traits = BTreeMap::from([(LENGTH.into(), serde_json::json!({"min":0,"max":255})),
            (PATTERN.into(), serde_json::json!("^arn:aws:wisdom:[a-z0-9-]+:.*$"))]);
        let (key, encoded) = field(&traits)?;
        assert_eq!(key, CatalogKey::ValueConstraints);
        assert_eq!(encoded, Value::Map(BTreeMap::from([
            (Key::Field(FieldId::new(1)?), Value::Int(0)),
            (Key::Field(FieldId::new(2)?), Value::Int(255)),
            (Key::Field(FieldId::new(3)?), Value::Text("^arn:aws:wisdom:[a-z0-9-]+:.*$".into())),
        ])));
        assert_eq!(field(&BTreeMap::new())?, (CatalogKey::ValueConstraints, Value::Null));
        Ok(())
    }

    #[test]
    fn given_malformed_source_constraints_when_compiled_then_catalog_creation_fails() {
        for length in [serde_json::json!(1), serde_json::json!({"min":-1}),
            serde_json::json!({"max":"4"}), serde_json::json!({"min":5,"max":4}),
            serde_json::json!({"minimum":1})] {
            assert!(matches!(field(&BTreeMap::from([(LENGTH.into(), length)])),
                Err(CompileError::Trait(name)) if name == LENGTH));
        }
        for pattern in [serde_json::json!(false), serde_json::json!("x".repeat(MAX_PATTERN_BYTES + 1))] {
            assert!(matches!(field(&BTreeMap::from([(PATTERN.into(), pattern)])),
                Err(CompileError::Trait(name)) if name == PATTERN));
        }
    }

    #[test]
    fn given_one_sided_length_traits_when_compiled_then_no_missing_bound_is_invented() -> TestResult {
        for (source, id, expected) in [(serde_json::json!({"min":0}), 1, 0),
            (serde_json::json!({"max":255}), 2, 255)] {
            assert_eq!(field(&BTreeMap::from([(LENGTH.into(), source)]))?,
                (CatalogKey::ValueConstraints, Value::Map(BTreeMap::from([
                    (Key::Field(FieldId::new(id)?), Value::Int(expected)),
                ]))));
        }
        assert_eq!(field(&BTreeMap::from([(LENGTH.into(), serde_json::json!({}))]))?,
            (CatalogKey::ValueConstraints, Value::Null));
        Ok(())
    }

    #[test]
    fn given_empty_or_byte_bounded_patterns_when_compiled_then_the_exact_source_survives() -> TestResult {
        for pattern in [String::new(), "é".repeat(MAX_PATTERN_BYTES / 2)] {
            assert_eq!(field(&BTreeMap::from([(PATTERN.into(), serde_json::json!(&pattern))]))?,
                (CatalogKey::ValueConstraints, Value::Map(BTreeMap::from([
                    (Key::Field(FieldId::new(3)?), Value::Text(pattern)),
                ]))));
        }
        Ok(())
    }
}
