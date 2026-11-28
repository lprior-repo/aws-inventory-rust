mod member_exclusions;
mod page_size;
use super::Violation;
use crate::overlay::ViolationRule;
use crate::overlay::fixed_input::range as range_matches;
use crate::{
    overlay::{FixedValue, ItemSelector, OperationOverlay},
    plans::{self, Model, Pagination, Vocabulary},
};
pub(super) use page_size::{modeled_page_size, validate_page_size};
use std::collections::BTreeSet;

pub(super) fn mark_provided(provided: &mut BTreeSet<String>, path: &str) {
    path.match_indices('.')
        .filter_map(|(offset, _)| path.get(..offset))
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
            Some(
                "string"
                    | "enum"
                    | "byte"
                    | "short"
                    | "integer"
                    | "long"
                    | "boolean"
                    | "float"
                    | "double"
                    | "blob"
                    | "timestamp"
            )
        )
        .then_some(selected);
    }
    model.resolve_path_from(selected, path).ok()
}

pub(in crate::overlay) fn selected_for_parent<'m>(
    model: &Model<'m>,
    operation: &str,
    overlay: Option<&OperationOverlay>,
    vocabulary: &Vocabulary,
) -> Option<crate::plans::model::ShapeRef<'m>> {
    let explicit = overlay.and_then(|overlay| overlay.items.as_ref());
    if let Some(ItemSelector::Nested(nested)) = explicit {
        return nested_selected(model, operation, nested);
    }
    if let Some(ItemSelector::Explicit { path, kind }) = explicit {
        return explicit_selected(model, operation, path.as_str(), *kind);
    }
    if matches!(explicit, Some(ItemSelector::Root)) {
        return root_selection(model, operation);
    }
    let path =
        explicit_path(explicit).or_else(|| inferred_item_path(model, operation, vocabulary))?;
    selected_aggregate(model, operation, path)
}
fn nested_selected<'m>(
    model: &Model<'m>,
    operation: &str,
    nested: &crate::overlay::NestedItems,
) -> Option<crate::plans::model::ShapeRef<'m>> {
    if nested.items.as_str().contains('.') || nested.owner.as_str().contains('.') {
        return None;
    }
    let parent_list = model
        .resolve_path(operation, "output", nested.parents.as_str())
        .ok()??;
    if plans::model::shape_type(parent_list.value) != Some("list") {
        return None;
    }
    let parent = model.unwrap_aggregates(parent_list).ok()?;
    let item_list = model
        .resolve_path_from(parent, nested.items.as_str())
        .ok()?;
    let owner = model
        .resolve_path_from(parent, nested.owner.as_str())
        .ok()?;
    if plans::model::shape_type(item_list.value) != Some("list")
        || plans::model::shape_type(owner.value) != Some("string")
    {
        return None;
    }
    model.unwrap_aggregates(item_list).ok()
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
    member_exclusions::validate(service, operation, item, model, violations);
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
        .filter(|path| {
            path.as_str() != "$"
                || selected.as_ref().is_none_or(|shape| {
                    !matches!(plans::model::shape_type(shape.value), Some("list" | "map"))
                })
        })
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
    selected.is_none_or(|selected| resolve_selected(model, *selected, path.as_str()).is_none())
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
    crate::overlay::fixed_input::matches(model, member, *target, value)
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

pub(in crate::overlay::check::validation) fn explicit_selected<'m>(
    model: &Model<'m>,
    operation: &str,
    path: &str,
    kind: crate::overlay::SelectionKind,
) -> Option<crate::plans::model::ShapeRef<'m>> {
    use crate::overlay::SelectionKind;
    let selected = if path == "$" {
        model.side_root(operation, "output")?
    } else {
        model.resolve_path(operation, "output", path).ok()??
    };
    match (kind, plans::model::shape_type(selected.value)?) {
        (SelectionKind::Structure, "structure") => Some(selected),
        (
            SelectionKind::Scalar,
            "string" | "enum" | "byte" | "short" | "integer" | "long" | "boolean",
        ) => Some(selected),
        (SelectionKind::List, "list") => aggregate_member(model, selected, "member"),
        (SelectionKind::MapKeys, "map") => aggregate_member(model, selected, "key"),
        (SelectionKind::MapEntries | SelectionKind::MapValues, "map") => {
            aggregate_member(model, selected, "value")
        }
        _ => None,
    }
}

fn aggregate_member<'m>(
    model: &Model<'m>,
    shape: crate::plans::model::ShapeRef<'m>,
    name: &str,
) -> Option<crate::plans::model::ShapeRef<'m>> {
    let target = shape.value.get(name)?.get("target")?.as_str()?;
    model.shape(target)
}
