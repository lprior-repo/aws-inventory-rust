#[cfg(test)]
mod scalar_tests;
#[cfg(test)]
mod identity_tests;

const MAX_PAGE_ITEMS: usize = 100_000;
use std::{borrow::Cow, fmt::Write};

use inventory_core::{
    catalog::runtime::{IdentityPart, IdentityPlan, ItemPlan},
    inventory::{AccountId, RegionId, ResourceId},
    record::{FieldId, MapRef, Record, ValueRef, scalar_item, scalar_record},
};

use super::SweepError;

pub(super) fn path_value<'a>(
    root: MapRef<'a>,
    path: &[FieldId],
) -> Result<Option<ValueRef<'a>>, SweepError> {
    let Some(leaf) = path.last().copied() else {
        return scalar_item(root).map_err(SweepError::from);
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
        Some(value @ ValueRef::Map(_)) => Ok(vec![item_record(value)?]),
        Some(ValueRef::List(list)) => list.iter()?.try_fold(Vec::new(), |mut records, item| {
            if records.len() >= MAX_PAGE_ITEMS {
                return Err(SweepError::Operation(
                    "page exceeds the atomic observation bound",
                ));
            }
            records.try_reserve(1)?;
            records.push(item_record(item?)?);
            Ok(records)
        }),
        _ => Err(SweepError::Operation(
            "compiled enumeration item path is neither a list nor a structure",
        )),
    }
}

fn item_record(value: ValueRef<'_>) -> Result<Record, SweepError> {
    match value {
        ValueRef::Map(_) => Ok(Record::encode(&value.to_value()?)?),
        ValueRef::List(_) => Err(SweepError::Operation(
            "enumeration item is neither a structure nor a scalar",
        )),
        scalar => Ok(scalar_record(scalar)?),
    }
}

pub(super) struct IdentityContext<'a> {
    pub account: &'a AccountId,
    pub region: &'a RegionId,
}

pub(super) fn resource_id(
    record: &Record,
    plan: &IdentityPlan,
    context: Option<IdentityContext<'_>>,
) -> Result<ResourceId, SweepError> {
    let text = match plan {
        IdentityPlan::Arn(path) => required_text(record, path)?,
        IdentityPlan::Template(parts) => Cow::Owned(template_text(record, parts, context.as_ref())?),
        IdentityPlan::Key(paths) => key_text(record, paths)?,
    };
    ResourceId::parse(&text)
        .map_err(|_| SweepError::Operation("modeled resource identity is invalid"))
}
fn required_text<'a>(record: &'a Record, path: &[FieldId]) -> Result<Cow<'a, str>, SweepError> {
    path_value(record.root(), path)?
        .map(value_text)
        .transpose()?
        .ok_or(SweepError::Operation("modeled identity path is absent"))
}

fn template_text(
    record: &Record,
    parts: &[IdentityPart],
    context: Option<&IdentityContext<'_>>,
) -> Result<String, SweepError> {
    parts.iter().try_fold(String::new(), |mut text, part| {
        let value = match part {
            IdentityPart::Literal(value) => Cow::Borrowed(value.as_str()),
            IdentityPart::Field(path) => required_text(record, path)?,
            _ => Cow::Borrowed(context_text(context, part)?),
        };
        if text.len().saturating_add(value.len()) > 4096 {
            return Err(SweepError::Operation("modeled resource identity is too long"));
        }
        text.try_reserve(value.len())?;
        text.push_str(&value);
        Ok(text)
    })
}

fn context_text<'a>(
    context: Option<&IdentityContext<'a>>,
    part: &IdentityPart,
) -> Result<&'a str, SweepError> {
    let context = context.ok_or(SweepError::Operation("identity request context is absent"))?;
    let region = context.region.as_str();
    if !commercial_region(region) {
        return Err(SweepError::Operation("identity context requires the commercial partition"));
    }
    match part {
        IdentityPart::Partition => Ok("aws"),
        IdentityPart::Account => Ok(context.account.as_str()),
        IdentityPart::Region => Ok(region),
        _ => Err(SweepError::Operation("invalid identity context part")),
    }
}

fn commercial_region(region: &str) -> bool {
    let Some((area, rest)) = region.split_once('-') else { return false };
    let Some((direction, _)) = rest.rsplit_once('-') else { return false };
    matches!(area, "af" | "ap" | "ca" | "eu" | "il" | "me" | "mx" | "sa" | "us")
        && matches!(direction, "central" | "east" | "west" | "north" | "northeast"
            | "northwest" | "south" | "southeast" | "southwest")
}

fn key_text<'a>(record: &'a Record, paths: &[Box<[FieldId]>]) -> Result<Cow<'a, str>, SweepError> {
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
        .map(Cow::Owned)
}

fn value_text(value: ValueRef<'_>) -> Result<Cow<'_, str>, SweepError> {
    match value {
        ValueRef::Text(text) => Ok(Cow::Borrowed(text)),
        ValueRef::Int(value) => Ok(Cow::Owned(value.to_string())),
        ValueRef::Bool(value) => Ok(Cow::Borrowed(if value { "true" } else { "false" })),
        _ => Err(SweepError::Operation(
            "modeled identity must be text, integer, or boolean",
        )),
    }
}
