use super::{State, ViolationRule, pagination, paths};
use crate::{
    overlay::{Binding, MemberPath, OperationName, OperationOverlay},
    plans::{self, Model, Vocabulary},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn gather_required_inputs(state: &mut State<'_, '_, '_>) {
    state.required = required_paths(state);
    state.required.sort();
    apply_fixed_inputs(state);
    apply_explicit_bindings(state);
    state
        .derived_bindings
        .keys()
        .for_each(|path| paths::mark_provided(&mut state.provided, path.as_str()));
    apply_page_size(state);
}
fn required_paths(state: &mut State<'_, '_, '_>) -> Vec<String> {
    let supplied = state
        .item
        .fixed
        .keys()
        .chain(state.item.bind.keys())
        .map(|path| path.as_str().to_owned())
        .collect();
    match state.model.required_paths(state.name, &supplied) {
        Ok(paths) => paths,
        Err(reason) => {
            state.violations.push(paths::violation(
                state.service,
                state.name,
                "input",
                reason,
                ViolationRule::InvalidPath,
            ));
            Vec::new()
        }
    }
}

struct InputContext<'s, 'm> {
    service: &'s str,
    name: &'s str,
    model: &'s Model<'m>,
    overlays: &'s BTreeMap<String, OperationOverlay>,
    vocabulary: &'s Vocabulary,
    provided: &'s mut BTreeSet<String>,
    violations: &'s mut Vec<super::Violation>,
}

fn apply_fixed_inputs(state: &mut State<'_, '_, '_>) {
    let mut context = InputContext {
        service: state.service,
        name: state.name,
        model: state.model,
        overlays: state.overlays,
        vocabulary: state.vocabulary,
        provided: &mut state.provided,
        violations: &mut state.violations,
    };
    state
        .item
        .fixed
        .iter()
        .for_each(|(path, value)| check_fixed_input(&mut context, path, value));
}

fn check_fixed_input(
    context: &mut InputContext<'_, '_>,
    path: &MemberPath,
    value: &crate::overlay::FixedValue,
) {
    match context
        .model
        .resolve_member(context.name, "input", path.as_str())
    {
        Ok(Some((member, target)))
            if paths::fixed_matches(&target, member, value, context.model) =>
        {
            paths::mark_provided(context.provided, path.as_str());
        }
        Ok(_) => report_fixed(context, path.as_str(), ViolationRule::InvalidValue),
        Err(_reason) => report_fixed(context, path.as_str(), ViolationRule::InvalidPath),
    }
}

fn report_fixed(context: &mut InputContext<'_, '_>, value: &str, rule: ViolationRule) {
    push(
        context.service,
        context.name,
        context.violations,
        "fixed",
        value,
        rule,
    );
}

fn apply_explicit_bindings(state: &mut State<'_, '_, '_>) {
    let mut context = InputContext {
        service: state.service,
        name: state.name,
        model: state.model,
        overlays: state.overlays,
        vocabulary: state.vocabulary,
        provided: &mut state.provided,
        violations: &mut state.violations,
    };
    state
        .item
        .bind
        .iter()
        .for_each(|(path, binding)| validate_binding(&mut context, path, binding));
}

fn validate_binding(context: &mut InputContext<'_, '_>, path: &MemberPath, binding: &Binding) {
    let target = match context
        .model
        .resolve_member(context.name, "input", path.as_str())
    {
        Ok(Some((_, target))) => target,
        Ok(None) => {
            report_binding_path(context, path.as_str());
            return;
        }
        Err(_reason) => {
            report_binding_path(context, path.as_str());
            return;
        }
    };
    check_binding_target(context, path, binding, target);
}

fn report_binding_path(context: &mut InputContext<'_, '_>, value: &str) {
    push(
        context.service,
        context.name,
        context.violations,
        "bind",
        value,
        ViolationRule::InvalidPath,
    );
}

fn check_binding_target(
    context: &mut InputContext<'_, '_>,
    path: &MemberPath,
    binding: &Binding,
    target: crate::plans::model::ShapeRef<'_>,
) {
    if binding_valid(context, binding, &target) {
        paths::mark_provided(context.provided, path.as_str());
    } else {
        push(
            context.service,
            context.name,
            context.violations,
            "bind",
            path.as_str(),
            ViolationRule::InvalidValue,
        );
    }
}

fn binding_valid(
    context: &InputContext<'_, '_>,
    binding: &Binding,
    target: &crate::plans::model::ShapeRef<'_>,
) -> bool {
    match binding {
        Binding::Account | Binding::Region => {
            plans::model::shape_type(target.value) == Some("string")
        }
        Binding::From {
            operation,
            path: source,
        } => source_binding_valid(
            context.model,
            context.overlays,
            context.vocabulary,
            operation,
            source,
            target,
        ),
    }
}

fn source_binding_valid(
    model: &Model<'_>,
    overlays: &BTreeMap<String, OperationOverlay>,
    vocabulary: &Vocabulary,
    parent: &OperationName,
    source_path: &MemberPath,
    target: &crate::plans::model::ShapeRef<'_>,
) -> bool {
    paths::selected_for_parent(
        model,
        parent.as_str(),
        overlays.get(parent.as_str()),
        vocabulary,
    )
    .and_then(|selected| model.resolve_path_from(selected, source_path.as_str()).ok())
    .is_some_and(|source| pagination::compatible(model, &source, target))
}

fn push(
    service: &str,
    name: &str,
    violations: &mut Vec<super::Violation>,
    key: &str,
    value: &str,
    rule: ViolationRule,
) {
    violations.push(paths::violation(service, name, key, value, rule));
}

fn apply_page_size(state: &mut State<'_, '_, '_>) {
    let valid = paths::validate_page_size(
        state.service,
        state.name,
        &state.item,
        state.model,
        state.pagination.as_ref(),
        &mut state.violations,
    );
    if valid && page_size_provided(state) {
        mark_page_size(state);
    }
}

fn page_size_provided(state: &State<'_, '_, '_>) -> bool {
    state.item.page_size.is_some() || state.inferred_page_size.is_some()
}

fn mark_page_size(state: &mut State<'_, '_, '_>) {
    if let Some(plans::Pagination::Modeled {
        page_size: Some(path),
        ..
    }) = state.pagination.as_ref()
    {
        paths::mark_provided(&mut state.provided, path);
    }
}
