use inventory_core::{
    catalog::runtime::{RuntimeCatalog, compiled_inputs_sha256},
    record::{FieldId, Key, MapRef, Record, Value, ValueRef},
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

type TestResult<T> = Result<T, Box<dyn std::error::Error>>;

fn key(id: u32) -> TestResult<Key> {
    Ok(Key::Field(FieldId::new(id)?))
}

fn fields(value: &mut Value) -> TestResult<&mut BTreeMap<Key, Value>> {
    match value {
        Value::Map(fields) => Ok(fields),
        _ => Err("runtime provenance fixture expected a map".into()),
    }
}

fn map(entries: impl IntoIterator<Item = (u32, Value)>) -> TestResult<Value> {
    Ok(Value::Map(
        entries
            .into_iter()
            .map(|(id, value)| Ok((key(id)?, value)))
            .collect::<TestResult<BTreeMap<_, _>>>()?,
    ))
}

pub fn base_bytes(base: &[u8]) -> TestResult<Vec<u8>> {
    let limit = inventory_core::catalog::MAX_CATALOG_BYTES;
    let mut root = ValueRef::Map(MapRef::from_container_bytes(base, limit)?).to_value()?;
    fields(&mut root)?
        .remove(&key(60_000)?)
        .ok_or("base catalog has no runtime extension")?;
    Ok(inventory_core::record::encode_container(&root, limit)?)
}

pub fn reseal(root: &mut Value, base: &[u8]) -> TestResult<()> {
    let runtime = RuntimeCatalog::from_bytes(base)?;
    let source = runtime.source_provenance();
    let extension = fields(root)?
        .get_mut(&key(60_000)?)
        .ok_or("fixture has no runtime extension")?;
    let encoded = Record::encode(extension)?;
    let reviewed = compiled_inputs_sha256(MapRef::from_bytes_bounded(encoded.as_bytes())?)?;
    let sha256: [u8; 32] = Sha256::digest(base_bytes(base)?).into();
    fields(extension)?.insert(
        key(7)?,
        map([
            (1, Value::Int(1)),
            (
                2,
                map([(1, Value::Int(1)), (2, Value::Bytes(sha256.to_vec()))])?,
            ),
            (3, Value::Int(i64::from(source.kind_map_version))),
            (4, Value::Bytes(source.kind_map_sha256.to_vec())),
            (5, Value::Null),
            (6, Value::Bytes(source.embedded_sources_sha256.to_vec())),
            (7, Value::Bytes(reviewed.to_vec())),
        ])?,
    );
    Ok(())
}
