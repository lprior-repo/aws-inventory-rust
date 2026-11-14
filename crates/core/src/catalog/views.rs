mod operation;
mod shape;
mod wire;
use super::{CatalogError, CatalogKey, integer, list, map_value, text};
use crate::record::{ListRef, MapRef, ValueRef};
pub use operation::{CompressionAlgorithm, HttpOperation, Operation, Paginator};
pub use shape::{ErrorKind, Member, ModeledError, Shape, Target};
pub(super) use wire::field_name;
pub use wire::{HttpBinding, TimestampFormat, WireTraits, XmlNamespace};

/// One structurally validated service record, borrowing its containing catalog.
#[derive(Clone, Copy, Debug)]
pub struct Service<'a> {
    map: MapRef<'a>,
    shape_count: usize,
    field_names: &'a [u8],
    field_offsets: &'a [u8],
}
impl<'a> Service<'a> {
    pub(super) fn open(
        bytes: &'a [u8],
        field_names: &'a [u8],
        field_offsets: &'a [u8],
    ) -> Result<Self, CatalogError> {
        let map = MapRef::from_bytes_bounded(bytes)?;
        let shape_count = list(map, CatalogKey::Shapes)?
            .iter()?
            .try_fold(0usize, |n, item| {
                item?;
                n.checked_add(1).ok_or(CatalogError::Overflow)
            })?;
        let service = Self {
            map,
            shape_count,
            field_names,
            field_offsets,
        };
        service.validate()?;
        Ok(service)
    }
    fn validate(self) -> Result<(), CatalogError> {
        self.name()?;
        self.shape_id()?;
        self.version()?;
        self.sdk_id()?;
        self.endpoint_prefix()?;
        self.arn_namespace()?;
        self.signing_name()?;
        self.protocols()?.try_for_each(|item| item.map(|_| ()))?;
        self.resources()?.try_for_each(|item| item.map(|_| ()))?;
        self.wire()?.validate()?;
        self.aws_query_compatible()?;
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
    pub fn version(self) -> Result<&'a str, CatalogError> {
        text(self.map, CatalogKey::ApiVersion)
    }
    pub fn sdk_id(self) -> Result<Option<&'a str>, CatalogError> {
        optional_text(self.map, CatalogKey::SdkId)
    }
    pub fn endpoint_prefix(self) -> Result<Option<&'a str>, CatalogError> {
        optional_text(self.map, CatalogKey::EndpointPrefix)
    }
    pub fn arn_namespace(self) -> Result<Option<&'a str>, CatalogError> {
        optional_text(self.map, CatalogKey::ArnNamespace)
    }
    pub fn signing_name(self) -> Result<Option<&'a str>, CatalogError> {
        optional_text(self.map, CatalogKey::Sigv4Name)
    }
    pub fn wire(self) -> Result<WireTraits<'a>, CatalogError> {
        Ok(WireTraits::service(self.map))
    }
    pub fn aws_query_compatible(self) -> Result<bool, CatalogError> {
        bool_field(self.map, CatalogKey::AwsQueryCompatible)
    }
    pub fn no_error_wrapping(self, protocol: &str) -> Result<bool, CatalogError> {
        let values = match self.map.get_field(CatalogKey::NoErrorWrapping.field()?)? {
            None => return Ok(false),
            Some(ValueRef::List(values)) => values,
            _ => return Err(CatalogError::Field(CatalogKey::NoErrorWrapping)),
        };
        texts(values)?.try_fold(false, |found, value| Ok(found || value? == protocol))
    }
    pub fn field_name(self, id: crate::record::FieldId) -> Result<&'a str, CatalogError> {
        field_name(self.field_names, self.field_offsets, id)
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
            .map(move |value| Ok(Operation::new(map_value(value?)?, self.shape_count))))
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
            .map(move |value| {
                Shape::new(
                    map_value(value?)?,
                    self.shape_count,
                    self.field_names,
                    self.field_offsets,
                )
            }))
    }
    /// Look up a zero-based local shape index; an absent index is not corruption.
    /// # Errors
    /// Reports malformed selected shape maps.
    pub fn shape(self, index: usize) -> Result<Option<Shape<'a>>, CatalogError> {
        list(self.map, CatalogKey::Shapes)?
            .get(index)?
            .map(|value| {
                Shape::new(
                    map_value(value)?,
                    self.shape_count,
                    self.field_names,
                    self.field_offsets,
                )
            })
            .transpose()
    }
}
pub(super) fn texts(
    list: ListRef<'_>,
) -> Result<impl Iterator<Item = Result<&str, CatalogError>>, CatalogError> {
    Ok(list.iter()?.map(|value| match value? {
        ValueRef::Text(text) => Ok(text),
        _ => Err(CatalogError::Reference),
    }))
}

pub(super) fn optional_text(
    map: MapRef<'_>,
    key: CatalogKey,
) -> Result<Option<&str>, CatalogError> {
    match map.get_field(key.field()?)? {
        None => Ok(None),
        Some(ValueRef::Text(value)) => Ok(Some(value)),
        _ => Err(CatalogError::Field(key)),
    }
}

pub(super) fn bool_field(map: MapRef<'_>, key: CatalogKey) -> Result<bool, CatalogError> {
    match map.get_field(key.field()?)? {
        None => Ok(false),
        Some(ValueRef::Bool(value)) => Ok(value),
        _ => Err(CatalogError::Field(key)),
    }
}

pub(super) fn optional_list<'a>(
    map: MapRef<'a>,
    key: CatalogKey,
) -> Result<
    Option<impl Iterator<Item = Result<ValueRef<'a>, crate::record::RecordError>> + 'a>,
    CatalogError,
> {
    match map.get_field(key.field()?)? {
        None => Ok(None),
        Some(ValueRef::List(value)) => Ok(Some(value.iter()?)),
        _ => Err(CatalogError::Field(key)),
    }
}
