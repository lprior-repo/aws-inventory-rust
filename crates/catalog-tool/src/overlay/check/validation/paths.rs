mod page_size;
use super::Violation;
use crate::overlay::ViolationRule;
use crate::{
    overlay::{FixedValue, ItemSelector, OperationOverlay},
    plans::{self, Model, Pagination, Vocabulary},
};
pub(super) use page_size::{modeled_page_size, range_bound, validate_page_size};
use std::collections::BTreeSet;

pub(super) fn mark_provided(provided: &mut BTreeSet<String>, path: &str) {
    path.match_indices('.')
        .map(|(offset, _)| &path[..offset])
        .chain(std::iter::once(path))
        .for_each(|prefix| {
            provided.insert(prefix.to_owned());
        });
}

pub(super) fn resolve_selected<'a>(
    model: &Model<'a>,
    selected: crate::plans::model::ShapeRef<'a>,
    path: &str,
) -> Option<crate::plans::model::ShapeRef<'a>> {
    if path == "$" {
        return matches!(
            plans::model::shape_type(selected.value),
            Some("string" | "enum" | "byte" | "short" | "integer" | "long" | "boolean" | "float" | "double" | "blob" | "timestamp")
        )
        .then_some(selected);
    }
    model.resolve_path_from(selected, path).ok()
}

pub(super) fn selected_for_parent<'m>(
    model: &Model<'m>,
    operation: &str,
    overlay: Option<&OperationOverlay>,
    vocabulary: &Vocabulary,
) -> Option<crate::plans::model::ShapeRef<'m>> {
    let explicit = overlay.and_then(|overlay| overlay.items.as_ref());
    if matches!(explicit, Some(ItemSelector::Root)) {
        return root_selection(model, operation);
    }
    let path =
        explicit_path(explicit).or_else(|| inferred_item_path(model, operation, vocabulary))?;
    selected_aggregate(model, operation, path)
}

fn root_selection<'m>(
    model: &Model<'m>,
    operation: &str,
) -> Option<crate::plans::model::ShapeRef<'m>> {
    model
        .side_root(operation, "output")
        .filter(|root| plans::model::shape_type(root.value) == Some("structure"))
}

fn explicit_path(selector: Option<&ItemSelector>) -> Option<&str> {
    match selector {
        Some(ItemSelector::Member(path)) => Some(path.as_str()),
        _ => None,
    }
}

fn inferred_item_path<'a>(
    model: &Model<'a>,
    operation: &str,
    vocabulary: &Vocabulary,
) -> Option<&'a str> {
    model
        .operation_id(operation)
        .and_then(|id| plans::pagination(model, id, vocabulary).ok())
        .and_then(|pagination| match pagination {
            Pagination::Modeled { items, .. } => items,
            _ => None,
        })
}

fn selected_aggregate<'m>(
    model: &Model<'m>,
    operation: &str,
    path: &str,
) -> Option<crate::plans::model::ShapeRef<'m>> {
    let shape = model.resolve_path(operation, "output", path).ok()??;
    match plans::model::shape_type(shape.value) {
        Some("structure") => Some(shape),
        Some("list" | "map") => model.unwrap_aggregates(shape).ok(),
        _ => None,
    }
}

pub(super) fn validate_output_paths(
    service: &str,
    operation: &str,
    item: &OperationOverlay,
    model: &Model<'_>,
    selected: Option<crate::plans::model::ShapeRef<'_>>,
    violations: &mut Vec<Violation>,
) {
    let mut validate = |key, paths| {
        check_paths(
            service,
            operation,
            key,
            paths,
            model,
            selected.as_ref(),
            violations,
        );
    };
    validate("exclude_from_hash", &item.exclude_from_hash);
    validate("queryable", &item.queryable);
    item.tags
        .iter()
        .for_each(|path| validate("tags", std::slice::from_ref(path)));
}

fn check_paths(
    service: &str,
    operation: &str,
    key: &str,
    paths: &[crate::overlay::MemberPath],
    model: &Model<'_>,
    selected: Option<&crate::plans::model::ShapeRef<'_>>,
    violations: &mut Vec<Violation>,
) {
    paths
        .iter()
        .filter(|path| invalid_item_path(model, selected, path))
        .for_each(|path| {
            violations.push(violation(
                service,
                operation,
                key,
                path.as_str(),
                ViolationRule::InvalidPath,
            ))
        });
}

