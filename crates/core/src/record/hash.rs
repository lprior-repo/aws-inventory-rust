//! Content hash: BLAKE3 over the format version and the record's kept entries.
//!
//! Input: one byte `FORMAT_VERSION`, then the record re-encoded with maps and
//! lists in indefinite-length form, entries in stored order, scalars in the
//! fixed forms of the encoding table. An entry whose field-ID path is excluded
//! contributes nothing, so a record lacking it hashes the same.
use super::{
    Exclusions, FORMAT_VERSION, FieldId, KeyRef, ListRef, MapRef, Record, RecordError, Time,
    ValueRef, container_depth,
};
use minicbor::{Encoder, data::Tag, encode::Write};
use std::convert::Infallible;

/// A 32-byte BLAKE3 content hash.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ContentHash([u8; 32]);

impl ContentHash {
    /// Borrow the 32 hash bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
    /// Wrap 32 stored hash bytes.
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

/// Exclusion paths that share the field IDs already entered; `level` is how many.
#[derive(Clone, Copy)]
struct Scope<'p> {
    paths: &'p [Box<[FieldId]>],
    level: usize,
}

impl Scope<'_> {
    /// Enter field `id`: `None` when a path ends here, else the narrowed scope.
    fn enter(self, id: FieldId) -> Result<Option<Self>, RecordError> {
        if self.paths.is_empty() {
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
        let level = self.level.checked_add(1).ok_or(RecordError::DepthLimit)?;
        let ends_here = paths.first().is_some_and(|path| path.len() == level);
        Ok((!ends_here).then_some(Scope { paths, level }))
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
        level: 0,
    };
    hash_map(&mut sink, record.root(), scope, container_depth(0)?)?;
    Ok(ContentHash(*sink.into_writer().0.finalize().as_bytes()))
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
            sink.str(name)?;
            scope
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
