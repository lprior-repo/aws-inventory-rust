use super::{IdentityTextOperandValidation, path_value, SweepError};
use inventory_core::record::{FieldId, Record, ValueRef};
use std::{borrow::Cow, fmt::Write};

pub(super) fn integer<'a>(record: &'a Record, path: &[FieldId], minimum: i64, maximum: i64) -> Result<Cow<'a, str>, SweepError> {
    if minimum > maximum { return Err(SweepError::Operation("invalid identity integer bounds")) }
    let value = path_value(record.root(), path)?.ok_or(SweepError::Operation("modeled identity path is absent"))?;
    let ValueRef::Int(value) = value else { return Err(SweepError::Operation("modeled identity operand must be integer")) };
    if !(minimum..=maximum).contains(&value) { return Err(SweepError::Operation("modeled identity integer is out of range")) }
    Ok(Cow::Owned(value.to_string()))
}

pub(super) fn string_list<'a>(record: &'a Record, path: &[FieldId], max_items: u32, max_item_bytes: u32, element: &IdentityTextOperandValidation<'_, '_>) -> Result<Cow<'a, str>, SweepError> {
    if !(1..=64).contains(&max_items) || !(1..=4096).contains(&max_item_bytes) {
        return Err(SweepError::Operation("invalid identity string list bounds"))
    }
    let value = path_value(record.root(), path)?.ok_or(SweepError::Operation("modeled identity path is absent"))?;
    let ValueRef::List(list) = value else { return Err(SweepError::Operation("modeled identity operand must be string list")) };
    let count = list.count()?;
    if u32::try_from(count).map_or(true, |count| count > max_items) {
        return Err(SweepError::Operation("modeled identity string list count is out of range"))
    }
    let mut text = String::new();
    write!(text, "list:{count}").map_err(|_| SweepError::Operation("identity formatting failed"))?;
    list.iter()?.take(65).try_for_each(|value| {
        let ValueRef::Text(component) = value? else { return Err(SweepError::Operation("modeled identity list member must be text")) };
        if component.is_empty() || component.chars().any(char::is_control)
            || u32::try_from(component.len()).map_or(true, |length| length > max_item_bytes) {
            return Err(SweepError::Operation("modeled identity list member is empty, malformed, or too long"))
        }
        element.accepts(component)?;
        append(&mut text, component)
    })?;
    Ok(Cow::Owned(text))
}

fn append(text: &mut String, component: &str) -> Result<(), SweepError> {
    let digits = usize::try_from(component.len().ilog10().saturating_add(1))
        .map_err(|_| SweepError::Operation("identity length overflow"))?;
    let additional = component.len().checked_add(digits).and_then(|length| length.checked_add(2))
        .ok_or(SweepError::Operation("identity length overflow"))?;
    if text.len().checked_add(additional).is_none_or(|length| length > 4096) {
        return Err(SweepError::Operation("modeled resource identity is too long"))
    }
    text.try_reserve(additional)?;
    write!(text, "|{}:{component}", component.len()).map_err(|_| SweepError::Operation("identity formatting failed"))
}
