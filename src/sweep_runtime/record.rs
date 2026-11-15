const MAX_PAGE_ITEMS: usize = 1_000;
use std::fmt::Write;

use inventory_core::{
    catalog::runtime::{IdentityPart, IdentityPlan, ItemPlan},
    inventory::ResourceId,
    record::{FieldId, MapRef, Record, Value, ValueRef},
};

use super::SweepError;

pub(super) fn path_value<'a>(
    root: MapRef<'a>,
    path: &[FieldId],
) -> Result<Option<ValueRef<'a>>, SweepError> {
    let Some(leaf) = path.last().copied() else {
        return Ok(None);
    };
    let parent = path.iter().take(path.len().saturating_sub(1)).try_fold(
        Some(root),
        |current, id| -> Result<Option<MapRef<'a>>, SweepError> {
            current.map_or(Ok(None), |map| match map.get_field(*id)? {
                Some(ValueRef::Map(child)) => Ok(Some(child)),
                _ => Ok(None),
            })
        },
    )?;
    parent.map_or(Ok(None), |map| {
        map.get_field(leaf).map_err(SweepError::from)
    })
}

pub(super) fn item_records(record: &Record, plan: &ItemPlan) -> Result<Vec<Record>, SweepError> {
    match plan {
        ItemPlan::Root => Ok(vec![record.clone()]),
        ItemPlan::Path(path) => item_records_at(record, path),
    }
}

fn item_records_at(record: &Record, path: &[FieldId]) -> Result<Vec<Record>, SweepError> {
    match path_value(record.root(), path)? {
        None | Some(ValueRef::Null) => Ok(Vec::new()),
        Some(ValueRef::List(list)) => list.iter()?.try_fold(Vec::new(), |mut records, item| {
            if records.len() >= MAX_PAGE_ITEMS {
                return Err(SweepError::Operation(
                    "page exceeds the atomic observation bound",
                ));
            }
            let value = item?.to_value()?;
            let Value::Map(_) = value else {
                return Err(SweepError::Operation(
                    "enumeration item is not a native record map",
                ));
            };
            records.try_reserve(1)?;
            records.push(Record::encode(&value)?);
            Ok(records)
        }),
        _ => Err(SweepError::Operation(
            "compiled enumeration item path is not a list",
        )),
    }
}

pub(super) fn resource_id(record: &Record, plan: &IdentityPlan) -> Result<ResourceId, SweepError> {
    let text = match plan {
        IdentityPlan::Arn(path) => required_text(record, path)?,
        IdentityPlan::Template(parts) => template_text(record, parts)?,
        IdentityPlan::Key(paths) => key_text(record, paths)?,
    };
    ResourceId::parse(&text)
        .map_err(|_| SweepError::Operation("modeled resource identity is invalid"))
}
fn required_text(record: &Record, path: &[FieldId]) -> Result<String, SweepError> {
    path_value(record.root(), path)?
        .map(value_text)
        .transpose()?
        .ok_or(SweepError::Operation("modeled identity path is absent"))
}

fn template_text(record: &Record, parts: &[IdentityPart]) -> Result<String, SweepError> {
    parts.iter().try_fold(String::new(), |mut text, part| {
        match part {
            IdentityPart::Literal(value) => text.push_str(value),
            IdentityPart::Field(path) => text.push_str(&required_text(record, path)?),
        }
        Ok(text)
    })
}

fn key_text(record: &Record, paths: &[Box<[FieldId]>]) -> Result<String, SweepError> {
    match paths {
        [] => {
            return Err(SweepError::Operation(
                "modeled composite identity has no fields",
            ));
        }
        [single] => return required_text(record, single),
        _ => {}
    }
    paths
        .iter()
        .enumerate()
        .try_fold(String::new(), |mut text, (position, path)| {
            let component = required_text(record, path)?;
            if position > 0 {
                text.push('|');
            }
            write!(text, "{}:{component}", component.len())
                .map_err(|_| SweepError::Operation("identity formatting failed"))?;
            Ok(text)
        })
}

fn value_text(value: ValueRef<'_>) -> Result<String, SweepError> {
    match value {
        ValueRef::Text(text) => Ok(text.to_owned()),
        ValueRef::Int(value) => Ok(value.to_string()),
        ValueRef::Bool(value) => Ok(value.to_string()),
        _ => Err(SweepError::Operation(
            "modeled identity must be text, integer, or boolean",
        )),
    }
}
