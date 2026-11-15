use crate::{
    catalog::{CatalogError, CatalogKey},
    record::{FieldId, KeyRef, MapRef, ValueRef},
};

const MAX_PATTERN_BYTES: usize = 16_384;

/// Retained Smithy length and pattern traits, borrowed from the checked catalog.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ValueConstraints<'a> {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<&'a str>,
}

pub(super) fn read(map: MapRef<'_>) -> Result<ValueConstraints<'_>, CatalogError> {
    parse(map.get_field(CatalogKey::ValueConstraints.field()?)?
        .ok_or(CatalogError::Field(CatalogKey::ValueConstraints))?)
}

pub(super) fn validate(map: MapRef<'_>) -> Result<(), CatalogError> {
    if let Some(value) = map.get_field(CatalogKey::ValueConstraints.field()?)? {
        parse(value)?;
    }
    Ok(())
}

pub(super) fn nested(map: MapRef<'_>, key: CatalogKey) -> Result<ValueConstraints<'_>, CatalogError> {
    match map.get_field(key.field()?)? {
        Some(ValueRef::Map(fields)) => read(fields),
        _ => Err(CatalogError::Field(key)),
    }
}

pub(super) fn validate_nested(map: MapRef<'_>, key: CatalogKey) -> Result<(), CatalogError> {
    match map.get_field(key.field()?)? {
        Some(ValueRef::Map(fields)) => validate(fields),
        None => Ok(()),
        _ => Err(CatalogError::Field(key)),
    }
}

fn parse(value: ValueRef<'_>) -> Result<ValueConstraints<'_>, CatalogError> {
    if matches!(value, ValueRef::Null) {
        return Ok(ValueConstraints::default());
    }
    let ValueRef::Map(fields) = value else { return Err(invalid()); };
    if !(1..=3).contains(&fields.count()?) { return Err(invalid()); }
    fields.iter()?.try_for_each(|entry| {
        let (key, _) = entry?;
        match key {
            KeyRef::Field(id) if (1..=3).contains(&id.value()) => Ok(()),
            _ => Err(invalid()),
        }
    })?;
    let min_length = length(fields, 1)?;
    let max_length = length(fields, 2)?;
    if matches!((min_length, max_length), (Some(min), Some(max)) if min > max) {
        return Err(invalid());
    }
    let pattern = match fields.get_field(FieldId::new(3)?)? {
        None => None,
        Some(ValueRef::Text(value)) if value.len() <= MAX_PATTERN_BYTES => Some(value),
        _ => return Err(invalid()),
    };
    Ok(ValueConstraints { min_length, max_length, pattern })
}

fn length(fields: MapRef<'_>, id: u32) -> Result<Option<usize>, CatalogError> {
    match fields.get_field(FieldId::new(id)?)? {
        None => Ok(None),
        Some(ValueRef::Int(value)) => usize::try_from(value).map(Some).map_err(|_| invalid()),
        _ => Err(invalid()),
    }
}

fn invalid() -> CatalogError {
    CatalogError::Field(CatalogKey::ValueConstraints)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{Key, Record, Value};
    use std::collections::BTreeMap;
    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn constraints(entries: impl IntoIterator<Item = (u32, Value)>) -> Result<Value, CatalogError> {
        entries.into_iter().map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
            .collect::<Result<BTreeMap<_, _>, CatalogError>>().map(Value::Map)
    }

    fn check(value: Option<Value>, inspect: impl FnOnce(Result<ValueConstraints<'_>, CatalogError>) -> TestResult) -> TestResult {
        let mut fields = BTreeMap::new();
        if let Some(value) = value {
            fields.insert(Key::Field(CatalogKey::ValueConstraints.field()?), value);
        }
        let record = Record::encode(&Value::Map(fields))?;
        inspect(read(record.root()))
    }

    #[test]
    fn given_retained_constraints_when_borrowed_then_bounds_and_exact_pattern_survive() -> TestResult {
        check(Some(constraints([(1, Value::Int(2)), (2, Value::Int(7)), (3, Value::Text("^é+$".into()))])?), |result| {
            assert_eq!(result?, ValueConstraints { min_length: Some(2), max_length: Some(7), pattern: Some("^é+$") });
            Ok(())
        })
    }

    #[test]
    fn given_missing_metadata_when_identity_requests_constraints_then_absence_fails_closed() -> TestResult {
        check(None, |result| { assert!(matches!(result, Err(CatalogError::Field(CatalogKey::ValueConstraints)))); Ok(()) })?;
        check(Some(Value::Null), |result| { assert_eq!(result?, ValueConstraints::default()); Ok(()) })
    }

    #[test]
    fn given_malformed_constraints_when_borrowed_then_no_invalid_rule_is_accepted() -> TestResult {
        for value in [Value::Bool(true), constraints([(1, Value::Int(-1))])?,
            constraints([(1, Value::Int(8)), (2, Value::Int(7))])?,
            constraints([(2, Value::Text("7".into()))])?, constraints([(4, Value::Null)])?,
            constraints([])?, constraints([(1, Value::Null)])?, constraints([(3, Value::Null)])?,
            constraints([(3, Value::Bool(false))])?,
            constraints([(3, Value::Text("a".repeat(MAX_PATTERN_BYTES + 1)))])?] {
            check(Some(value), |result| { assert!(matches!(result, Err(CatalogError::Field(CatalogKey::ValueConstraints)))); Ok(()) })?;
        }
        Ok(())
    }
}