fn invalid_item_path(
    model: &Model<'_>,
    selected: Option<&crate::plans::model::ShapeRef<'_>>,
    path: &crate::overlay::MemberPath,
) -> bool {
    selected.is_none_or(|selected| model.resolve_path_from(*selected, path.as_str()).is_err())
}

pub(super) fn validate_batch(
    service: &str,
    operation: &str,
    item: &OperationOverlay,
    model: &Model<'_>,
    violations: &mut Vec<Violation>,
) {
    let Some(batch) = &item.batch else { return };
    let valid = model
        .resolve_path(operation, "input", batch.member.as_str())
        .ok()
        .flatten()
        .is_some_and(|shape| plans::model::shape_type(shape.value) == Some("list"));
    if !valid {
        violations.push(violation(
            service,
            operation,
            "batch.member",
            batch.member.as_str(),
            ViolationRule::InvalidPath,
        ));
    }
}

pub(super) fn fixed_matches(
    target: &crate::plans::model::ShapeRef<'_>,
    member: &serde_json::Value,
    value: &FixedValue,
    model: &Model<'_>,
) -> bool {
    match (plans::model::shape_type(target.value), value) {
        (Some("string"), FixedValue::String(text)) => {
            length_matches(member, text.len()) && length_matches(target.value, text.len())
        }
        (Some("enum"), FixedValue::String(text)) => enum_value_matches(target.value, text),
        (Some("integer"), FixedValue::Integer(number)) => {
            range_matches(member, *number) && range_matches(target.value, *number)
        }
        (Some("boolean"), FixedValue::Boolean(_)) => true,
        (Some("list"), FixedValue::Strings(items)) => {
            fixed_string_list(target, member, items, model)
        }
        _ => false,
    }
}

fn enum_value_matches(shape: &serde_json::Value, text: &str) -> bool {
    shape
        .get("members")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|members| {
            members.values().any(|member| {
                member
                    .get("traits")
                    .and_then(|traits| traits.get("smithy.api#enumValue"))
                    .and_then(serde_json::Value::as_str)
                    == Some(text)
            })
        })
}

fn fixed_string_list(
    target: &crate::plans::model::ShapeRef<'_>,
    member: &serde_json::Value,
    items: &[String],
    model: &Model<'_>,
) -> bool {
    let element = target
        .value
        .get("member")
        .and_then(crate::plans::model::target)
        .and_then(|id| model.shape(id));
    element.is_some_and(|element| {
        match plans::model::shape_type(element.value) {
            Some("string") => items.iter()
                .all(|item| length_matches(element.value, item.len())),
            Some("enum") => items.iter()
                .all(|item| enum_value_matches(element.value, item)),
            _ => false,
        }
    }) && length_matches(member, items.len())
        && length_matches(target.value, items.len())
}

fn range_matches(shape: &serde_json::Value, value: i64) -> bool {
    range_bound(shape, "min").is_none_or(|min| value >= min)
        && range_bound(shape, "max").is_none_or(|max| value <= max)
}

fn length_matches(shape: &serde_json::Value, value: usize) -> bool {
    let value = i64::try_from(value).ok();
    let (min, max) = length_bounds(shape);
    value.is_some_and(|value| {
        min.is_none_or(|min| value >= min) && max.is_none_or(|max| value <= max)
    })
}

fn length_bounds(shape: &serde_json::Value) -> (Option<i64>, Option<i64>) {
    let range = shape
        .get("traits")
        .and_then(|traits| traits.get("smithy.api#length"));
    let bound = |name| {
        range
            .and_then(|range| range.get(name))
            .and_then(serde_json::Value::as_i64)
    };
    (bound("min"), bound("max"))
}

pub(in crate::overlay::check) fn violation(
    service: &str,
    operation: &str,
    key: &str,
    value: &str,
    rule: ViolationRule,
) -> Violation {
    Violation {
        service: service.to_owned(),
        operation: operation.to_owned(),
        key: key.to_owned(),
        value: value.to_owned(),
        rule,
    }
}
