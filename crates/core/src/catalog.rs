mod counts;
mod keys;
pub mod runtime;
#[cfg(test)]
mod tests;
mod views;

use crate::record::{FieldId, ListRef, MapRef, RecordError, ValueRef};
pub use counts::Counts;
pub use keys::{CatalogKey, SHAPE_KINDS, kind_code};
pub use views::{
    CompressionAlgorithm, ErrorKind, HttpBinding, HttpOperation, Member, ModeledError, Operation,
    Paginator, Service, Shape, Target, TimestampFormat, ValueConstraints, WireTraits, XmlNamespace,
};

pub const CATALOG_VERSION: i64 = 4;

pub const MAX_CATALOG_BYTES: usize = 67_108_864;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CatalogError {
    #[error("empty catalog; run `cargo run -p catalog-tool -- models compile --write`")]
    Empty,

    #[error(transparent)]
    Record(#[from] RecordError),

    #[error("invalid catalog field {0:?}")]
    Field(CatalogKey),

    #[error("unsupported catalog version {0}")]
    Version(i64),

    #[error("invalid catalog reference or code")]
    Reference,

    #[error("catalog count overflow")]
    Overflow,

    #[error(transparent)]
    Formatting(#[from] std::fmt::Error),
}

#[derive(Clone, Copy, Debug)]
pub struct Catalog<'a> {
    root: MapRef<'a>,
    services: ListRef<'a>,
    field_names: &'a [u8],
    field_offsets: &'a [u8],
}

impl Catalog<'static> {
    pub fn open(bytes: &'static [u8]) -> Result<Self, CatalogError> {
        Self::borrow(bytes)
    }
}

impl<'a> Catalog<'a> {
    pub fn borrow(bytes: &'a [u8]) -> Result<Self, CatalogError> {
        if bytes.is_empty() {
            return Err(CatalogError::Empty);
        }
        let root = MapRef::from_container_bytes(bytes, MAX_CATALOG_BYTES)?;
        let version = integer(root, CatalogKey::Format)?;
        if version != CATALOG_VERSION {
            return Err(CatalogError::Version(version));
        }
        text(root, CatalogKey::ModelsCommit)?;
        let field_names = bytes_field(root, CatalogKey::FieldNames)?;
        let field_offsets = bytes_field(root, CatalogKey::FieldNameOffsets)?;
        validate_field_registry(field_names, field_offsets)?;
        let services = list(root, CatalogKey::Services)?;
        services
            .iter()?
            .try_for_each(|value| bytes_value(value?).map(|_| ()))?;
        Ok(Self {
            root,
            services,
            field_names,
            field_offsets,
        })
    }

    pub fn models_commit(self) -> Result<&'a str, CatalogError> {
        text(self.root, CatalogKey::ModelsCommit)
    }

    pub fn services(
        self,
    ) -> Result<impl Iterator<Item = Result<Service<'a>, CatalogError>>, CatalogError> {
        Ok(self.services.iter()?.map(move |value| {
            Service::open(bytes_value(value?)?, self.field_names, self.field_offsets)
        }))
    }

    pub fn service(self, name: &str) -> Result<Option<Service<'a>>, CatalogError> {
        let found = self.services.iter()?.try_fold(None, |found, value| {
            let bytes = bytes_value(value?)?;
            if text(MapRef::from_bytes_bounded(bytes)?, CatalogKey::Name)? != name {
                return Ok(found);
            }
            if found.is_some() {
                return Err(CatalogError::Reference);
            }
            Ok(Some(bytes))
        })?;
        found
            .map(|bytes| Service::open(bytes, self.field_names, self.field_offsets))
            .transpose()
    }
    pub fn field_name(self, id: FieldId) -> Result<&'a str, CatalogError> {
        views::field_name(self.field_names, self.field_offsets, id)
    }

    pub fn counts(self) -> Result<Counts<'a>, CatalogError> {
        Counts::read(self)
    }
}

fn value(map: MapRef<'_>, key: CatalogKey) -> Result<ValueRef<'_>, CatalogError> {
    map.get_field(key.field()?)?.ok_or(CatalogError::Field(key))
}
fn text(map: MapRef<'_>, key: CatalogKey) -> Result<&str, CatalogError> {
    match value(map, key)? {
        ValueRef::Text(text) => Ok(text),
        _ => Err(CatalogError::Field(key)),
    }
}
fn integer(map: MapRef<'_>, key: CatalogKey) -> Result<i64, CatalogError> {
    match value(map, key)? {
        ValueRef::Int(number) => Ok(number),
        _ => Err(CatalogError::Field(key)),
    }
}
fn list(map: MapRef<'_>, key: CatalogKey) -> Result<ListRef<'_>, CatalogError> {
    match value(map, key)? {
        ValueRef::List(list) => Ok(list),
        _ => Err(CatalogError::Field(key)),
    }
}
fn bytes_field(map: MapRef<'_>, key: CatalogKey) -> Result<&[u8], CatalogError> {
    match value(map, key)? {
        ValueRef::Bytes(bytes) => Ok(bytes),
        _ => Err(CatalogError::Field(key)),
    }
}
fn validate_field_registry(names: &[u8], offsets: &[u8]) -> Result<(), CatalogError> {
    if !offsets.len().is_multiple_of(8) {
        return Err(CatalogError::Reference);
    }
    let mut previous = 0usize;
    for pair in offsets.as_chunks::<8>().0 {
        let start = usize::try_from(u32::from_be_bytes(
            pair.get(..4)
                .ok_or(CatalogError::Reference)?
                .try_into()
                .map_err(|_| CatalogError::Reference)?,
        ))
        .map_err(|_| CatalogError::Reference)?;
        let end = usize::try_from(u32::from_be_bytes(
            pair.get(4..8)
                .ok_or(CatalogError::Reference)?
                .try_into()
                .map_err(|_| CatalogError::Reference)?,
        ))
        .map_err(|_| CatalogError::Reference)?;
        if start != previous || end <= start {
            return Err(CatalogError::Reference);
        }
        let name = names.get(start..end).ok_or(CatalogError::Reference)?;
        std::str::from_utf8(name).map_err(|_| CatalogError::Reference)?;
        previous = end;
    }
    if previous != names.len() {
        return Err(CatalogError::Reference);
    }
    Ok(())
}
fn bytes_value(value: ValueRef<'_>) -> Result<&[u8], CatalogError> {
    match value {
        ValueRef::Bytes(bytes) => Ok(bytes),
        _ => Err(CatalogError::Field(CatalogKey::Services)),
    }
}
fn map_value(value: ValueRef<'_>) -> Result<MapRef<'_>, CatalogError> {
    match value {
        ValueRef::Map(map) => Ok(map),
        _ => Err(CatalogError::Reference),
    }
}
fn number(value: ValueRef<'_>) -> Result<i64, CatalogError> {
    match value {
        ValueRef::Int(number) => Ok(number),
        _ => Err(CatalogError::Reference),
    }
}
