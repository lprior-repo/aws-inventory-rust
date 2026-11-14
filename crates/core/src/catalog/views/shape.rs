//! Shape and member views; targets are indexes or negative prelude kind codes.
use super::super::{CatalogError, CatalogKey, SHAPE_KINDS, integer, list, number, text};
use crate::record::{FieldId, MapRef, ValueRef};

/// A local zero-based index or a scalar/prelude kind name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Target {
    /// Index into the same service's shape list.
    Shape(usize),
    /// Smithy prelude kind, including Unit.
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

/// One borrowed Smithy shape.
#[derive(Clone, Copy, Debug)]
pub struct Shape<'a> {
    map: MapRef<'a>,
    count: usize,
}
impl<'a> Shape<'a> {
    pub(super) fn new(map: MapRef<'a>, count: usize) -> Result<Self, CatalogError> {
        let shape = Self { map, count };
        shape.name()?;
        shape.kind()?;
        Ok(shape)
    }
    /// Unqualified shape name.
    /// # Errors
    /// Reports malformed names.
    pub fn name(self) -> Result<&'a str, CatalogError> {
        text(self.map, CatalogKey::Name)
    }
    /// Smithy kind name from the fixed code vocabulary.
    /// # Errors
    /// Rejects unknown kind codes.
    pub fn kind(self) -> Result<&'static str, CatalogError> {
        let code = usize::try_from(integer(self.map, CatalogKey::Kind)?)
            .map_err(|_| CatalogError::Reference)?;
        SHAPE_KINDS
            .get(code)
            .copied()
            .ok_or(CatalogError::Reference)
    }
    /// Structure/union members, strictly ascending by registry field ID.
    /// # Errors
    /// Reports invalid member triples or order.
    pub fn members(
        self,
    ) -> Result<impl Iterator<Item = Result<Member, CatalogError>> + 'a, CatalogError> {
        let members = list(self.map, CatalogKey::Members)?.iter()?;
        Ok(members.scan(None, move |previous, value| {
            Some(read_member(value, self.count).and_then(|member| {
                if previous.is_some_and(|id| id >= member.id) {
                    return Err(CatalogError::Reference);
                }
                *previous = Some(member.id);
                Ok(member)
            }))
        }))
    }
    /// List element target, present only on lists.
    /// # Errors
    /// Reports invalid target fields.
    pub fn element(self) -> Result<Option<Target>, CatalogError> {
        self.reference(CatalogKey::Element)
    }
    /// Map key target, present only on maps.
    /// # Errors
    /// Reports invalid target fields.
    pub fn map_key(self) -> Result<Option<Target>, CatalogError> {
        self.reference(CatalogKey::MapKey)
    }
    /// Map value target, present only on maps.
    /// # Errors
    /// Reports invalid target fields.
    pub fn map_value(self) -> Result<Option<Target>, CatalogError> {
        self.reference(CatalogKey::MapValue)
    }
    /// ARN template text, when modeled.
    /// # Errors
    /// Reports mistyped templates.
    pub fn arn(self) -> Result<Option<&'a str>, CatalogError> {
        match self.map.get_field(CatalogKey::Arn.field()?)? {
            Some(ValueRef::Text(text)) => Ok(Some(text)),
            Some(ValueRef::Null) => Ok(None),
            _ => Err(CatalogError::Field(CatalogKey::Arn)),
        }
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
        self.validate_members(kind)?;
        let element = self.element()?;
        let key = self.map_key()?;
        let value = self.map_value()?;
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
            member?;
            if !matches!(kind, "structure" | "union") {
                return Err(CatalogError::Field(CatalogKey::Members));
            }
            Ok(())
        })
    }
}

/// A stable member identity, bounded target, and modeled requiredness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Member {
    /// Stable identity from the append-only field registry.
    pub id: FieldId,
    /// Target shape.
    pub target: Target,
    /// Whether the required trait is present.
    pub required: bool,
}
fn read_member(
    value: Result<ValueRef<'_>, crate::record::RecordError>,
    count: usize,
) -> Result<Member, CatalogError> {
    let ValueRef::List(list) = value? else {
        return Err(CatalogError::Reference);
    };
    let mut items = list.iter()?;
    let id = member_id(next(&mut items)?)?;
    let target = target(number(next(&mut items)?)?, count)?;
    let required = match next(&mut items)? {
        ValueRef::Bool(required) => required,
        _ => return Err(CatalogError::Reference),
    };
    if items.next().is_some() {
        return Err(CatalogError::Reference);
    }
    Ok(Member {
        id,
        target,
        required,
    })
}

fn member_id(value: ValueRef<'_>) -> Result<FieldId, CatalogError> {
    let raw = u32::try_from(number(value)?).map_err(|_| CatalogError::Reference)?;
    Ok(FieldId::new(raw)?)
}

fn next<'a>(
    items: &mut impl Iterator<Item = Result<ValueRef<'a>, crate::record::RecordError>>,
) -> Result<ValueRef<'a>, CatalogError> {
    Ok(items.next().ok_or(CatalogError::Reference)??)
}
