//! Validated service and operation views.
mod shape;
use super::{CatalogError, CatalogKey, integer, list, map_value, text};
use crate::record::{ListRef, MapRef, ValueRef};
pub use shape::{Member, Shape, Target};

/// One structurally validated service record, borrowing its containing catalog.
#[derive(Clone, Copy, Debug)]
pub struct Service<'a> {
    map: MapRef<'a>,
    shape_count: usize,
}
impl<'a> Service<'a> {
    pub(super) fn open(bytes: &'a [u8]) -> Result<Self, CatalogError> {
        let map = MapRef::from_bytes_bounded(bytes)?;
        let shape_count = list(map, CatalogKey::Shapes)?
            .iter()?
            .try_fold(0usize, |n, item| {
                item?;
                n.checked_add(1).ok_or(CatalogError::Overflow)
            })?;
        let service = Self { map, shape_count };
        service.validate()?;
        Ok(service)
    }
    fn validate(self) -> Result<(), CatalogError> {
        self.name()?;
        self.shape_id()?;
        self.protocols()?.try_for_each(|item| item.map(|_| ()))?;
        self.resources()?.try_for_each(|item| item.map(|_| ()))?;
        self.operations()?.try_for_each(|item| item?.validate())?;
        self.shapes()?.try_for_each(|item| item?.validate())?;
        Ok(())
    }
    /// Unique model directory name, e.g. `ec2` or `storage-gateway`.
    /// # Errors
    /// Reports a missing or mistyped name.
    pub fn name(self) -> Result<&'a str, CatalogError> {
        text(self.map, CatalogKey::Name)
    }
    /// Full Smithy service shape ID, retained separately from the directory identity.
    /// # Errors
    /// Reports a missing or mistyped shape identity.
    pub fn shape_id(self) -> Result<&'a str, CatalogError> {
        text(self.map, CatalogKey::ServiceShape)
    }
    /// Protocol trait IDs in bytewise order.
    /// # Errors
    /// Reports malformed protocol lists or entries.
    pub fn protocols(
        self,
    ) -> Result<impl Iterator<Item = Result<&'a str, CatalogError>>, CatalogError> {
        texts(list(self.map, CatalogKey::Protocols)?)
    }
    /// Resource shape names in full-shape-ID order.
    /// # Errors
    /// Reports malformed resource lists or entries.
    pub fn resources(
        self,
    ) -> Result<impl Iterator<Item = Result<&'a str, CatalogError>>, CatalogError> {
        texts(list(self.map, CatalogKey::Resources)?)
    }
    /// Operations in full-shape-ID byte order.
    /// # Errors
    /// Reports malformed operation maps.
    pub fn operations(
        self,
    ) -> Result<impl Iterator<Item = Result<Operation<'a>, CatalogError>>, CatalogError> {
        Ok(list(self.map, CatalogKey::Operations)?
            .iter()?
            .map(move |value| {
                Ok(Operation {
                    map: map_value(value?)?,
                    shape_count: self.shape_count,
                })
            }))
    }
    /// Shape count bounded by this service record's declared list length.
    pub fn shape_count(self) -> usize {
        self.shape_count
    }
    /// Shapes in full-ID byte order.
    /// # Errors
    /// Reports malformed shape maps.
    pub fn shapes(
        self,
    ) -> Result<impl Iterator<Item = Result<Shape<'a>, CatalogError>>, CatalogError> {
        Ok(list(self.map, CatalogKey::Shapes)?
            .iter()?
            .map(move |value| Shape::new(map_value(value?)?, self.shape_count)))
    }
    /// Look up a zero-based local shape index; an absent index is not corruption.
    /// # Errors
    /// Reports malformed selected shape maps.
    pub fn shape(self, index: usize) -> Result<Option<Shape<'a>>, CatalogError> {
        list(self.map, CatalogKey::Shapes)?
            .get(index)?
            .map(|value| Shape::new(map_value(value)?, self.shape_count))
            .transpose()
    }
}

/// Borrowed operation with bounded input and output references.
#[derive(Clone, Copy, Debug)]
pub struct Operation<'a> {
    map: MapRef<'a>,
    shape_count: usize,
}
impl<'a> Operation<'a> {
    /// Operation shape name.
    /// # Errors
    /// Reports malformed name fields.
    pub fn name(self) -> Result<&'a str, CatalogError> {
        text(self.map, CatalogKey::Name)
    }
    /// Flags: readonly=1, paginated=2, List/Describe=4, input-free=8, read-named=16.
    /// # Errors
    /// Rejects negative or unknown flags.
    pub fn flags(self) -> Result<u8, CatalogError> {
        let flags = u8::try_from(integer(self.map, CatalogKey::Flags)?)
            .map_err(|_| CatalogError::Reference)?;
        if flags > 31 {
            return Err(CatalogError::Reference);
        }
        Ok(flags)
    }
    /// Input shape (absent Smithy input is the Unit prelude).
    /// # Errors
    /// Rejects out-of-range references.
    pub fn input(self) -> Result<Target, CatalogError> {
        shape::target(integer(self.map, CatalogKey::Input)?, self.shape_count)
    }
    /// Output shape (absent Smithy output is the Unit prelude).
    /// # Errors
    /// Rejects out-of-range references.
    pub fn output(self) -> Result<Target, CatalogError> {
        shape::target(integer(self.map, CatalogKey::Output)?, self.shape_count)
    }
    fn validate(self) -> Result<(), CatalogError> {
        self.name()?;
        self.flags()?;
        self.input()?;
        self.output()?;
        Ok(())
    }
}
fn texts(
    list: ListRef<'_>,
) -> Result<impl Iterator<Item = Result<&str, CatalogError>>, CatalogError> {
    Ok(list.iter()?.map(|value| match value? {
        ValueRef::Text(text) => Ok(text),
        _ => Err(CatalogError::Reference),
    }))
}
