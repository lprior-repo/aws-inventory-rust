mod builder;
mod containers;
mod encode;
mod error;
mod exclusions;
mod hash;
mod iter;
mod readers;
mod selected;
mod skip;
mod structure;
#[cfg(test)]
mod tests;
mod value;

pub use builder::{BuildError, RecordBuilder};
pub use containers::{ListRef, MapRef};
pub use error::RecordError;
pub use exclusions::{Exclusions, MapKeyExclusion};
pub use hash::{ContentHash, selected_content_hash};
pub use value::{FieldId, Float, Key, KeyRef, Time, Value, ValueKind, ValueRef};
pub use selected::{SCALAR_ITEM_NAME, scalar_item, scalar_record};

pub const MAX_RECORD_BYTES: usize = 16_777_216;

pub const MAX_DEPTH: usize = 32;

pub const FORMAT_VERSION: u8 = 1;

pub(crate) fn read_one(bytes: &[u8]) -> Result<(ValueRef<'_>, usize), RecordError> {
    let mut decoder = minicbor::Decoder::new(bytes);
    let value = readers::read_value(&mut decoder)?;
    Ok((value, decoder.position()))
}
pub fn encode_list(items: &[Value]) -> Result<Vec<u8>, RecordError> {
    encode::encode_list_record(items)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record(Vec<u8>);

impl Record {
    pub fn encode(value: &Value) -> Result<Self, RecordError> {
        if !matches!(value, Value::Map(_)) {
            return Err(RecordError::NotMap);
        }
        encode::encode(value).map(Self)
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, RecordError> {
        validate(&bytes)?;
        Ok(Self(bytes))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    pub fn root(&self) -> MapRef<'_> {
        MapRef { bytes: &self.0 }
    }

    pub fn content_hash(&self, exclusions: &Exclusions) -> Result<ContentHash, RecordError> {
        hash::compute(self, exclusions)
    }
}

fn container_depth(depth: usize) -> Result<usize, RecordError> {
    depth
        .checked_add(1)
        .filter(|next| *next <= MAX_DEPTH)
        .ok_or(RecordError::DepthLimit)
}

fn validate(bytes: &[u8]) -> Result<(), RecordError> {
    validate_with_depth(bytes, None)
}

fn validate_with_depth(bytes: &[u8], limit: Option<usize>) -> Result<(), RecordError> {
    if bytes.len() > MAX_RECORD_BYTES {
        return Err(RecordError::SizeLimit);
    }
    let mut decoder = minicbor::Decoder::new(bytes);
    if !matches!(
        decoder.datatype()?,
        minicbor::data::Type::Map | minicbor::data::Type::MapIndef
    ) {
        return Err(RecordError::NotMap);
    }
    structure::validate(&mut decoder, limit)?;
    if decoder.position() != bytes.len() {
        return Err(RecordError::NotOneItem {
            offset: decoder.position(),
        });
    }
    Ok(())
}
