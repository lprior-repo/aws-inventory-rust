use super::{Violation, range_matches, violation};
use crate::overlay::ViolationRule;
use crate::{
    overlay::OperationOverlay,
    plans::{self, Model, Pagination},
};
use serde_json::Value;

pub(in crate::overlay::check) fn validate_page_size(
    service: &str,
    operation: &str,
    item: &OperationOverlay,
    model: &Model<'_>,
    pagination: Option<&Pagination<'_>>,
    violations: &mut Vec<Violation>,
) -> bool {
    let Some(value) = item.page_size else {
        return true;
    };
    let Some(path) = page_size_path(pagination) else {
        invalid_page_value(service, operation, value, violations);
        return false;
    };
    validate_page_value(service, operation, model, path, value, violations)
}

fn page_size_path<'a>(pagination: Option<&Pagination<'a>>) -> Option<&'a str> {
    match pagination {
        Some(Pagination::Modeled {
            page_size: Some(path),
            ..
        }) => Some(path),
        _ => None,
    }
}

fn validate_page_value(
    service: &str,
    operation: &str,
    model: &Model<'_>,
    path: &str,
    value: i64,
    violations: &mut Vec<Violation>,
) -> bool {
    let Some((member, target)) = resolve_page_member(model, operation, path) else {
        invalid_page_path(service, operation, path, violations);
        return false;
    };
    let valid = page_value_matches(model, member, target, value);
    if !valid {
        invalid_page_value(service, operation, value, violations);
    }
    valid
}

fn page_value_matches(
    model: &Model<'_>,
    member: &Value,
    target: plans::model::ShapeRef<'_>,
    value: i64,
) -> bool {
    if value <= 0
        || u32::try_from(value).is_err()
        || !range_matches(page_range(member, target.value), value)
    {
        return false;
    }
    match plans::model::shape_type(target.value) {
        Some("byte") => i8::try_from(value).is_ok(),
        Some("short") => i16::try_from(value).is_ok(),
        Some("integer") => i32::try_from(value).is_ok(),
        Some("long") => true,
        Some("string" | "enum") => crate::overlay::fixed_input::matches(
            model,
            member,
            target,
            &crate::overlay::FixedValue::String(value.to_string()),
        ),
        _ => false,
    }
}

fn resolve_page_member<'a>(
    model: &'a Model<'_>,
    operation: &str,
    path: &str,
) -> Option<(&'a Value, crate::plans::model::ShapeRef<'a>)> {
    model
        .resolve_member(operation, "input", path)
        .ok()
        .flatten()
}

fn invalid_page_path(service: &str, operation: &str, path: &str, violations: &mut Vec<Violation>) {
    violations.push(violation(
        service,
        operation,
        "page_size",
        path,
        ViolationRule::InvalidPath,
    ));
}

fn invalid_page_value(service: &str, operation: &str, value: i64, violations: &mut Vec<Violation>) {
    violations.push(violation(
        service,
        operation,
        "page_size",
        &value.to_string(),
        ViolationRule::InvalidValue,
    ));
}

pub(in crate::overlay::check) fn modeled_page_size(
    model: &Model<'_>,
    operation: &str,
    pagination: Option<&Pagination<'_>>,
) -> Option<i64> {
    let path = page_size_path(pagination)?;
    let (member, target) = model
        .resolve_member(operation, "input", path)
        .ok()
        .flatten()?;
    modeled_page_max(member, target.value)
        .filter(|value| page_value_matches(model, member, target, *value))
}

fn modeled_page_max(member: &Value, target: &Value) -> Option<i64> {
    if !matches!(
        plans::model::shape_type(target),
        Some("byte" | "short" | "integer" | "long" | "string" | "enum")
    ) {
        return None;
    }
    let range = page_range(member, target);
    let max = range_bound(range, "max")?;
    (1..=100_000)
        .contains(&max)
        .then_some(max)
        .filter(|value| range_matches(range, *value))
}
fn page_range<'a>(member: &'a Value, target: &'a Value) -> &'a Value {
    if range_bound(member, "min").is_some() || range_bound(member, "max").is_some() {
        member
    } else {
        target
    }
}

pub fn range_bound(shape: &Value, bound: &str) -> Option<i64> {
    shape
        .get("traits")?
        .get("smithy.api#range")?
        .get(bound)?
        .as_i64()
}
