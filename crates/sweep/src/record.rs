mod composite;
use composite::key_text;
#[cfg(test)]
mod contact_exclusion_ac_tests;
#[cfg(test)]
mod contact_exclusion_df_tests;
#[cfg(test)]
mod contact_exclusion_lp_tests;
#[cfg(test)]
mod contact_exclusion_support;
mod identity_choice;
#[cfg(test)]
mod identity_tests;
mod identity_validation;
#[cfg(test)]
mod organizations_email_tests;
#[cfg(test)]
mod policy_ac_tests;
#[cfg(test)]
pub(crate) mod policy_dfgk_tests;
#[cfg(test)]
mod policy_gk_gamelift_fleets_tests;
#[cfg(test)]
mod policy_gk_iot_iam_tests;
#[cfg(test)]
mod policy_gk_ivs_roots_tests;
#[cfg(test)]
mod policy_gk_root_tests;
#[cfg(test)]
pub(crate) mod policy_lp_tests;
#[cfg(test)]
mod policy_qstz_tests;
#[cfg(test)]
mod scalar_tests;
mod selection;
#[cfg(test)]
mod selection_tests;
mod tags;
pub(crate) mod trusted_arn;
pub(crate) use identity_choice::foreign_owned;
pub(super) use identity_choice::is_identity_member;
pub(super) use identity_validation::IdentityTextValidation;
pub(super) use tags::tag_observation;
#[cfg(test)]
#[path = "../../../tests/sweep_record_exact_bounds.rs"]
mod exact_bounds;

#[cfg(test)]
#[test]
fn compiled_page_preserves_exactly_100000_scalar_rows() -> Result<(), Box<dyn std::error::Error>> {
    exact_bounds::accepts_exact_page(item_records)
}

#[cfg(test)]
#[test]
fn compiled_page_rejects_100001_scalar_rows_atomically() -> Result<(), Box<dyn std::error::Error>> {
    exact_bounds::rejects_oversized_page(item_records)
}

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
    let root = match root.get_name("$value")? {
        Some(value) if path.is_empty() => return Ok(Some(value)),
        Some(ValueRef::Map(selected)) => selected,
        Some(_) => return Ok(None),
        None => root,
    };
    let Some(leaf) = path.last().copied() else {
        return scalar_item(root).map_err(SweepError::from);
    };
    let parent = path_parent(root, path)?;
    parent.map_or(Ok(None), |map| {
        map.get_field(leaf).map_err(SweepError::from)
    })
}

fn path_parent<'a>(root: MapRef<'a>, path: &[FieldId]) -> Result<Option<MapRef<'a>>, SweepError> {
    path.iter().take(path.len().saturating_sub(1)).try_fold(
        Some(root),
        |current, id| -> Result<Option<MapRef<'a>>, SweepError> {
            current.map_or(Ok(None), |map| match map.get_field(*id)? {
                Some(ValueRef::Map(child)) => Ok(Some(child)),
                _ => Ok(None),
            })
        },
    )
}

pub(super) fn item_records(record: &Record, plan: &ItemPlan) -> Result<Vec<Record>, SweepError> {
    let (count, selected) = item_selection(record, plan)?;
    let mut records = Vec::new();
    records.try_reserve(count)?;
    for item in selected {
        records.push(item?);
    }
    Ok(records)
}

pub(super) fn item_selection<'a>(
    record: &'a Record,
    plan: &ItemPlan,
) -> Result<
    (
        usize,
        impl Iterator<Item = Result<Record, SweepError>> + 'a + use<'a>,
    ),
    SweepError,
> {
    selection::items(record, plan)
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

#[cfg(test)]
pub(super) fn resource_id(
    record: &Record,
    plan: &IdentityPlan,
    context: Option<IdentityContext<'_>>,
) -> Result<ResourceId, SweepError> {
    identity(
        record,
        plan,
        context.as_ref(),
        None,
        None,
        &IdentityTextValidation::RecipeOnly,
    )
}

#[cfg(test)]
pub(super) fn resource_id_with_context(
    record: &Record,
    plan: &IdentityPlan,
    trusted: Option<IdentityContext<'_>>,
    discovery: &inventory_core::inventory::DiscoveryContext,
    current_request: &Record,
) -> Result<ResourceId, SweepError> {
    resource_id_validated(
        record,
        plan,
        trusted,
        discovery,
        current_request,
        &IdentityTextValidation::RecipeOnly,
    )
}

pub(super) fn resource_id_validated(
    record: &Record,
    plan: &IdentityPlan,
    trusted: Option<IdentityContext<'_>>,
    discovery: &inventory_core::inventory::DiscoveryContext,
    current_request: &Record,
    validation: &IdentityTextValidation<'_, '_>,
) -> Result<ResourceId, SweepError> {
    identity(
        record,
        plan,
        trusted.as_ref(),
        Some(discovery),
        Some(current_request),
        validation,
    )
}

fn identity(
    record: &Record,
    plan: &IdentityPlan,
    context: Option<&IdentityContext<'_>>,
    discovery: Option<&inventory_core::inventory::DiscoveryContext>,
    current_request: Option<&Record>,
    validation: &IdentityTextValidation<'_, '_>,
) -> Result<ResourceId, SweepError> {
    let text = match plan {
        IdentityPlan::Arn(path) => validated_arn(record, path, context)?,
        IdentityPlan::Template(parts) => Cow::Owned(template_text(record, parts, context)?),
        IdentityPlan::Key(paths) => key_text(record, paths)?,
        IdentityPlan::Choice(_) | IdentityPlan::CorrelatedKey(_) | IdentityPlan::MapArn { .. } => {
            return identity_choice::resource_id(
                record,
                plan,
                context,
                discovery,
                current_request,
                validation,
            );
        }
    };
    identity_choice::parse_identity(&text)
}

fn validated_arn<'a>(
    record: &'a Record,
    path: &[FieldId],
    context: Option<&IdentityContext<'_>>,
) -> Result<Cow<'a, str>, SweepError> {
    let text = required_text(record, path)?;
    trusted_arn::validate_arn(&text, context)?;
    Ok(text)
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
            return Err(SweepError::Operation(
                "modeled resource identity is too long",
            ));
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
        return Err(SweepError::Operation(
            "identity context requires the commercial partition",
        ));
    }
    match part {
        IdentityPart::Partition => Ok("aws"),
        IdentityPart::Account => Ok(context.account.as_str()),
        IdentityPart::Region => Ok(region),
        _ => Err(SweepError::Operation("invalid identity context part")),
    }
}

fn commercial_region(region: &str) -> bool {
    let Some((area, rest)) = region.split_once('-') else {
        return false;
    };
    let Some((direction, _)) = rest.rsplit_once('-') else {
        return false;
    };
    matches!(
        area,
        "af" | "ap" | "ca" | "eu" | "il" | "me" | "mx" | "sa" | "us"
    ) && matches!(
        direction,
        "central"
            | "east"
            | "west"
            | "north"
            | "northeast"
            | "northwest"
            | "south"
            | "southeast"
            | "southwest"
    )
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
