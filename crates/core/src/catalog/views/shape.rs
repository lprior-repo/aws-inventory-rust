mod enum_values;
mod error;
mod value_constraints;
use super::super::{CatalogError, CatalogKey, SHAPE_KINDS, integer, number, text};
use super::{optional_list, wire::WireTraits};
use crate::record::{FieldId, MapRef, ValueRef};
pub use error::{ErrorKind, ModeledError};
pub use value_constraints::ValueConstraints;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Target {
    Shape(usize),

    Prelude(&'static str),
}
pub(super) fn target(raw: i64, count: usize) -> Result<Target, CatalogError> {
    if raw >= 0 {
        let index = usize::try_from(raw).map_err(|_| CatalogError::Reference)?;
        return if index < count {
            Ok(Target::Shape(index))
        } else {
            Err(CatalogError::Reference)
        };
    }
    let code = raw
        .checked_neg()
        .and_then(|n| n.checked_sub(1))
        .and_then(|n| usize::try_from(n).ok())
        .ok_or(CatalogError::Reference)?;
    match SHAPE_KINDS.get(code).filter(|_| code <= 13) {
        Some(kind) => Ok(Target::Prelude(kind)),
        _ => Err(CatalogError::Reference),
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Shape<'a> {
    map: MapRef<'a>,
    count: usize,
    field_names: &'a [u8],
    field_offsets: &'a [u8],
}
impl<'a> Shape<'a> {
    pub(super) fn new(
        map: MapRef<'a>,
        count: usize,
        field_names: &'a [u8],
        field_offsets: &'a [u8],
    ) -> Result<Self, CatalogError> {
        let shape = Self {
            map,
            count,
            field_names,
            field_offsets,
        };
        shape.name()?;
        shape.kind()?;
        Ok(shape)
    }

    pub fn name(self) -> Result<&'a str, CatalogError> {
        text(self.map, CatalogKey::Name)
    }

    pub fn kind(self) -> Result<&'static str, CatalogError> {
        let code = usize::try_from(integer(self.map, CatalogKey::Kind)?)
            .map_err(|_| CatalogError::Reference)?;
        SHAPE_KINDS
            .get(code)
            .copied()
            .ok_or(CatalogError::Reference)
    }
    /// Tests a scalar against retained modeled enum wire values.
    ///
    /// # Errors
    /// Returns an error when enum metadata is malformed or exceeds its bounds.
    pub fn accepts_enum_value(self, value: ValueRef<'_>) -> Result<bool, CatalogError> {
        enum_values::accepts(self.map, self.kind()?, value)
    }
    /// Borrows the first retained enum wire value, if this shape declares any.
    ///
    /// # Errors
    /// Returns an error when the retained enum metadata is malformed.
    pub fn first_enum_value(self) -> Result<Option<ValueRef<'a>>, CatalogError> {
        enum_values::first(self.map, self.kind()?)
    }
    /// Borrows retained Smithy length, pattern and integer range constraints.
    ///
    /// Missing metadata fails closed; explicit null means unconstrained.
    pub fn value_constraints(self) -> Result<ValueConstraints<'a>, CatalogError> {
        value_constraints::read(self.map)
    }
    /// Constraints on a list's modeled member, separate from the member target.
    pub fn element_constraints(self) -> Result<ValueConstraints<'a>, CatalogError> {
        value_constraints::nested(self.map, CatalogKey::ElementWire)
    }
    /// Constraints on a map's modeled key member.
    pub fn map_key_constraints(self) -> Result<ValueConstraints<'a>, CatalogError> {
        value_constraints::nested(self.map, CatalogKey::MapKeyWire)
    }
    /// Constraints on a map's modeled value member.
    pub fn map_value_constraints(self) -> Result<ValueConstraints<'a>, CatalogError> {
        value_constraints::nested(self.map, CatalogKey::MapValueWire)
    }
    pub fn error(self) -> Result<Option<ModeledError<'a>>, CatalogError> {
        error::read(self.map)
    }
    pub fn wire(self) -> Result<WireTraits<'a>, CatalogError> {
        Ok(WireTraits::shape(self.map))
    }
    pub fn element_wire(self) -> Result<WireTraits<'a>, CatalogError> {
        WireTraits::nested(self.map, CatalogKey::ElementWire)
    }
    pub fn map_key_wire(self) -> Result<WireTraits<'a>, CatalogError> {
        WireTraits::nested(self.map, CatalogKey::MapKeyWire)
    }
    pub fn map_value_wire(self) -> Result<WireTraits<'a>, CatalogError> {
        WireTraits::nested(self.map, CatalogKey::MapValueWire)
    }

    pub fn members(
        self,
    ) -> Result<impl Iterator<Item = Result<Member<'a>, CatalogError>> + 'a, CatalogError> {
        let mut members = optional_list(self.map, CatalogKey::Members)?;
        let mut previous = None;
        Ok(std::iter::from_fn(move || {
            let value = members.as_mut()?.next()?;
            Some(
                read_member(value, self.count, self.field_names, self.field_offsets).and_then(
                    |member| {
                        if previous.is_some_and(|id| id >= member.id) {
                            return Err(CatalogError::Reference);
                        }
                        previous = Some(member.id);
                        Ok(member)
                    },
                ),
            )
        }))
    }

    pub fn element(self) -> Result<Option<Target>, CatalogError> {
        self.reference(CatalogKey::Element)
    }

    pub fn map_key(self) -> Result<Option<Target>, CatalogError> {
        self.reference(CatalogKey::MapKey)
    }

    pub fn map_value(self) -> Result<Option<Target>, CatalogError> {
        self.reference(CatalogKey::MapValue)
    }

    pub fn arn(self) -> Result<Option<&'a str>, CatalogError> {
        match self.map.get_field(CatalogKey::Arn.field()?)? {
            None => Ok(None),
            Some(ValueRef::Text(text)) => Ok(Some(text)),
            _ => Err(CatalogError::Field(CatalogKey::Arn)),
        }
    }
    /// Retained payload of the structure's Smithy reference declarations.
    pub fn references(self) -> Result<Option<&'a str>, CatalogError> {
        super::optional_text(self.map, CatalogKey::References)
    }

    /// Retained payload of the shape's ARN reference declaration.
    pub fn arn_reference(self) -> Result<Option<&'a str>, CatalogError> {
        super::optional_text(self.map, CatalogKey::ArnReference)
    }

    /// List and read operation names declared by a resource shape.
    pub fn resource_operations(
        self,
    ) -> Result<impl Iterator<Item = Result<&'a str, CatalogError>>, CatalogError> {
        Ok(optional_list(self.map, CatalogKey::ResourceOperations)?
            .into_iter()
            .flatten()
            .map(|value| match value? {
                ValueRef::Text(name) => Ok(name),
                _ => Err(CatalogError::Field(CatalogKey::ResourceOperations)),
            }))
    }
    /// Identifier names declared by a resource shape.
    pub fn resource_identifiers(
        self,
    ) -> Result<impl Iterator<Item = Result<&'a str, CatalogError>>, CatalogError> {
        Ok(optional_list(self.map, CatalogKey::ResourceIdentifiers)?
            .into_iter()
            .flatten()
            .map(|value| match value? {
                ValueRef::Text(name) => Ok(name),
                _ => Err(CatalogError::Field(CatalogKey::ResourceIdentifiers)),
            }))
    }
    fn reference(self, key: CatalogKey) -> Result<Option<Target>, CatalogError> {
        match self.map.get_field(key.field()?)? {
            Some(ValueRef::Int(raw)) => target(raw, self.count).map(Some),
            None => Ok(None),
            _ => Err(CatalogError::Field(key)),
        }
    }
    pub(super) fn validate(self) -> Result<(), CatalogError> {
        self.name()?;
        let kind = self.kind()?;
        self.arn()?;
        self.error()?;
        self.references()?;
        self.arn_reference()?;
        self.resource_operations()?
            .try_for_each(|value| value.map(|_| ()))?;
        self.resource_identifiers()?
            .try_for_each(|value| value.map(|_| ()))?;
        self.wire()?.validate()?;
        enum_values::validate(self.map, kind)?;
        value_constraints::validate(self.map)?;
        for key in [
            CatalogKey::ElementWire,
            CatalogKey::MapKeyWire,
            CatalogKey::MapValueWire,
        ] {
            value_constraints::validate_nested(self.map, key)?;
        }
        self.validate_members(kind)?;
        let element = self.element()?;
        let key = self.map_key()?;
        let value = self.map_value()?;
        self.element_wire()?.validate()?;
        self.map_key_wire()?.validate()?;
        self.map_value_wire()?.validate()?;
        if element.is_some() != (kind == "list")
            || key.is_some() != (kind == "map")
            || value.is_some() != (kind == "map")
        {
            return Err(CatalogError::Reference);
        }
        Ok(())
    }
    fn validate_members(self, kind: &str) -> Result<(), CatalogError> {
        self.members()?.try_for_each(|member| {
            let member = member?;
            member.name()?;
            member.wire()?.validate()?;
            member.arn_reference()?;
            member.references()?;
            value_constraints::validate(member.map)?;
            if !matches!(kind, "structure" | "union") {
                return Err(CatalogError::Field(CatalogKey::Members));
            }
            Ok(())
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Member<'a> {
    pub id: FieldId,
    pub target: Target,
    pub required: bool,
    map: MapRef<'a>,
    field_names: &'a [u8],
    field_offsets: &'a [u8],
}
impl<'a> Member<'a> {
    pub fn name(self) -> Result<&'a str, CatalogError> {
        super::field_name(self.field_names, self.field_offsets, self.id)
    }
    pub fn wire(self) -> Result<WireTraits<'a>, CatalogError> {
        Ok(WireTraits::member(self.map))
    }
    pub fn default_value(self) -> Result<Option<ValueRef<'a>>, CatalogError> {
        self.wire()?.default_value()
    }
    /// Borrows constraints attached to this member, independently of its target.
    pub fn value_constraints(self) -> Result<ValueConstraints<'a>, CatalogError> {
        value_constraints::read(self.map)
    }
    pub fn client_optional(self) -> Result<bool, CatalogError> {
        self.wire()?.client_optional()
    }
    /// Retained payload of this member's ARN reference declaration.
    pub fn arn_reference(self) -> Result<Option<&'a str>, CatalogError> {
        super::optional_text(self.map, CatalogKey::ArnReference)
    }

    /// Retained payload of this member's Smithy reference declarations.
    pub fn references(self) -> Result<Option<&'a str>, CatalogError> {
        super::optional_text(self.map, CatalogKey::References)
    }
}
fn read_member<'a>(
    value: Result<ValueRef<'a>, crate::record::RecordError>,
    count: usize,
    field_names: &'a [u8],
    field_offsets: &'a [u8],
) -> Result<Member<'a>, CatalogError> {
    let ValueRef::Map(map) = value? else {
        return Err(CatalogError::Reference);
    };
    let id = member_id(
        map.get_field(CatalogKey::MemberId.field()?)?
            .ok_or(CatalogError::Reference)?,
    )?;
    let target_value = map.get_field(CatalogKey::MemberTarget.field()?)?;
    let target_raw = number(target_value.ok_or(CatalogError::Reference)?)?;
    let target = target(target_raw, count)?;
    let required_value = map.get_field(CatalogKey::MemberRequired.field()?)?;
    let required = match required_value {
        Some(ValueRef::Bool(required)) => required,
        _ => return Err(CatalogError::Reference),
    };
    Ok(Member {
        id,
        target,
        required,
        map,
        field_names,
        field_offsets,
    })
}

fn member_id(value: ValueRef<'_>) -> Result<FieldId, CatalogError> {
    let raw = u32::try_from(number(value)?).map_err(|_| CatalogError::Reference)?;
    Ok(FieldId::new(raw)?)
}
