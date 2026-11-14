//! Borrowed API catalog. Opening validates only the root; service byte strings are opaque.
//! Structural validation uses an inline 32-frame array without heap allocation.
//! Opening, service access, lookup, iteration and views allocate nothing.
//! Report aggregation is a cold, allocating path.
mod counts;
mod keys;
#[cfg(test)]
mod tests;
mod views;

use crate::record::{ListRef, MapRef, RecordError, ValueKind, ValueRef};
pub use counts::Counts;
pub use keys::{CatalogKey, SHAPE_KINDS, kind_code};
pub use views::{Member, Operation, Service, Shape, Target};

/// Catalog schema version, independent of the underlying record format.
pub const CATALOG_VERSION: i64 = 1;

/// Malformed or unsupported catalog data.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CatalogError {
    /// The zero-length build placeholder was not generated.
    #[error("empty catalog; run `cargo run -p catalog-tool -- models compile --write`")]
    Empty,
    /// Invalid record structure or supported-form violation.
    #[error(transparent)]
    Record(#[from] RecordError),
    /// A required catalog field is absent or has the wrong shape.
    #[error("invalid catalog field {0:?}")]
    Field(CatalogKey),
    /// Unsupported schema version.
    #[error("unsupported catalog version {0}")]
    Version(i64),
    /// Invalid shape index, kind, member identity or operation flags.
    #[error("invalid catalog reference or code")]
    Reference,
    /// Checked count arithmetic overflowed.
    #[error("catalog count overflow")]
    Overflow,
    /// Rendering the report failed.
    #[error(transparent)]
    Formatting(#[from] std::fmt::Error),
}

/// Allocation-free borrowed root catalog view with inline, depth-bounded validation.
#[derive(Clone, Copy, Debug)]
pub struct Catalog<'a> {
    root: MapRef<'a>,
    services: ListRef<'a>,
}

impl Catalog<'static> {
    /// Open embedded bytes, validating the root once without reading service payloads.
    /// # Errors
    /// Rejects empty, oversized, malformed or unsupported root records.
    pub fn open(bytes: &'static [u8]) -> Result<Self, CatalogError> {
        Self::borrow(bytes)
    }
}

impl<'a> Catalog<'a> {
    /// Open caller-owned bytes with the same allocation-free, bounded checks as `open`.
    /// # Errors
    /// Returns structural, schema, field or version errors.
    pub fn borrow(bytes: &'a [u8]) -> Result<Self, CatalogError> {
        if bytes.is_empty() {
            return Err(CatalogError::Empty);
        }
        let root = MapRef::from_bytes_bounded(bytes)?;
        let version = integer(root, CatalogKey::Format)?;
        if version != CATALOG_VERSION {
            return Err(CatalogError::Version(version));
        }
        text(root, CatalogKey::ModelsCommit)?;
        let services = list(root, CatalogKey::Services)?;
        services
            .iter()?
            .try_for_each(|value| bytes_value(value?).map(|_| ()))?;
        Ok(Self { root, services })
    }

    /// Return the upstream commit, borrowing the root.
    /// # Errors
    /// Returns a typed field error if the commit is malformed.
    pub fn models_commit(self) -> Result<&'a str, CatalogError> {
        text(self.root, CatalogKey::ModelsCommit)
    }

    /// Validate each service on first access. Retaining its view avoids revalidation.
    /// # Errors
    /// Iterator entries report malformed service records, independently of the root.
    pub fn services(
        self,
    ) -> Result<impl Iterator<Item = Result<Service<'a>, CatalogError>>, CatalogError> {
        Ok(self
            .services
            .iter()?
            .map(|value| Service::open(bytes_value(value?)?)))
    }

    /// Find a service by its unique model directory name, e.g. `ec2`.
    /// # Errors
    /// Returns the first malformed service encountered, never silently skipping it.
    pub fn service(self, name: &str) -> Result<Option<Service<'a>>, CatalogError> {
        self.services()?.try_fold(None, |found, service| {
            let service = service?;
            if service.name()? != name {
                return Ok(found);
            }
            if found.is_some() {
                return Err(CatalogError::Reference);
            }
            Ok(Some(service))
        })
    }

    /// Compute report totals by walking embedded records, not model files or cached totals.
    /// # Errors
    /// Returns any malformed field/reference or count overflow.
    pub fn counts(self) -> Result<Counts<'a>, CatalogError> {
        Counts::read(self)
    }
}

fn value(map: MapRef<'_>, key: CatalogKey, kind: ValueKind) -> Result<ValueRef<'_>, CatalogError> {
    map.get_typed(key.field()?, kind)?
        .ok_or(CatalogError::Field(key))
}
fn text(map: MapRef<'_>, key: CatalogKey) -> Result<&str, CatalogError> {
    match value(map, key, ValueKind::Text)? {
        ValueRef::Text(text) => Ok(text),
        _ => Err(CatalogError::Field(key)),
    }
}
fn integer(map: MapRef<'_>, key: CatalogKey) -> Result<i64, CatalogError> {
    match value(map, key, ValueKind::Int)? {
        ValueRef::Int(number) => Ok(number),
        _ => Err(CatalogError::Field(key)),
    }
}
fn list(map: MapRef<'_>, key: CatalogKey) -> Result<ListRef<'_>, CatalogError> {
    match value(map, key, ValueKind::List)? {
        ValueRef::List(list) => Ok(list),
        _ => Err(CatalogError::Field(key)),
    }
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
