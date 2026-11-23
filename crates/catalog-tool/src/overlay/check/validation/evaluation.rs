mod state;
use state::State;
pub(in crate::overlay) use state::StateArgs;
mod inputs;
mod selection;
mod special_inputs;
use super::derived;
use super::{OperationResult, Violation, ViolationRule, identity, pagination, paths};
use crate::{
    classes::Class,
    overlay::{Binding, ItemSelector, MemberPath, OverlayClass, Paginator, TokenPair},
    plans::Pagination,
};
use std::collections::BTreeMap;

pub(in crate::overlay) fn operation(args: StateArgs<'_, '_, '_>) -> OperationResult {
    let Some(operation_id) = args.model.operation_id(args.name) else {
        return unknown_operation(args.service, args.name);
    };
    evaluate(&mut State::new(args, operation_id))
}

fn unknown_operation(service: &str, name: &str) -> OperationResult {
    OperationResult {
        callable: None,
        effective: "unclassified",
        violations: vec![paths::violation(
            service,
            name,
            "operation",
            name,
            ViolationRule::UnknownOperation,
        )],
    }
}

fn evaluate(state: &mut State<'_, '_, '_>) -> OperationResult {
    set_classification(state);
    selection::select_items(state);
    inputs::gather_required_inputs(state);
    validate_complete_plan(state);
    make_result(state)
}

fn set_classification(state: &mut State<'_, '_, '_>) {
    let promoted = is_promoted(state);
    if promoted {
        state.violations.push(paths::violation(
            state.service,
            state.name,
            "class",
            "enumerate/expand",
            ViolationRule::OverlayPromotesDenied,
        ));
    }
    state.effective = effective_class(state, promoted);
    state.candidate = matches!(state.effective, "enumerate" | "expand");
    state.derived_bindings = derived_bindings(state);
}

fn is_promoted(state: &State<'_, '_, '_>) -> bool {
    matches!(
        state.item.class,
        Some(OverlayClass::Enumerate | OverlayClass::Expand)
    ) && matches!(state.class, Class::Denied(_))
}

fn effective_class(state: &State<'_, '_, '_>, promoted: bool) -> &'static str {
    if promoted {
        return "denied";
    }
    match state.item.class {
        Some(OverlayClass::Deny(_)) => "denied",
        Some(OverlayClass::Defer) => "deferred",
        Some(OverlayClass::Enumerate) => "enumerate",
        Some(OverlayClass::Expand) => "expand",
        None => modeled_class(state.class),
    }
}

fn modeled_class(class: Class) -> &'static str {
    match class {
        Class::Enumerate => "enumerate",
        Class::Expand => "expand",
        Class::Denied(_) => "denied",
        Class::Unclassified => "unclassified",
    }
}

fn derived_bindings(state: &State<'_, '_, '_>) -> BTreeMap<MemberPath, Binding> {
    if state.effective != "expand" {
        return BTreeMap::new();
    }
    derived::bindings(
        state.model,
        state.name,
        &state.item,
        state.overlays,
        state.vocabulary,
        state.deny,
    )
}

fn validate_complete_plan(state: &mut State<'_, '_, '_>) {
    check_required_inputs(state);
    validate_pagination(state);
    validate_identity(state);
    super::row_policy::validate(
        state.service,
        state.name,
        &state.item,
        state.model,
        state.vocabulary,
        state.overlays,
        &mut state.violations,
    );
    paths::validate_output_paths(
        state.service,
        state.name,
        &state.item,
        state.model,
        state.selected,
        &mut state.violations,
    );
    paths::validate_batch(
        state.service,
        state.name,
        &state.item,
        state.model,
        &mut state.violations,
    );
}

fn check_required_inputs(state: &mut State<'_, '_, '_>) {
    if !state.candidate {
        return;
    }
    state
        .required
        .iter()
        .filter(|path| !state.provided.contains(path.as_str()))
        .for_each(|path| {
            state.violations.push(paths::violation(
                state.service,
                state.name,
                "required",
                path,
                ViolationRule::IncompletePlan,
            ))
        });
}

fn validate_pagination(state: &mut State<'_, '_, '_>) {
    if state.candidate || state.item.paginator.is_some() {
        state.paginator_valid = pagination::validate_paginator(
            state.service,
            state.name,
            &state.item,
            state.model,
            state.pagination.as_ref(),
            &mut state.violations,
        );
    }
}

fn validate_identity(state: &mut State<'_, '_, '_>) {
    let discovery_only = if state.item.branches.is_empty() {
        state.item.emission == crate::overlay::Emission::DiscoveryOnly
    } else {
        state.item.branches.iter().all(|branch| {
            branch
                .emission
                .as_ref()
                .map_or(&state.item.emission, |emission| emission)
                == &crate::overlay::Emission::DiscoveryOnly
        })
    };
    if discovery_only || state.item.tag_endpoint.is_some() || state.item.endpoint_discovery {
        state.identity_valid = state.selected.is_some();
        return;
    }
    if state.candidate || state.item.identity.is_some() {
        state.identity_valid = identity::validate_identity(
            state.service,
            state.name,
            state.item.identity.as_ref(),
            state.model,
            state.selected,
            &mut state.violations,
        );
    }
}

fn make_result(state: &State<'_, '_, '_>) -> OperationResult {
    let complete = !state
        .violations
        .iter()
        .any(|error| error.operation == state.name);
    let callable = (state.candidate && complete && state.paginator_valid && state.identity_valid)
        .then(|| callable_handle(state));
    OperationResult {
        callable,
        effective: state.effective,
        violations: state.violations.clone(),
    }
}

fn callable_handle(state: &State<'_, '_, '_>) -> crate::overlay::check::CallableOperation {
    let class = if state.effective == "enumerate" {
        super::super::EffectiveClass::Enumerate
    } else {
        super::super::EffectiveClass::Expand
    };
    let mut plan = state.item.clone();
    if plan.items.is_none() {
        plan.items = modeled_items(state);
    }
    if plan.paginator.is_none() {
        plan.paginator = modeled_paginator(state);
    }
    if plan.page_size.is_none() {
        plan.page_size = state.inferred_page_size;
    }
    plan.bind.extend(state.derived_bindings.clone());
    crate::overlay::check::CallableOperation {
        service: state.service.to_owned(),
        operation: state.name.to_owned(),
        class,
        plan,
        origin: crate::overlay::Origin::Hand,
    }
}
fn modeled_items(state: &State<'_, '_, '_>) -> Option<ItemSelector> {
    match state.pagination.as_ref()? {
        Pagination::Modeled {
            items: Some(path), ..
        } => ItemSelector::parse(path).ok(),
        _ => None,
    }
}

fn modeled_paginator(state: &State<'_, '_, '_>) -> Option<Paginator> {
    let Pagination::Modeled {
        input_token,
        output_token,
        ..
    } = state.pagination.as_ref()?
    else {
        return None;
    };
    Some(Paginator::Token(TokenPair {
        input: MemberPath::new(input_token).ok()?,
        output: MemberPath::new(output_token).ok()?,
        optional: false,
    }))
}
