use super::*;

pub(super) fn encode(pages: u32, current: &[u8], partial: bool) -> Result<Record, SweepError> {
    let mut map = std::collections::BTreeMap::new();
    map.insert(
        Key::Field(FieldId::new(PAGES)?),
        Value::Int(i64::from(pages)),
    );
    map.insert(
        Key::Field(FieldId::new(CURRENT)?),
        Value::Bytes(current.to_vec()),
    );
    map.insert(Key::Field(FieldId::new(PARTIAL)?), Value::Bool(partial));
    Record::encode(&Value::Map(map)).map_err(SweepError::from)
}

pub(super) fn integer(record: &Record, field_id: u32) -> Result<Option<i64>, SweepError> {
    Ok(match field(record, field_id)? {
        Some(ValueRef::Int(value)) => Some(value),
        Some(ValueRef::Null) | None => None,
        _ => {
            return Err(SweepError::Operation(
                "persisted pagination metadata has wrong type",
            ));
        }
    })
}

pub(super) fn bytes_field(record: &Record, field_id: u32) -> Result<Vec<u8>, SweepError> {
    match field(record, field_id)? {
        Some(ValueRef::Bytes(value)) if value.len() <= MAX_TOKEN_RECORD => Ok(value.to_vec()),
        _ => Err(SweepError::Operation(
            "persisted pagination cursor has invalid token",
        )),
    }
}

pub(super) fn field(record: &Record, field_id: u32) -> Result<Option<ValueRef<'_>>, SweepError> {
    record::path_value(record.root(), &[FieldId::new(field_id)?])
}
pub(super) fn token_digest(request: &Record) -> Result<[u8; 32], SweepError> {
    Ok(*request.content_hash(&Exclusions::none())?.as_bytes())
}
pub(super) fn terminal_partial(
    truncated: bool,
    has_next: bool,
    pages: u32,
    limit: u32,
    token_size: usize,
) -> bool {
    (truncated && !has_next) || pages >= limit || token_size > MAX_TOKEN_RECORD
}
