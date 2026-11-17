use inventory_core::{
    catalog::runtime::compiled_inputs_sha256,
    record::{FieldId, Key, Record, Value},
};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, error::Error};

pub fn seal(mut catalog: BTreeMap<Key, Value>, mut runtime: Value) -> Result<Vec<u8>, Box<dyn Error>> {
    let models = Sha256::digest(Record::encode(&Value::Map(catalog.clone()))?.as_bytes());
    let reviewed = compiled_inputs_sha256(Record::encode(&runtime)?.root())?;
    let source = Sha256::digest(include_bytes!("seal.rs"));
    let Value::Map(extension) = &mut runtime else { return Err("runtime extension must be a map".into()); };
    let receipt = [
        (1, Value::Int(1)),
        (2, Value::Map(BTreeMap::from([
            (Key::Field(FieldId::new(1)?), Value::Int(1)),
            (Key::Field(FieldId::new(2)?), Value::Bytes(models.to_vec())),
        ]))),
        (3, Value::Int(1)),
        (4, Value::Bytes(source.to_vec())),
        (5, Value::Null),
        (6, Value::Bytes(source.to_vec())),
        (7, Value::Bytes(reviewed.to_vec())),
    ];
    let receipt = receipt.into_iter()
        .map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
        .collect::<Result<BTreeMap<_, _>, Box<dyn Error>>>()?;
    extension.insert(Key::Field(FieldId::new(7)?), Value::Map(receipt));
    catalog.insert(Key::Field(FieldId::new(60_000)?), runtime);
    Ok(Record::encode(&Value::Map(catalog))?.into_bytes())
}
