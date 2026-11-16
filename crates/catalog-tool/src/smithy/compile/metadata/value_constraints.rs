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
        if object.keys().any(|key| !matches!(key.as_str(), "min" | "max" | "unit")) {
            return Err(invalid(LENGTH));
        }
        // A byte-unit bound measures encoded size, not the item or code-point
        // count that fields 1 and 2 carry, so it is validated but never emitted.
        let counted = match object.get("unit").map(Json::as_str) {
            None => true,
            Some(Some("bytes")) => false,
            Some(_) => return Err(invalid(LENGTH)),
        };
        let min = bound(length, "min")?;
        let max = bound(length, "max")?;
        if matches!((min, max), (Some(min), Some(max)) if min > max) {
            return Err(invalid(LENGTH));
        }
        for (id, value) in [(1, min), (2, max)] {
            if let Some(value) = value.filter(|_| counted) {
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
            serde_json::json!({"minimum":1}), serde_json::json!({"max":4,"unit":"items"}),
            serde_json::json!({"max":4,"unit":1}), serde_json::json!({"min":5,"max":4,"unit":"bytes"})] {
            assert!(matches!(field(&BTreeMap::from([(LENGTH.into(), length)])),
                Err(CompileError::Trait(name)) if name == LENGTH));
        }
        for pattern in [serde_json::json!(false), serde_json::json!("x".repeat(MAX_PATTERN_BYTES + 1))] {
            assert!(matches!(field(&BTreeMap::from([(PATTERN.into(), pattern)])),
                Err(CompileError::Trait(name)) if name == PATTERN));
        }
    }

    #[test]
    fn given_a_byte_unit_length_when_compiled_then_no_count_bound_is_emitted() -> TestResult {
        // Given: the pinned verifiedpermissions EncryptionContext map trait.
        let traits = BTreeMap::from([(LENGTH.into(), serde_json::json!({"max":8192,"unit":"bytes"}))]);
        // When/Then: the size bound is not misreported as an entry count.
        assert_eq!(field(&traits)?, (CatalogKey::ValueConstraints, Value::Null));
        Ok(())
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

    #[test]
    fn given_pinned_shape_constraints_when_deserialized_then_exact_metadata_is_compiled() -> TestResult {
        // Given: the exact trait payloads on the rejecting pinned string shapes.
        let cases = [
            (serde_json::json!({"min":12,"max":12}), Some(r"^\d{12}$"), Some(12), Some(12)),
            (serde_json::json!({"min":0,"max":1024}), None, Some(0), Some(1024)),
            (serde_json::json!({"min":20,"max":2048}), Some(r"^arn:aws[0-9a-zA-Z_\-:\/]+$"), Some(20), Some(2048)),
            (Json::Null, Some("^[a-z0-9]+$"), None, None),
        ];
        for (length, pattern, min, max) in cases {
            let mut traits = BTreeMap::new();
            if !length.is_null() {
                traits.insert(LENGTH, length);
            }
            if let Some(pattern) = pattern {
                traits.insert(PATTERN, serde_json::json!(pattern));
            }
            // When: the production Smithy deserializer and metadata compiler run.
            let shape: crate::smithy::model::Shape = serde_json::from_value(
                serde_json::json!({"type":"string","traits":traits}),
            )?;
            let mut expected = BTreeMap::new();
            for (id, bound) in [(1, min), (2, max)] {
                if let Some(bound) = bound {
                    expected.insert(Key::Field(FieldId::new(id)?), Value::Int(bound));
                }
            }
            if let Some(pattern) = pattern {
                expected.insert(Key::Field(FieldId::new(3)?), Value::Text(pattern.into()));
            }
            // Then: every explicit bound and pattern survives without invented bounds.
            assert_eq!(field(&shape.traits)?, (CatalogKey::ValueConstraints, Value::Map(expected)));
        }
        Ok(())
    }

    #[test]
    fn given_pinned_access_member_length_when_deserialized_then_its_one_sided_bound_survives() -> TestResult {
        // Given: Access.actions and Access.resources both specify only max: 100.
        let member: crate::smithy::model::Member = serde_json::from_value(serde_json::json!({
            "target":"com.amazonaws.accessanalyzer#ActionsList",
            "traits":{"smithy.api#length":{"max":100}}
        }))?;
        // When: deserialized member traits pass through the metadata compiler.
        let encoded = field(&member.traits)?;
        // Then: the explicit maximum survives and no minimum is invented.
        assert_eq!(encoded, (CatalogKey::ValueConstraints, Value::Map(BTreeMap::from([
            (Key::Field(FieldId::new(2)?), Value::Int(100)),
        ]))));
        Ok(())
    }

    #[test]
    fn given_malformed_serialized_constraints_when_compiled_then_traits_are_not_discarded() -> TestResult {
        // Given: invalid types, bounds and unknown keys must survive parsing for rejection.
        let cases = [
            (LENGTH, serde_json::json!(null)),
            (LENGTH, serde_json::json!({"max":"100"})),
            (LENGTH, serde_json::json!({"max":100,"unknown":0})),
            (LENGTH, serde_json::json!({"min":-1})),
            (LENGTH, serde_json::json!({"min":5,"max":4})),
            (LENGTH, serde_json::json!({"max":1.5})),
            (PATTERN, serde_json::json!(null)),
            (PATTERN, serde_json::json!({"pattern":"^[a-z0-9]+$"})),
            (PATTERN, serde_json::json!(false)),
        ];
        for (name, payload) in cases {
            // When: malformed traits take the same parse-to-compile path as pinned models.
            let shape: crate::smithy::model::Shape = serde_json::from_value(
                serde_json::json!({"type":"string","traits":BTreeMap::from([(name, payload.clone())])}),
            )?;
            // Then: the payload is retained faithfully and fails with the exact trait name.
            assert_eq!(shape.traits.get(name), Some(&payload));
            assert!(matches!(field(&shape.traits), Err(CompileError::Trait(trait_name)) if trait_name == name));
        }
        Ok(())
    }
}
