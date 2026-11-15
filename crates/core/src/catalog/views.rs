mod operation;
mod shape;
mod wire;
use super::{CatalogError, CatalogKey, integer, list, map_value, text};
use crate::record::{ListRef, MapRef, ValueRef, read_one};
pub use operation::{CompressionAlgorithm, HttpOperation, Operation, Paginator};
pub use shape::{ErrorKind, Member, ModeledError, Shape, Target, ValueConstraints};
pub(super) use wire::field_name;
pub use wire::{HttpBinding, TimestampFormat, WireTraits, XmlNamespace};

#[derive(Clone, Copy, Debug)]
pub struct Service<'a> {
    map: MapRef<'a>,
    shape_count: usize,
    shapes_bytes: &'a [u8],
    shape_offsets: &'a [u8],
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
        let shapes = list(map, CatalogKey::Shapes)?;
        let shapes_storage = shapes.encoded_storage();
        let offsets = match map.get_field(CatalogKey::ShapeOffsets.field()?)? {
            Some(ValueRef::Bytes(bytes)) => bytes,
            _ => return Err(CatalogError::Field(CatalogKey::ShapeOffsets)),
        };
        let (shape_count, shapes_bytes) =
            validate_shapes(shapes, shapes_storage, offsets, field_names, field_offsets)?;
        let service = Self {
            map,
            shape_count,
            shapes_bytes,
            shape_offsets: offsets,
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
        Ok(())
    }

    pub fn name(self) -> Result<&'a str, CatalogError> {
        text(self.map, CatalogKey::Name)
    }

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

    pub fn protocols(
        self,
    ) -> Result<impl Iterator<Item = Result<&'a str, CatalogError>>, CatalogError> {
        texts(list(self.map, CatalogKey::Protocols)?)
    }

    pub fn resources(
        self,
    ) -> Result<impl Iterator<Item = Result<&'a str, CatalogError>>, CatalogError> {
        texts(list(self.map, CatalogKey::Resources)?)
    }

    pub fn operations(
        self,
    ) -> Result<impl Iterator<Item = Result<Operation<'a>, CatalogError>>, CatalogError> {
        Ok(list(self.map, CatalogKey::Operations)?
            .iter()?
            .map(move |value| Ok(Operation::new(map_value(value?)?, self.shape_count))))
    }

    pub fn shape_count(self) -> usize {
        self.shape_count
    }

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

    pub fn shape(self, index: usize) -> Result<Option<Shape<'a>>, CatalogError> {
        let Some(encoded) = indexed_shape_bytes(
            self.shape_offsets,
            self.shapes_bytes,
            self.shape_count,
            index,
        )?
        else {
            return Ok(None);
        };
        let (value, consumed) = read_one(encoded)?;
        if consumed != encoded.len() {
            return Err(CatalogError::Reference);
        }
        Shape::new(
            map_value(value)?,
            self.shape_count,
            self.field_names,
            self.field_offsets,
        )
        .map(Some)
    }
}
fn indexed_shape_bytes<'a>(
    offsets: &[u8],
    shapes: &'a [u8],
    count: usize,
    index: usize,
) -> Result<Option<&'a [u8]>, CatalogError> {
    if index >= count {
        return Ok(None);
    }
    let table_start = index.checked_mul(4).ok_or(CatalogError::Overflow)?;
    let table_end = table_start.checked_add(4).ok_or(CatalogError::Overflow)?;
    let start = decode_offset(
        offsets
            .get(table_start..table_end)
            .ok_or(CatalogError::Reference)?,
    )?;
    let end = offsets
        .get(table_end..)
        .and_then(|tail| tail.get(..4))
        .map(decode_offset)
        .transpose()?
        .unwrap_or(shapes.len());
    shapes
        .get(start..end)
        .map(Some)
        .ok_or(CatalogError::Reference)
}
fn validate_shapes<'a>(
    shapes: ListRef<'a>,
    encoded: &'a [u8],
    offsets: &[u8],
    field_names: &[u8],
    field_offsets: &[u8],
) -> Result<(usize, &'a [u8]), CatalogError> {
    let count = shapes.count()?;
    if offsets.len() != count.checked_mul(4).ok_or(CatalogError::Overflow)? {
        return Err(CatalogError::Reference);
    }
    let end = scan_shapes(
        count,
        shapes.header_len()?,
        encoded,
        offsets,
        (field_names, field_offsets),
    )?;
    Ok((count, encoded.get(..end).ok_or(CatalogError::Reference)?))
}
fn scan_shapes(
    count: usize,
    header: usize,
    encoded: &[u8],
    offsets: &[u8],
    field_registry: (&[u8], &[u8]),
) -> Result<usize, CatalogError> {
    (0..count).try_fold(header, |end, index| {
        let start = offset_at(offsets, index)?;
        if start != end {
            return Err(CatalogError::Reference);
        }
        validate_shape_item(encoded, start, count, field_registry)
    })
}

fn offset_at(offsets: &[u8], index: usize) -> Result<usize, CatalogError> {
    let start = index.checked_mul(4).ok_or(CatalogError::Overflow)?;
    let end = start.checked_add(4).ok_or(CatalogError::Overflow)?;
    decode_offset(offsets.get(start..end).ok_or(CatalogError::Reference)?)
}

fn validate_shape_item(
    encoded: &[u8],
    start: usize,
    count: usize,
    field_registry: (&[u8], &[u8]),
) -> Result<usize, CatalogError> {
    let suffix = encoded.get(start..).ok_or(CatalogError::Reference)?;
    let (value, consumed) = read_one(suffix)?;
    let end = start.checked_add(consumed).ok_or(CatalogError::Overflow)?;
    Shape::new(map_value(value)?, count, field_registry.0, field_registry.1)?.validate()?;
    if consumed == 0 || end > encoded.len() {
        return Err(CatalogError::Reference);
    }
    Ok(end)
}

fn decode_offset(bytes: &[u8]) -> Result<usize, CatalogError> {
    let offset = u32::from_be_bytes(bytes.try_into().map_err(|_| CatalogError::Reference)?);
    usize::try_from(offset).map_err(|_| CatalogError::Reference)
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
