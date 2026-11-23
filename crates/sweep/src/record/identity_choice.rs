mod fields;
#[path = "typed_operand.rs"]
mod typed_operand;
use super::identity_validation::{IdentityTextOperandValidation, IdentityTextValidation};
use super::trusted_arn::{validate_arn, validate_resource_arn};
use super::{
    IdentityContext, SweepError, key_text, path_value, required_text, template_text, value_text,
};
use inventory_core::{
    catalog::runtime::{
        IdentityChoicePlan, IdentityFieldPlan, IdentityLeafPlan, IdentityPlan,
        IdentityPredicatePlan,
    },
    inventory::{DiscoveryContext, ResourceId},
    record::{Record, ValueRef},
};
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
        IdentityPlan::CorrelatedKey(fields) => {
            fields_text(row, fields, trusted, context, request, validation)?
        }
        _ => map_identity(row, identity, trusted, request)?,
    };
    parse_identity(&value)
}

pub(super) fn parse_identity(text: &str) -> Result<ResourceId, SweepError> {
    ResourceId::parse(text)
        .map_err(|_| SweepError::Operation("modeled resource identity is invalid"))
}

fn choose<'a>(
    row: &'a Record,
    choice: &IdentityChoicePlan,
    trusted: Option<&IdentityContext<'_>>,
    context: Option<&'a DiscoveryContext>,
    request: Option<&'a Record>,
    validation: &IdentityTextValidation<'_, '_>,
) -> Result<Cow<'a, str>, SweepError> {
    match choice {
        IdentityChoicePlan::PreferArn { path, fallback, .. } => {
            match path_value(row.root(), path)? {
                None => leaf(row, fallback, trusted, context, request, validation),
                Some(value) => {
                    let text = value_text(value)?;
                    validate_arn(&text, trusted)?;
                    Ok(text)
                }
            }
        }
        IdentityChoicePlan::Discriminated(branches) => {
            let branch = matching_branch(row, branches, request)?;
            leaf(row, &branch.identity, trusted, context, request, validation)
        }
    }
}

fn matches_predicate(
    row: &Record,
    predicate: &IdentityPredicatePlan,
    request: Option<&Record>,
) -> Result<bool, SweepError> {
    let (record, path, expected) = match predicate {
        IdentityPredicatePlan::Present(path) => (row, path, None),
        IdentityPredicatePlan::TextEquals { path, value } => (row, path, Some(value)),
        IdentityPredicatePlan::RequestTextEquals { path, value } => (
            request.ok_or(SweepError::Operation(
                "identity requires the actual request context",
            ))?,
            path,
            Some(value),
        ),
    };
    match (path_value(record.root(), path)?, expected) {
        (None | Some(ValueRef::Null), _) => Ok(false),
        (Some(_), None) => Ok(true),
        (Some(ValueRef::Text(actual)), Some(expected)) => Ok(actual == expected),
        _ => Err(SweepError::Operation(
            "identity discriminant is not modeled text",
        )),
    }
}

fn leaf<'a>(
    row: &'a Record,
    leaf: &IdentityLeafPlan,
    trusted: Option<&IdentityContext<'_>>,
    context: Option<&'a DiscoveryContext>,
    request: Option<&'a Record>,
    validation: &IdentityTextValidation<'_, '_>,
) -> Result<Cow<'a, str>, SweepError> {
    match leaf {
        IdentityLeafPlan::Arn(path) => {
            let text = required_text(row, path)?;
            validate_arn(&text, trusted)?;
            Ok(text)
        }
        IdentityLeafPlan::Template(parts) => Ok(Cow::Owned(template_text(row, parts, trusted)?)),
        IdentityLeafPlan::Key(paths) => key_text(row, paths),
        IdentityLeafPlan::Fields(fields) => {
            fields_text(row, fields, trusted, context, request, validation)
        }
    }
}

fn fields_text<'a>(
    row: &'a Record,
    fields: &[IdentityFieldPlan],
    trusted: Option<&IdentityContext<'_>>,
    context: Option<&'a DiscoveryContext>,
    request: Option<&'a Record>,
    validation: &IdentityTextValidation<'_, '_>,
) -> Result<Cow<'a, str>, SweepError> {
    fields::Fields {
        row,
        trusted,
        context,
        request,
        validation,
    }
    .texts(fields)
}

fn identity_text<'a>(
    record: &'a Record,
    path: &[inventory_core::record::FieldId],
) -> Result<Cow<'a, str>, SweepError> {
    let text = required_text(record, path)?;
    if text.is_empty() {
        return Err(SweepError::Operation(
            "modeled identity text operand is empty",
        ));
    }
    Ok(text)
}

fn map_text<'a>(
    row: &'a Record,
    path: &[inventory_core::record::FieldId],
    key: &str,
) -> Result<Cow<'a, str>, SweepError> {
    let Some(ValueRef::Map(map)) = path_value(row.root(), path)? else {
        return Err(SweepError::Operation("modeled identity map is absent"));
    };
    let Some(ValueRef::Text(text)) = map.get_name(key)? else {
        return Err(SweepError::Operation(
            "modeled identity map entry is absent or not text",
        ));
    };
    Ok(Cow::Borrowed(text))
}

fn matching_branch<'a>(
    row: &Record,
    branches: &'a [inventory_core::catalog::runtime::IdentityBranchPlan],
    request: Option<&Record>,
) -> Result<&'a inventory_core::catalog::runtime::IdentityBranchPlan, SweepError> {
    branches
        .iter()
        .try_fold(None, |found, branch| {
            if found.is_some() {
                return Ok(found);
            }
            Ok::<_, SweepError>(
                matches_predicate(row, &branch.predicate, request)?.then_some(branch),
            )
        })?
        .ok_or(SweepError::Operation(
            "no reviewed identity branch matches the selected item",
        ))
}

fn validate_map(
    text: &str,
    resource_kind: &inventory_core::catalog::runtime::ArnResourceKind,
    trusted: Option<&IdentityContext<'_>>,
    request_url: Option<&str>,
) -> Result<(), SweepError> {
    validate_resource_arn(
        text,
        resource_kind,
        trusted.ok_or(SweepError::Operation(
            "map ARN identity requires trusted request scope",
        ))?,
        request_url,
    )
}

fn request_url_text<'a>(
    request: Option<&'a Record>,
    path: Option<&[inventory_core::record::FieldId]>,
) -> Result<Option<Cow<'a, str>>, SweepError> {
    path.map(|path| {
        required_text(
            request.ok_or(SweepError::Operation(
                "queue ARN identity requires actual request URL",
            ))?,
            path,
        )
    })
    .transpose()
}

fn map_identity<'a>(
    row: &'a Record,
    identity: &IdentityPlan,
    trusted: Option<&IdentityContext<'_>>,
    request: Option<&Record>,
) -> Result<Cow<'a, str>, SweepError> {
    let IdentityPlan::MapArn {
        path,
        key,
        resource_kind,
        request_url,
    } = identity
    else {
        return Err(SweepError::Operation(
            "unexpected contextual identity recipe",
        ));
    };
    let text = map_text(row, path, key)?;
    let request_url = request_url_text(request, request_url.as_deref())?;
    validate_map(&text, resource_kind, trusted, request_url.as_deref())?;
    Ok(text)
}
