use super::{
    Exclusions, FORMAT_VERSION, FieldId, KeyRef, ListRef, MapKeyExclusion, MapRef, Record,
    RecordError, Time, ValueRef, container_depth,
};
use minicbor::{Encoder, data::Tag, encode::Write};
use std::convert::Infallible;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ContentHash([u8; 32]);

impl ContentHash {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

struct HashWriter(blake3::Hasher);

impl Write for HashWriter {
    type Error = Infallible;

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Infallible> {
        self.0.update(bytes);
        Ok(())
    }
}

type Sink = Encoder<HashWriter>;

#[derive(Clone, Copy)]
struct Scope<'p> {
    paths: &'p [Box<[FieldId]>],
    map_keys: &'p [MapKeyExclusion],
    level: usize,
}

impl Scope<'_> {
    fn enter(self, id: FieldId) -> Result<Option<Self>, RecordError> {
        if self.paths.is_empty() && self.map_keys.is_empty() {
            return Ok(Some(self));
        }
        let at = |path: &[FieldId]| path.get(self.level).copied();
        let start = self
            .paths
            .partition_point(|path| at(path.as_ref()) < Some(id));
        let end = self
            .paths
            .partition_point(|path| at(path.as_ref()) <= Some(id));
        let paths = self.paths.get(start..end).ok_or(RecordError::Encode)?;
        let start = self
            .map_keys
            .partition_point(|mask| at(mask.path()) < Some(id));
        let end = self
            .map_keys
            .partition_point(|mask| at(mask.path()) <= Some(id));
        let map_keys = self.map_keys.get(start..end).ok_or(RecordError::Encode)?;
        let level = self.level.checked_add(1).ok_or(RecordError::DepthLimit)?;
        let ends_here = paths.first().is_some_and(|path| path.len() == level);
        Ok((!ends_here).then_some(Scope {
            paths,
            map_keys,
            level,
        }))
    }
}

pub(super) fn compute(
    record: &Record,
    exclusions: &Exclusions,
) -> Result<ContentHash, RecordError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&[FORMAT_VERSION]);
    let mut sink = Encoder::new(HashWriter(hasher));
    let scope = Scope {
        paths: exclusions.paths(),
        map_keys: exclusions.map_keys(),
        level: 0,
    };
    hash_map(&mut sink, record.root(), scope, container_depth(0)?)?;
    Ok(ContentHash(*sink.into_writer().0.finalize().as_bytes()))
}

pub fn selected_content_hash(value: ValueRef<'_>) -> Result<ContentHash, RecordError> {
    validate_selected(value)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(&[FORMAT_VERSION]);
    let mut sink = Encoder::new(HashWriter(hasher));
    let scope = Scope {
        paths: &[],
        map_keys: &[],
        level: 0,
    };
    sink.begin_map()?.str(super::SCALAR_ITEM_NAME)?;
    hash_value(&mut sink, value, scope, container_depth(0)?)?;
    sink.end()?;
    Ok(ContentHash(*sink.into_writer().0.finalize().as_bytes()))
}

fn validate_selected(value: ValueRef<'_>) -> Result<(), RecordError> {
    let bytes = match value {
        ValueRef::Map(map) => map.bytes,
        ValueRef::List(list) => list.bytes,
        ValueRef::Text(text) if text.len() > super::MAX_RECORD_BYTES => {
            return Err(RecordError::SizeLimit);
        }
        ValueRef::Bytes(bytes) if bytes.len() > super::MAX_RECORD_BYTES => {
            return Err(RecordError::SizeLimit);
        }
        _ => return Ok(()),
    };
    if bytes.len() > super::MAX_RECORD_BYTES {
        return Err(RecordError::SizeLimit);
    }
    let mut decoder = minicbor::Decoder::new(bytes);
    super::structure::validate(&mut decoder, Some(super::MAX_DEPTH.saturating_sub(1)))
}

fn hash_map(
    sink: &mut Sink,
    map: MapRef<'_>,
    scope: Scope<'_>,
    depth: usize,
) -> Result<(), RecordError> {
    sink.begin_map()?;
    map.iter()?.try_for_each(|entry| {
        let (key, value) = entry?;
        hash_entry(sink, key, value, scope, depth)
    })?;
    sink.end()?;
    Ok(())
}

fn hash_entry(
    sink: &mut Sink,
    key: KeyRef<'_>,
    value: ValueRef<'_>,
    scope: Scope<'_>,
    depth: usize,
) -> Result<(), RecordError> {
    let inner = match key {
        KeyRef::Name(name) => {
            if scope
                .map_keys
                .iter()
                .any(|mask| mask.path().len() == scope.level && mask.key() == name)
            {
                return Ok(());
            }
            sink.str(name)?;
            Scope {
                map_keys: &[],
                ..scope
            }
        }
        KeyRef::Field(id) => {
            let Some(inner) = scope.enter(id)? else {
                return Ok(());
            };
            sink.u32(id.value())?;
            inner
        }
    };
    hash_value(sink, value, inner, depth)
}

fn hash_value(
    sink: &mut Sink,
    value: ValueRef<'_>,
    scope: Scope<'_>,
    depth: usize,
) -> Result<(), RecordError> {
    match value {
        ValueRef::Null => sink.null(),
        ValueRef::Bool(v) => sink.bool(v),
        ValueRef::Int(v) => sink.i64(v),
        ValueRef::Float(v) => sink.f64(v.to_f64()),
        ValueRef::Text(v) => sink.str(v),
        ValueRef::Bytes(v) => sink.bytes(v),
        ValueRef::Time(v) => return hash_time(sink, v, depth),
        ValueRef::List(v) => return hash_list(sink, v, scope, container_depth(depth)?),
        ValueRef::Map(v) => return hash_map(sink, v, scope, container_depth(depth)?),
    }?;
    Ok(())
}

fn hash_list(
    sink: &mut Sink,
    list: ListRef<'_>,
    scope: Scope<'_>,
    depth: usize,
) -> Result<(), RecordError> {
    sink.begin_array()?;
    list.iter()?
        .try_for_each(|item| hash_value(sink, item?, scope, depth))?;
    sink.end()?;
    Ok(())
}

fn hash_time(sink: &mut Sink, time: Time, depth: usize) -> Result<(), RecordError> {
    container_depth(depth)?;
    sink.tag(Tag::new(1))?
        .array(2)?
        .i64(time.seconds())?
        .u32(time.nanoseconds())?;
    Ok(())
}
