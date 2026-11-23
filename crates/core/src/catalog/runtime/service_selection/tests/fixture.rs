use crate::{
    catalog::{MAX_CATALOG_BYTES, runtime::compiled_inputs_sha256},
    record::{FieldId, Key, MapRef, Record, Value, ValueRef, encode_container},
};
use std::collections::BTreeMap;
type TestResult<T> = Result<T, Box<dyn std::error::Error>>;

pub(super) fn duplicate_plan(bytes: &[u8]) -> TestResult<Vec<u8>> {
    let mut root =
        ValueRef::Map(MapRef::from_container_bytes(bytes, MAX_CATALOG_BYTES)?).to_value()?;
    let extension = child(&mut root, 60_000)?;
    let plans = match fields(extension)?.get_mut(&key(2)?) {
        Some(Value::List(plans)) => plans,
        _ => return Err("fixture callable plans are missing".into()),
    };
    let duplicate = plans
        .iter()
        .find(|plan| is_account_identity(plan))
        .ok_or("fixture account identity is missing")?
        .clone();
    plans.push(duplicate);
    let encoded = Record::encode(extension)?;
    let reviewed = compiled_inputs_sha256(MapRef::from_bytes_bounded(encoded.as_bytes())?)?;
    fields(child(extension, 7)?)?.insert(key(7)?, Value::Bytes(reviewed.to_vec()));
    Ok(encode_container(&root, MAX_CATALOG_BYTES)?)
}

fn is_account_identity(plan: &Value) -> bool {
    match (plan, key(1), key(2)) {
        (Value::Map(fields), Ok(service), Ok(operation)) => {
            matches!(fields.get(&service), Some(Value::Text(name)) if name == "sts")
                && matches!(fields.get(&operation), Some(Value::Text(name)) if name == "GetCallerIdentity")
        }
        _ => false,
    }
}

fn key(id: u32) -> TestResult<Key> {
    Ok(Key::Field(FieldId::new(id)?))
}

fn fields(value: &mut Value) -> TestResult<&mut BTreeMap<Key, Value>> {
    match value {
        Value::Map(fields) => Ok(fields),
        _ => Err("fixture expected a map".into()),
    }
}

fn child(value: &mut Value, id: u32) -> TestResult<&mut Value> {
    fields(value)?
        .get_mut(&key(id)?)
        .ok_or_else(|| "fixture field is missing".into())
}
