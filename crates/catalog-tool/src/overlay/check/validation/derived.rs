use super::{pagination::compatible, paths::selected_for_parent};
use crate::{
    classes::DenyList,
    overlay::{Binding, MemberPath, OperationName, OperationOverlay},
    plans::{self, Model, Vocabulary},
};
use std::collections::BTreeMap;

pub(super) fn bindings(
    model: &Model<'_>,
    operation: &str,
    plan: &OperationOverlay,
    overlays: &BTreeMap<String, OperationOverlay>,
    vocabulary: &Vocabulary,
    deny: &DenyList<'_>,
) -> BTreeMap<MemberPath, Binding> {
    let Some((parent, identifiers)) = expansion(model, operation, deny) else {
        return BTreeMap::new();
    };
    // A list operation that is its own resource's `list` has no parent rows to bind from.
    if parent == operation {
        return BTreeMap::new();
    }
    let Some(selected) = selected_for_parent(model, parent, overlays.get(parent), vocabulary)
    else {
        return BTreeMap::new();
    };
    identifiers
        .iter()
        .filter_map(|(member, identifier)| {
            binding(model, operation, plan, parent, selected, member, identifier)
        })
        .collect()
}

fn expansion<'a>(
    model: &Model<'a>,
    operation: &str,
    deny: &DenyList<'_>,
) -> Option<(&'a str, BTreeMap<&'a str, &'a str>)> {
    let id = model.operation_id(operation)?;
    match plans::expansion(model, id, deny).ok()? {
        plans::Expansion::Modeled {
            resource,
            identifiers,
            list_source: true,
            ambiguous_resource: false,
        } => {
            let list_id = model
                .shape(resource)
                .and_then(|shape| shape.value.get("list"))
                .and_then(plans::model::target)?;
            Some((plans::model::name(list_id), identifiers))
        }
        _ => None,
    }
}

fn binding(
    model: &Model<'_>,
    operation: &str,
    plan: &OperationOverlay,
    parent: &str,
    selected: crate::plans::model::ShapeRef<'_>,
    member: &str,
    identifier: &str,
) -> Option<(MemberPath, Binding)> {
    if overridden(plan, member) {
        return None;
    }
    let (_, input) = model.resolve_member(operation, "input", member).ok()??;
    let source = model.resolve_path_from(selected, identifier).ok()?;
    if !compatible(model, &source, &input) {
        return None;
    }
    let member = MemberPath::new(member).ok()?;
    let operation = OperationName::new(parent).ok()?;
    let path = MemberPath::new(identifier).ok()?;
    Some((member, Binding::From { operation, path }))
}

fn overridden(plan: &OperationOverlay, member: &str) -> bool {
    let declared = |path: &MemberPath| path.as_str() == member;
    plan.fixed.keys().any(declared)
        || plan.bind.keys().any(declared)
        || plan
            .branches
            .iter()
            .any(|branch| branch.fixed.keys().any(declared) || branch.bind.keys().any(declared))
}
