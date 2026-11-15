use super::{BuildError, KeyRef, MapRef, Record, RecordBuilder, RecordError, ValueRef};

pub const SCALAR_ITEM_NAME: &str = "$";

pub fn scalar_record(value: ValueRef<'_>) -> Result<Record, BuildError> {
    let mut builder = RecordBuilder::new(super::MAX_RECORD_BYTES, 2, 2)?;
    builder.begin_map(None)?;
    builder.scalar(Some(KeyRef::Name(SCALAR_ITEM_NAME)), value)?;
    builder.end()?;
    builder.finish()
}

pub fn scalar_item(root: MapRef<'_>) -> Result<Option<ValueRef<'_>>, RecordError> {
    let mut entries = root.iter()?;
    let Some(entry) = entries.next().transpose()? else {
        return Ok(None);
    };
    let (key, value) = entry;
    if key != KeyRef::Name(SCALAR_ITEM_NAME) || entries.next().transpose()?.is_some() {
        return Ok(None);
    }
    match value {
        ValueRef::Map(_) | ValueRef::List(_) => Ok(None),
        scalar => Ok(Some(scalar)),
    }
}
