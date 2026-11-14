use inventory_core::record::*;

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub fn fields<const N: usize>(entries: [(u32, Value); N]) -> Result<Value, RecordError> {
    entries
        .into_iter()
        .map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
        .collect::<Result<_, _>>()
        .map(Value::Map)
}

pub fn paths<const N: usize>(entries: [&[u32]; N]) -> Result<Exclusions, RecordError> {
    let entries = entries
        .into_iter()
        .map(|path| path.iter().copied().map(FieldId::new).collect())
        .collect::<Result<Vec<Vec<FieldId>>, RecordError>>()?;
    Exclusions::new(entries)
}

pub fn expected(input: &[u8]) -> ContentHash {
    ContentHash::from_bytes(*blake3::hash(input).as_bytes())
}

pub fn hash(value: &Value) -> Result<ContentHash, RecordError> {
    Record::encode(value)?.content_hash(&Exclusions::none())
}

pub fn excluding(value: &Value, exclusions: &Exclusions) -> Result<ContentHash, RecordError> {
    Record::encode(value)?.content_hash(exclusions)
}
