mod fields;
#[path = "typed_operand.rs"]
mod typed_operand;
use super::identity_validation::{IdentityTextOperandValidation, IdentityTextValidation};
use super::trusted_arn::{validate_arn, validate_owned_arn, validate_resource_arn};
use super::{
    IdentityContext, SweepError, key_text, path_value, required_text, template_text, value_text,
};
use inventory_core::{
    catalog::runtime::{
        ArnResourceKind, IdentityChoicePlan, IdentityFieldPlan, IdentityLeafPlan, IdentityPart,
        IdentityPlan, IdentityPredicatePlan,
    },
    inventory::{DiscoveryContext, ResourceId},
    record::{FieldId, Record, ValueRef},
};
use std::{borrow::Cow, fmt::Write};

/// `GetSubscriptionAttributes` reports the subscription owner under this key.
const SUBSCRIPTION_OWNER_KEY: &str = "Owner";

pub(crate) fn is_identity_member(
    row: &Record,
    request: &Record,
    identity: &IdentityPlan,
    member: &[FieldId],
) -> Result<bool, SweepError> {
    Ok(match identity {
        IdentityPlan::Arn(path) => path.as_ref() == member,
        IdentityPlan::Template(parts) => template_member(parts) == Some(member),
        IdentityPlan::Key(paths) => key_member(paths) == Some(member),
        IdentityPlan::CorrelatedKey(fields) => selected_member(fields) == Some(member),
        IdentityPlan::Choice(IdentityChoicePlan::PreferArn { path, fallback, .. }) => {
            let arn_matches = path.as_ref() == member;
            let fallback_matches = leaf_member(fallback) == Some(member);
            (arn_matches || fallback_matches)
                && if path_value(row.root(), path)?.is_some() {
                    arn_matches
                } else {
                    fallback_matches
                }
        }
        IdentityPlan::Choice(IdentityChoicePlan::Discriminated(branches)) => {
            branches
                .iter()
                .any(|branch| leaf_member(&branch.identity) == Some(member))
                && leaf_member(&matching_branch(row, branches, Some(request))?.identity)
                    == Some(member)
        }
        IdentityPlan::MapArn { .. } => false,
    })
}

fn leaf_member(leaf: &IdentityLeafPlan) -> Option<&[FieldId]> {
    match leaf {
        IdentityLeafPlan::Arn(path) | IdentityLeafPlan::OwnedArn { path, .. } => Some(path),
        IdentityLeafPlan::Template(parts) => template_member(parts),
        IdentityLeafPlan::Key(paths) => key_member(paths),
        IdentityLeafPlan::Fields(fields) => selected_member(fields),
    }
}

fn template_member(parts: &[IdentityPart]) -> Option<&[FieldId]> {
    match parts {
        [IdentityPart::Field(path)] => Some(path),
        _ => None,
    }
}

fn key_member(paths: &[Box<[FieldId]>]) -> Option<&[FieldId]> {
    match paths {
        [path] => Some(path),
        _ => None,
    }
}

fn selected_member(fields: &[IdentityFieldPlan]) -> Option<&[FieldId]> {
    match fields {
        [IdentityFieldPlan::Selected(path) | IdentityFieldPlan::SelectedArn(path)] => Some(path),
        _ => None,
    }
}

/// Whether `row` is owned by another account while merely sitting under the
/// scanned account's namespace (for example a subscription another account made
/// to one of its topics). Such rows belong to their owner's inventory, so
/// enumeration skips them instead of attributing them to the scanned account or
/// failing the page. A row whose ARN also names a foreign account is not
/// skipped: identity validation rejects it.
pub(crate) fn foreign_owned(
    row: &Record,
    identity: &IdentityPlan,
    trusted: &IdentityContext<'_>,
    request: Option<&Record>,
) -> Result<bool, SweepError> {
    let IdentityPlan::Choice(IdentityChoicePlan::Discriminated(branches)) = identity else {
        return Ok(false);
    };
    let IdentityLeafPlan::OwnedArn { path, owner, .. } =
        &matching_branch(row, branches, request)?.identity
    else {
        return Ok(false);
    };
    let scanned = trusted.account.as_str();
    Ok(required_text(row, owner)? != scanned
        && super::trusted_arn::names_account(&required_text(row, path)?, scanned))
}

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
        IdentityLeafPlan::OwnedArn {
            path,
            owner,
            resource_kind,
        } => {
            let text = required_text(row, path)?;
            validate_owned_arn(&text, resource_kind, &required_text(row, owner)?, trusted)?;
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
    if matches!(resource_kind, ArnResourceKind::SnsSubscription) {
        let owner = map_text(row, path, SUBSCRIPTION_OWNER_KEY)?;
        let trusted = trusted.ok_or(SweepError::Operation(
            "map ARN identity requires trusted request scope",
        ))?;
        validate_owned_arn(&text, resource_kind, &owner, Some(trusted))?;
        return Ok(text);
    }
    let request_url = request_url_text(request, request_url.as_deref())?;
    validate_map(&text, resource_kind, trusted, request_url.as_deref())?;
    Ok(text)
}
