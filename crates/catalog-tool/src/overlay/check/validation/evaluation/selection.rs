use super::{State, ViolationRule, paths};
use crate::{
    overlay::ItemSelector,
    plans::{self, Pagination, model::ShapeRef},
};

pub(super) fn select_items(state: &mut State<'_, '_, '_>) {
    state.pagination = plans::pagination(state.model, state.operation_id, state.vocabulary).ok();
    state.inferred_page_size = state
        .item
        .page_size
        .is_none()
        .then(|| paths::modeled_page_size(state.model, state.name, state.pagination.as_ref()))
        .flatten();
    let selected = match state.item.items.clone() {
        Some(ItemSelector::Root) => select_root(state),
        Some(ItemSelector::Member(path)) => select_member(state, path.as_str()),
        None => select_inferred(state),
    };
    state.selected = selected;
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
