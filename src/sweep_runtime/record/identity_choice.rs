#[path = "typed_operand.rs"]
mod typed_operand;
use super::identity_validation::{IdentityTextOperandValidation, IdentityTextValidation};
use super::{IdentityContext, SweepError, key_text, path_value, required_text, template_text, value_text};
use super::trusted_arn::{validate_arn, validate_resource_arn};
use inventory_core::{catalog::runtime::{IdentityChoicePlan, IdentityFieldPlan, IdentityLeafPlan, IdentityPlan, IdentityPredicatePlan}, inventory::{DiscoveryContext, ResourceId}, record::{Record, ValueRef}};
use std::{borrow::Cow, fmt::Write};

pub(super) fn resource_id(
    row: &Record,
    identity: &IdentityPlan,
    trusted: Option<&IdentityContext<'_>>,
    context: Option<&DiscoveryContext>,
    request: Option<&Record>,
    validation: &IdentityTextValidation<'_, '_>,
) -> Result<ResourceId, SweepError> {
    let value = match identity {
        IdentityPlan::Choice(choice) => choose(row, choice, trusted, context, request, validation)?,
        IdentityPlan::CorrelatedKey(fields) => fields_text(row, fields, trusted, context, request, validation)?,
        IdentityPlan::MapArn { path, key, resource_kind, request_url } => {
            let text = map_text(row, path, key)?;
            let request_url = request_url.as_ref().map(|path| {
                required_text(request.ok_or(SweepError::Operation("queue ARN identity requires actual request URL"))?, path)
            }).transpose()?;
            validate_resource_arn(&text, resource_kind,
                trusted.ok_or(SweepError::Operation("map ARN identity requires trusted request scope"))?, request_url.as_deref())?;
            validation.selected(path, &text)?;
            text
        }
        _ => return Err(SweepError::Operation("unexpected contextual identity recipe")),
    };
    ResourceId::parse(&value).map_err(|_| SweepError::Operation("modeled resource identity is invalid"))
}

fn choose<'a>(row: &'a Record, choice: &IdentityChoicePlan, trusted: Option<&IdentityContext<'_>>, context: Option<&'a DiscoveryContext>, request: Option<&'a Record>, validation: &IdentityTextValidation<'_, '_>) -> Result<Cow<'a, str>, SweepError> {
    match choice {
        IdentityChoicePlan::PreferArn { path, fallback, .. } => {
            match path_value(row.root(), path)? {
                None => leaf(row, fallback, trusted, context, request, validation),
                Some(value) => {
                    let text = value_text(value)?;
                    validate_arn(&text, trusted)?;
                    validation.selected(path, &text)?;
                    Ok(text)
                }
            }
        }
        IdentityChoicePlan::Discriminated(branches) => {
            let branch = branches.iter().try_fold(None, |found, branch| {
                if found.is_some() { return Ok(found) }
                Ok::<_, SweepError>(matches_predicate(row, &branch.predicate, request)?.then_some(branch))
            })?.ok_or(SweepError::Operation("no reviewed identity branch matches the selected item"))?;
            leaf(row, &branch.identity, trusted, context, request, validation)
        }
    }
}

fn matches_predicate(row: &Record, predicate: &IdentityPredicatePlan, request: Option<&Record>) -> Result<bool, SweepError> {
    let (record, path, expected) = match predicate {
        IdentityPredicatePlan::Present(path) => (row, path, None),
        IdentityPredicatePlan::TextEquals { path, value } => (row, path, Some(value)),
        IdentityPredicatePlan::RequestTextEquals { path, value } => (request.ok_or(SweepError::Operation("identity requires the actual request context"))?, path, Some(value)),
    };
    match (path_value(record.root(), path)?, expected) {
        (None | Some(ValueRef::Null), _) => Ok(false),
        (Some(_), None) => Ok(true),
        (Some(ValueRef::Text(actual)), Some(expected)) => Ok(actual == expected),
        _ => Err(SweepError::Operation("identity discriminant is not modeled text")),
    }
}

fn leaf<'a>(row: &'a Record, leaf: &IdentityLeafPlan, trusted: Option<&IdentityContext<'_>>, context: Option<&'a DiscoveryContext>, request: Option<&'a Record>, validation: &IdentityTextValidation<'_, '_>) -> Result<Cow<'a, str>, SweepError> {
    match leaf {
        IdentityLeafPlan::Arn(path) => {
            let text = required_text(row, path)?;
            validate_arn(&text, trusted)?;
            validation.selected(path, &text)?;
            Ok(text)
        }
        IdentityLeafPlan::Template(parts) => Ok(Cow::Owned(template_text(row, parts, trusted)?)),
        IdentityLeafPlan::Key(paths) => key_text(row, paths),
        IdentityLeafPlan::Fields(fields) => fields_text(row, fields, trusted, context, request, validation),
    }
}

