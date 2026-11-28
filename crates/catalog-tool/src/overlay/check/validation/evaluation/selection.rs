use super::{State, ViolationRule, paths};
use crate::{
    overlay::ItemSelector,
    plans::{self, Pagination, model::ShapeRef},
};

pub(super) fn select_items(state: &mut State<'_, '_, '_>) {
    normalize_discovery(state);
    state.pagination = plans::pagination(state.model, state.operation_id, state.vocabulary).ok();
    state.inferred_page_size = state
        .item
        .page_size
        .is_none()
        .then(|| paths::modeled_page_size(state.model, state.name, state.pagination.as_ref()))
        .flatten()
        .filter(|_| !page_size_overridden(state));
    let selected = match state.item.items.clone() {
        Some(ItemSelector::Root) => select_root(state),
        Some(ItemSelector::Member(path)) => select_member(state, path.as_str()),
        Some(ItemSelector::Explicit { path, kind }) => {
            paths::explicit_selected(state.model, state.name, path.as_str(), kind)
                .or_else(|| invalid_items(state, path.as_str(), ViolationRule::InvalidPath))
        }
        Some(ItemSelector::Nested(nested)) => {
            paths::selected_for_parent(
                state.model,
                state.name,
                Some(&state.item),
                state.vocabulary,
            )
            .or_else(|| {
                invalid_items(
                    state,
                    nested.parents.as_str(),
                    ViolationRule::InvalidPath,
                )
            })
        }
        None => select_inferred(state),
    };
    state.selected = selected;
}

fn normalize_discovery(state: &mut State<'_, '_, '_>) {
    if state.item.endpoint_discovery {
        state.item.items.get_or_insert(ItemSelector::Root);
        state
            .item
            .paginator
            .get_or_insert(crate::overlay::Paginator::None);
    }
}

fn page_size_overridden(state: &State<'_, '_, '_>) -> bool {
    let Some(Pagination::Modeled {
        page_size: Some(path),
        ..
    }) = &state.pagination
    else {
        return false;
    };
    state
        .item
        .fixed
        .keys()
        .chain(state.item.bind.keys())
        .any(|input| {
            input.as_str() == *path
                || path
                    .strip_prefix(input.as_str())
                    .is_some_and(|suffix| suffix.starts_with('.'))
        })
}

fn select_root<'a, 'm, 'd>(state: &mut State<'a, 'm, 'd>) -> Option<ShapeRef<'m>> {
    let root = state.model.side_root(state.name, "output");
    match root {
        Some(root) if plans::model::shape_type(root.value) == Some("structure") => Some(root),
        _ => invalid_items(state, "$", ViolationRule::InvalidPath),
    }
}

fn select_member<'a, 'm, 'd>(state: &mut State<'a, 'm, 'd>, path: &str) -> Option<ShapeRef<'m>> {
    match state.model.resolve_path(state.name, "output", path) {
        Ok(Some(shape)) => aggregate_item(state, shape, path),
        Ok(None) => invalid_items(state, path, ViolationRule::InvalidPath),
        Err(_reason) => invalid_items(state, path, ViolationRule::InvalidPath),
    }
}

fn select_inferred<'a, 'm, 'd>(state: &mut State<'a, 'm, 'd>) -> Option<ShapeRef<'m>> {
    let path = match state.pagination.as_ref() {
        Some(Pagination::Modeled { items, .. }) => *items,
        _ => None,
    };
    let Some(path) = path else {
        return missing_inferred_items(state);
    };
    select_member(state, path)
}

fn missing_inferred_items<'a, 'm, 'd>(state: &mut State<'a, 'm, 'd>) -> Option<ShapeRef<'m>> {
    if state.candidate {
        invalid_items(state, "missing", ViolationRule::IncompletePlan)
    } else {
        None
    }
}

fn aggregate_item<'a, 'm, 'd>(
    state: &mut State<'a, 'm, 'd>,
    shape: ShapeRef<'m>,
    path: &str,
) -> Option<ShapeRef<'m>> {
    if plans::model::shape_type(shape.value) == Some("structure") {
        return Some(shape);
    }
    if !matches!(plans::model::shape_type(shape.value), Some("list" | "map")) {
        return invalid_items(state, path, ViolationRule::InvalidPath);
    }
    state.model.unwrap_aggregates(shape).ok()
}

fn invalid_items<'a, 'm, 'd>(
    state: &mut State<'a, 'm, 'd>,
    value: &str,
    rule: ViolationRule,
) -> Option<ShapeRef<'m>> {
    state.violations.push(paths::violation(
        state.service,
        state.name,
        "items",
        value,
        rule,
    ));
    None
}
