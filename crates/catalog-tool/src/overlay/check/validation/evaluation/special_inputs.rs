use super::{State, ViolationRule, paths};
use crate::overlay::{OperationOverlay, TagEndpointParent, TagEndpointRequest};
use crate::plans::{Model, Vocabulary, model::{ShapeRef, shape_type}};
use std::collections::{BTreeMap, BTreeSet};

struct TagContext<'a, 'm> {
    name: &'a str,
    model: &'a Model<'m>,
    overlays: &'a BTreeMap<String, OperationOverlay>,
    vocabulary: &'a Vocabulary,
}

pub(super) fn apply(state: &mut State<'_, '_, '_>) {
    let tag_paths = tag_inputs(state);
    state.provided.extend(tag_paths);
}


fn tag_inputs(state: &mut State<'_, '_, '_>) -> BTreeSet<String> {
    let Some(endpoint) = &state.item.tag_endpoint else { return BTreeSet::new(); };
    if endpoint.parents.is_empty() || endpoint.parents.len() > 64 {
        invalid(state, "tag_endpoint", "parents");
        return BTreeSet::new();
    }
    let mut names = BTreeSet::new();
    let context = TagContext { name: state.name, model: state.model,
        overlays: state.overlays, vocabulary: state.vocabulary };
    let shared = endpoint.parents.iter().fold(None::<BTreeSet<String>>, |shared, parent| {
        let mut provided = BTreeSet::new();
        if names.insert(parent.operation.as_str()) && parent_valid(&context, parent) {
            if let Some(path) = request_input(&parent.request) {
                paths::mark_provided(&mut provided, path);
            }
        } else {
            state.violations.push(paths::violation(state.service, state.name, "tag_endpoint.parent",
                parent.operation.as_str(), ViolationRule::InvalidValue));
        }
        Some(match shared {
            Some(shared) => provided.intersection(&shared).cloned().collect(),
            None => provided,
        })
    });
    shared.unwrap_or_else(BTreeSet::new)
}

fn parent_valid(state: &TagContext<'_, '_>, parent: &TagEndpointParent) -> bool {
    if parent.operation.as_str() == state.name || state.model.operation_id(parent.operation.as_str()).is_none() {
        return false;
    }
    let Some(input) = request_input(&parent.request) else { return true; };
    let target = state.model.resolve_path(state.name, "input", input).ok().flatten();
    if !text(target) { return false; }
    match &parent.request {
        TagEndpointRequest::None | TagEndpointRequest::Owner { .. } => true,
        TagEndpointRequest::Context { parent: path, .. } =>
            text(state.model.resolve_path(parent.operation.as_str(), "input", path.as_str()).ok().flatten()),
        TagEndpointRequest::Member { parent: path, .. } => {
            paths::selected_for_parent(state.model, parent.operation.as_str(),
                state.overlays.get(parent.operation.as_str()), state.vocabulary)
                .and_then(|item| paths::resolve_selected(state.model, item, path.as_str()))
                .is_some_and(|source| text(Some(source)))
        }
    }
}

fn request_input(request: &TagEndpointRequest) -> Option<&str> {
    match request {
        TagEndpointRequest::None => None,
        TagEndpointRequest::Member { input, .. } | TagEndpointRequest::Context { input, .. }
            | TagEndpointRequest::Owner { input } => Some(input.as_str()),
    }
}

fn text(target: Option<ShapeRef<'_>>) -> bool {
    target.is_some_and(|target| matches!(shape_type(target.value), Some("string" | "enum")))
}


fn invalid(state: &mut State<'_, '_, '_>, key: &str, value: &str) {
    state.violations.push(paths::violation(state.service, state.name, key, value,
        ViolationRule::InvalidValue));
}