fn fields_text<'a>(row: &'a Record, fields: &[IdentityFieldPlan], trusted: Option<&IdentityContext<'_>>, context: Option<&'a DiscoveryContext>, request: Option<&'a Record>, validation: &IdentityTextValidation<'_, '_>) -> Result<Cow<'a, str>, SweepError> {
    if fields.is_empty() || fields.len() > 64 { return Err(SweepError::Operation("invalid correlated identity field count")) }
    if let [field] = fields { return field_text(row, field, trusted, context, request, validation) }
    fields.iter().enumerate().try_fold(String::new(), |mut result, (position, field)| {
        let component = field_text(row, field, trusted, context, request, validation)?;
        let digits = if component.is_empty() { 1 } else { component.len().ilog10().saturating_add(1) };
        let additional = component.len().checked_add(usize::try_from(digits).map_err(|_| SweepError::Operation("identity length overflow"))?)
            .and_then(|length| length.checked_add(1usize.saturating_add(usize::from(position > 0)))).ok_or(SweepError::Operation("identity length overflow"))?;
        if result.len().checked_add(additional).is_none_or(|length| length > 4096) { return Err(SweepError::Operation("modeled resource identity is too long")) }
        result.try_reserve(additional)?;
        if position > 0 { result.push('|'); }
        write!(result, "{}:{component}", component.len()).map_err(|_| SweepError::Operation("identity formatting failed"))?;
        Ok(result)
    }).map(Cow::Owned)
}

fn field_text<'a>(row: &'a Record, field: &IdentityFieldPlan, trusted: Option<&IdentityContext<'_>>, context: Option<&'a DiscoveryContext>, request: Option<&'a Record>, validation: &IdentityTextValidation<'_, '_>) -> Result<Cow<'a, str>, SweepError> {
    match field {
        IdentityFieldPlan::Selected(path) => identity_text(row, path),
        IdentityFieldPlan::Request(path) => identity_text(request.ok_or(SweepError::Operation("identity requires actual request context"))?, path),
        IdentityFieldPlan::SelectedArn(path) | IdentityFieldPlan::RequestArn(path) => {
            let record = if matches!(field, IdentityFieldPlan::SelectedArn(_)) { row } else {
                request.ok_or(SweepError::Operation("identity requires actual request context"))?
            };
            let value = path_value(record.root(), path)?.ok_or(SweepError::Operation("modeled identity path is absent"))?;
            let ValueRef::Text(text) = value else { return Err(SweepError::Operation("modeled ARN identity operand must be text")) };
            validate_arn(text, trusted)?;
            if matches!(field, IdentityFieldPlan::SelectedArn(_)) { validation.selected(path, text)?; } else { validation.request(path, text)?; }
            Ok(Cow::Borrowed(text))
        }
        IdentityFieldPlan::SelectedInteger { path, minimum, maximum } =>
            typed_operand::integer(row, path, *minimum, *maximum),
        IdentityFieldPlan::RequestInteger { path, minimum, maximum } =>
            typed_operand::integer(request.ok_or(SweepError::Operation("identity requires actual request context"))?, path, *minimum, *maximum),
        IdentityFieldPlan::SelectedStringList { path, max_items, max_item_bytes } =>
            typed_operand::string_list(row, path, *max_items, *max_item_bytes, &validation.selected_operand(path)?),
        IdentityFieldPlan::SelectedDigest(path) => {
            let value = path_value(row.root(), path)?.ok_or(SweepError::Operation("modeled identity path is absent"))?;
            let empty = match value { ValueRef::Null => true, ValueRef::Map(map) => map.count()? == 0, ValueRef::List(list) => list.count()? == 0, _ => false };
            if empty { return Err(SweepError::Operation("modeled identity digest value is null or empty")) }
            let hash = inventory_core::record::selected_content_hash(value)?;
            let text = hash.as_bytes().iter().try_fold(String::from("blake3:"), |mut text, byte| {
                write!(text, "{byte:02x}").map_err(|_| SweepError::Operation("identity digest formatting failed"))?;
                Ok::<_, SweepError>(text)
            })?;
            Ok(Cow::Owned(text))
        }
        IdentityFieldPlan::MapValue { path, key } => map_text(row, path, key),
        IdentityFieldPlan::Context { service, operation, source, path } => {
            let context = context.ok_or(SweepError::Operation("identity requires durable discovery context"))?;
            context.value(service, operation, *source, path).map_err(|_| SweepError::Operation("invalid durable discovery context"))?
                .ok_or(SweepError::Operation("required discovery ancestor identity is absent")).and_then(value_text)
        }
    }
}

fn identity_text<'a>(record: &'a Record, path: &[inventory_core::record::FieldId]) -> Result<Cow<'a, str>, SweepError> {
    let text = required_text(record, path)?;
    if text.is_empty() { return Err(SweepError::Operation("modeled identity text operand is empty")) }
    Ok(text)
}

fn map_text<'a>(row: &'a Record, path: &[inventory_core::record::FieldId], key: &str) -> Result<Cow<'a, str>, SweepError> {
    let Some(ValueRef::Map(map)) = path_value(row.root(), path)? else { return Err(SweepError::Operation("modeled identity map is absent")) };
    let Some(ValueRef::Text(text)) = map.get_name(key)? else { return Err(SweepError::Operation("modeled identity map entry is absent or not text")) };
    Ok(Cow::Borrowed(text))
}

