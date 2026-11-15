use inventory_core::{inventory::{MAX_RESOURCE_TAGS, ResourceKey, Tag, TagCoverage, TagObservation}, record::{FieldId, Key, MapRef, Record, Value, ValueRef}};
use crate::sweep_runtime::SweepError;

const PENDING: u32 = 65_529;
const MAX_CHECKPOINT: usize = 65_536;

/// Pending pairs are bound to a complete owner key and persisted atomically with page progress.
pub(super) struct Accumulation {
    owner: ResourceKey,
    tags: Vec<Tag>,
    missing: bool,
}

impl Accumulation {
    pub(super) fn restore(owner: &ResourceKey, token: Option<&[u8]>) -> Result<Self, SweepError> {
        let Some(bytes) = token else { return Ok(Self { owner: owner.clone(), tags: Vec::new(), missing: false }); };
        let record = Record::from_bytes(bytes.to_vec())?;
        let Some(ValueRef::Map(pending)) = record.root().get_field(FieldId::new(PENDING)?)? else {
            return Err(SweepError::Operation("tag checkpoint has no pending snapshot"));
        };
        match pending.get_field(FieldId::new(1)?)? {
            Some(ValueRef::Bytes(digest)) if digest == owner.digest().as_bytes() => {},
            _ => return Err(SweepError::Operation("tag checkpoint owner differs from trusted parent")),
        }
        let missing = match pending.get_field(FieldId::new(3)?)? {
            Some(ValueRef::Bool(value)) => value,
            _ => return Err(SweepError::Operation("tag checkpoint has invalid coverage")),
        };
        let mut tags = pending_tags(pending)?;
        tags.sort_unstable_by(|left, right| left.key().cmp(right.key()));
        if tags.windows(2).any(|pair| matches!(pair, [left, right] if left.key() == right.key())) {
            return Err(SweepError::Operation("tag checkpoint contains duplicate keys"));
        }
        Ok(Self { owner: owner.clone(), tags, missing })
    }

    pub(super) fn append(self, page: TagObservation) -> Result<Self, SweepError> {
        if page.resource() != &self.owner { return Err(SweepError::Operation("tag page owner differs from trusted parent")); }
        let missing = self.missing || page.coverage() != TagCoverage::Complete;
        let tags = page.tags().iter().try_fold(self.tags, |mut tags, pair| {
            match tags.binary_search_by(|known| known.key().cmp(pair.key())) {
                Ok(index) if tags.get(index) == Some(pair) => {},
                Ok(_) => return Err(SweepError::Operation("tag snapshot contains conflicting values across pages")),
                Err(index) => {
                    if tags.len() >= MAX_RESOURCE_TAGS { return Err(SweepError::Operation("tag snapshot exceeds resource bound")); }
                    tags.try_reserve(1)?;
                    tags.insert(index, pair.clone());
                }
            }
            Ok(tags)
        })?;
        Ok(Self { owner: self.owner, tags, missing })
    }

    pub(super) fn observation(&self, terminal: bool) -> Result<TagObservation, SweepError> {
        let coverage = match (terminal, self.missing, self.tags.is_empty()) {
            (true, false, _) => TagCoverage::Complete,
            (true, true, true) => TagCoverage::Unobserved,
            _ => TagCoverage::Partial,
        };
        TagObservation::new(self.owner.clone(), coverage, self.tags.clone()).map_err(SweepError::from)
    }

    pub(super) fn checkpoint(&self, cursor: Record) -> Result<Record, SweepError> {
        let Value::Map(mut values) = ValueRef::Map(cursor.root()).to_value()? else {
            return Err(SweepError::Operation("tag checkpoint cursor is not a map"));
        };
        let pairs = self.tags.iter().map(|tag| Value::List(vec![Value::Text(tag.key().to_owned()), Value::Text(tag.value().to_owned())])).collect();
        let pending = fields([(1, Value::Bytes(self.owner.digest().as_bytes().to_vec())), (2, Value::List(pairs)), (3, Value::Bool(self.missing))])?;
        values.insert(Key::Field(FieldId::new(PENDING)?), pending);
        let record = Record::encode(&Value::Map(values))?;
        if record.as_bytes().len() > MAX_CHECKPOINT { return Err(SweepError::Operation("pending tag snapshot exceeds checkpoint bound")); }
        Ok(record)
    }
}

fn pending_tags(pending: MapRef<'_>) -> Result<Vec<Tag>, SweepError> {
    let Some(ValueRef::List(pairs)) = pending.get_field(FieldId::new(2)?)? else { return Err(SweepError::Operation("tag checkpoint has invalid pairs")); };
    if pairs.count()? > MAX_RESOURCE_TAGS { return Err(SweepError::Operation("tag checkpoint exceeds resource bound")); }
    pairs.iter()?.map(|pair| {
        let ValueRef::List(pair) = pair? else { return Err(SweepError::Operation("tag checkpoint pair is not a list")); };
        let mut values = pair.iter()?;
        match (values.next().transpose()?, values.next().transpose()?, values.next()) {
            (Some(ValueRef::Text(key)), Some(ValueRef::Text(value)), None) => Tag::new(key, value).map_err(SweepError::from),
            _ => Err(SweepError::Operation("tag checkpoint pair is malformed")),
        }
    }).collect()
}

fn fields<const N: usize>(values: [(u32, Value); N]) -> Result<Value, SweepError> {
    values.into_iter().map(|(key, value)| Ok((Key::Field(FieldId::new(key)?), value)))
        .collect::<Result<std::collections::BTreeMap<_, _>, SweepError>>().map(Value::Map)
}

#[cfg(test)]
mod tests;
