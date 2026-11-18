use crate::{
    catalog::{CatalogError, CatalogKey, runtime::BindingValue},
    record::{MapRef, ValueRef},
};

const MAX_VALUES: usize = 65_536;

pub(super) fn validate(map: MapRef<'_>, kind: &str) -> Result<(), CatalogError> {
    let Some(values) = map.get_field(CatalogKey::EnumValues.field()?)? else {
        return Ok(());
    };
    if matches!(values, ValueRef::Null) && kind == "string" {
        return Ok(());
    }
    let ValueRef::List(values) = values else {
        return Err(CatalogError::Field(CatalogKey::EnumValues));
    };
    if !matches!(kind, "string" | "enum" | "intEnum") {
        return Err(CatalogError::Field(CatalogKey::EnumValues));
    }
    values
        .iter()?
        .take(MAX_VALUES.saturating_add(1))
        .enumerate()
        .try_fold(0usize, |bytes, (index, value)| {
            bounded(kind, value?, bytes, index)
        })
        .map(|_| ())
}

pub(super) fn accepts(
    map: MapRef<'_>,
    kind: &str,
    expected: ValueRef<'_>,
) -> Result<bool, CatalogError> {
    let Some(values) = map.get_field(CatalogKey::EnumValues.field()?)? else {
        return Ok(!matches!(kind, "string" | "enum" | "intEnum"));
    };
    if matches!(values, ValueRef::Null) && kind == "string" {
        return Ok(true);
    }
    let ValueRef::List(values) = values else {
        return Err(CatalogError::Field(CatalogKey::EnumValues));
    };
    values
        .iter()?
        .take(MAX_VALUES.saturating_add(1))
        .enumerate()
        .try_fold((false, 0usize), |(found, bytes), (index, value)| {
            let value = value?;
            let bytes = bounded(kind, value, bytes, index)?;
            let matches = match (expected, value) {
                (ValueRef::Text(expected), ValueRef::Text(candidate)) => expected == candidate,
                (ValueRef::Int(expected), ValueRef::Int(candidate)) => expected == candidate,
                _ => false,
            };
            Ok((found || matches, bytes))
        })
        .map(|(found, _)| found)
}
fn bounded(
    kind: &str,
    value: ValueRef<'_>,
    bytes: usize,
    index: usize,
) -> Result<usize, CatalogError> {
    if index >= MAX_VALUES {
        return Err(CatalogError::Field(CatalogKey::EnumValues));
    }
    match (kind, value) {
        ("string" | "enum", ValueRef::Text(value)) => {
            let bytes = bytes
                .checked_add(value.len())
                .ok_or(CatalogError::Field(CatalogKey::EnumValues))?;
            if value.len() > BindingValue::MAX_STRING || bytes > BindingValue::MAX_TEXT_BYTES {
                return Err(CatalogError::Field(CatalogKey::EnumValues));
            }
            Ok(bytes)
        }
        ("intEnum", ValueRef::Int(value)) if i32::try_from(value).is_ok() => Ok(bytes),
        _ => Err(CatalogError::Field(CatalogKey::EnumValues)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{Key, Record, Value};
    use std::collections::BTreeMap;
    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn accepts(
        values: Option<Value>,
        kind: &str,
        expected: ValueRef<'_>,
    ) -> Result<bool, CatalogError> {
        let fields = match values {
            Some(values) => BTreeMap::from([(Key::Field(CatalogKey::EnumValues.field()?), values)]),
            None => BTreeMap::new(),
        };
        let record = Record::encode(&Value::Map(fields))?;
        super::validate(record.root(), kind)?;
        super::accepts(record.root(), kind, expected)
    }

    #[test]
    fn given_versioned_enum_metadata_when_absent_then_only_explicitly_unconstrained_strings_remain_usable()
    -> TestResult {
        assert!(!accepts(None, "enum", ValueRef::Text("anything"))?);
        assert!(!accepts(None, "intEnum", ValueRef::Int(1))?);
        assert!(!accepts(None, "string", ValueRef::Text("anything"))?);
        assert!(accepts(
            Some(Value::Null),
            "string",
            ValueRef::Text("anything")
        )?);
        for kind in ["enum", "intEnum", "integer", "structure"] {
            assert!(matches!(
                accepts(Some(Value::Null), kind, ValueRef::Text("anything")),
                Err(CatalogError::Field(CatalogKey::EnumValues))
            ));
        }
        assert!(accepts(
            Some(Value::List(vec![Value::Text("wire".into())])),
            "enum",
            ValueRef::Text("wire")
        )?);
        assert!(!accepts(
            Some(Value::List(vec![Value::Text("wire".into())])),
            "enum",
            ValueRef::Text("WIRE")
        )?);
        assert!(accepts(
            Some(Value::List(vec![Value::Int(7)])),
            "intEnum",
            ValueRef::Int(7)
        )?);
        assert!(!accepts(
            Some(Value::List(vec![Value::Int(7)])),
            "intEnum",
            ValueRef::Int(6)
        )?);
        assert!(!accepts(
            Some(Value::List(vec![Value::Text("wire".into())])),
            "string",
            ValueRef::Text("other")
        )?);
        Ok(())
    }

    #[test]
    fn given_enum_metadata_when_strings_or_table_bytes_exceed_bounds_then_exact_field_error_is_returned()
    -> TestResult {
        assert!(accepts(
            Some(Value::List(vec![Value::Text(
                "x".repeat(BindingValue::MAX_STRING)
            )])),
            "enum",
            ValueRef::Text(&"x".repeat(BindingValue::MAX_STRING))
        )?);
        let over_string = accepts(
            Some(Value::List(vec![Value::Text(
                "x".repeat(BindingValue::MAX_STRING.saturating_add(1)),
            )])),
            "enum",
            ValueRef::Text("x"),
        );
        assert!(matches!(
            over_string,
            Err(CatalogError::Field(CatalogKey::EnumValues))
        ));
        let at_limit = vec![Value::Text("x".repeat(BindingValue::MAX_STRING)); 64];
        assert!(accepts(
            Some(Value::List(at_limit.clone())),
            "enum",
            ValueRef::Text(&"x".repeat(BindingValue::MAX_STRING))
        )?);
        let mut over_limit = at_limit;
        over_limit.push(Value::Text("x".into()));
        assert!(matches!(
            accepts(Some(Value::List(over_limit)), "enum", ValueRef::Text("x")),
            Err(CatalogError::Field(CatalogKey::EnumValues))
        ));
        Ok(())
    }

    #[test]
    fn given_enum_metadata_when_count_or_value_type_is_invalid_then_exact_field_error_is_returned()
    -> TestResult {
        assert!(accepts(
            Some(Value::List(vec![Value::Int(7); MAX_VALUES])),
            "intEnum",
            ValueRef::Int(7)
        )?);
        assert!(matches!(
            accepts(
                Some(Value::List(vec![
                    Value::Int(7);
                    MAX_VALUES.saturating_add(1)
                ])),
                "intEnum",
                ValueRef::Int(7)
            ),
            Err(CatalogError::Field(CatalogKey::EnumValues))
        ));
        for (kind, value) in [
            ("enum", Value::Int(7)),
            ("intEnum", Value::Text("7".into())),
            ("intEnum", Value::Int(i64::MAX)),
        ] {
            assert!(matches!(
                accepts(Some(Value::List(vec![value])), kind, ValueRef::Int(7)),
                Err(CatalogError::Field(CatalogKey::EnumValues))
            ));
        }
        Ok(())
    }
}
