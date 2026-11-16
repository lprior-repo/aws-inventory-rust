#[path = "flow/rows.rs"]
mod rows;
use super::super::{
    CallableOperation, ReviewDisposition, ReviewedDecision, ReviewedService, Violation,
    ViolationRule,
};
use super::{ReportRow, ServiceCheck, binding_cycle_operations, validate_reference};
use crate::{
    classes::{Class, DenyList},
    overlay::{Binding, ServiceOverlay},
    plans::{self, Model},
};
use rows::*;
use std::collections::{BTreeMap, BTreeSet};
mod operation;
type OperationRows = (
    Vec<CallableOperation>,
    Vec<ReviewedDecision>,
    Vec<ReportRow>,
);

pub(super) fn check_service(
    overlay: &ServiceOverlay,
    model: &Model<'_>,
    classes: &BTreeMap<String, Class>,
    deny: &DenyList<'_>,
    vocabulary: &plans::Vocabulary,
) -> ServiceCheck {
    let names: BTreeSet<&str> = model.operation_names().collect();
    let (plans, mut errors) = review_data(overlay, &names);
    let inputs = OperationInputs {
        overlay,
        model,
        classes,
        deny,
        vocabulary,
        plan_map: &plans,
    };
    let (callable, decisions, rows) = check_operations(&inputs, &mut errors);
    check_references_and_cycles(overlay, &callable, &names, &mut errors);
    finish(overlay, callable, decisions, rows, errors)
}

struct OperationInputs<'r, 'm> {
    overlay: &'r ServiceOverlay,
    model: &'r Model<'m>,
    classes: &'r BTreeMap<String, Class>,
    deny: &'r DenyList<'m>,
    vocabulary: &'r plans::Vocabulary,
    plan_map: &'r BTreeMap<String, super::super::super::OperationOverlay>,
}
fn review_data(
    overlay: &ServiceOverlay,
    names: &BTreeSet<&str>,
) -> (
    BTreeMap<String, super::super::super::OperationOverlay>,
    Vec<Violation>,
) {
    (operation_plans(overlay), unknown_operations(overlay, names))
}
fn check_references_and_cycles(
    overlay: &ServiceOverlay,
    callable: &[CallableOperation],
    names: &BTreeSet<&str>,
    errors: &mut Vec<Violation>,
) {
    check_references(overlay, callable, names, errors);
    append_cycles(overlay, callable, errors);
}

fn operation_plans(
    overlay: &ServiceOverlay,
) -> BTreeMap<String, super::super::super::OperationOverlay> {
    overlay
        .operations()
        .iter()
        .map(|(name, plan)| (name.as_str().to_owned(), plan.clone()))
        .collect()
}

fn unknown_operations(overlay: &ServiceOverlay, names: &BTreeSet<&str>) -> Vec<Violation> {
    overlay
        .operations()
        .iter()
        .filter(|(name, _)| !names.contains(name.as_str()))
        .map(|(name, _)| {
            super::violation(
                overlay.service(),
                name.as_str(),
                "operation",
                name.as_str(),
                ViolationRule::UnknownOperation,
            )
        })
        .collect()
}

fn check_operations(
    inputs: &OperationInputs<'_, '_>,
    errors: &mut Vec<Violation>,
) -> OperationRows {
    let mut rows = (Vec::new(), Vec::new(), Vec::new());
    for name in inputs.model.operation_names() {
        append_operation(inputs, errors, name, &mut rows);
    }
    rows
}

fn append_operation(
    inputs: &OperationInputs<'_, '_>,
    errors: &mut Vec<Violation>,
    name: &str,
    (callable, decisions, rows): &mut OperationRows,
) {
    let class = inputs.classes.get(name).copied();
    let result = operation::check_operation(
        inputs.overlay,
        inputs.model,
        inputs.deny,
        inputs.vocabulary,
        name,
        class.unwrap_or(Class::Unclassified),
        inputs.plan_map,
    );
    errors.extend(result.3);
    callable.extend(result.0);
    decisions.push(result.1);
    rows.push(result.2);
}

fn check_references(
    overlay: &ServiceOverlay,
    callable: &[CallableOperation],
    names: &BTreeSet<&str>,
    errors: &mut Vec<Violation>,
) {
    let callable_names = callable_names(callable);
    super::stages::references(overlay, callable, errors);
    for (operation, plan) in overlay.operations() {
        check_plan_references(
            overlay.service(),
            operation.as_str(),
            plan,
            names,
            &callable_names,
            errors,
        );
    }
    for handle in callable {
        check_handle_references(overlay.service(), handle, names, &callable_names, errors);
    }
}

fn callable_names(callable: &[CallableOperation]) -> BTreeSet<&str> {
    callable.iter().map(CallableOperation::operation).collect()
}

fn check_plan_references(
    service: &str,
    operation: &str,
    plan: &super::super::super::OperationOverlay,
    names: &BTreeSet<&str>,
    callable: &BTreeSet<&str>,
    errors: &mut Vec<Violation>,
) {
    check_skipped_references(service, operation, plan, names, callable, errors);
    check_plan_bindings(service, operation, plan, names, callable, errors);
    check_batch_reference(service, operation, plan, names, callable, errors);
}

fn check_skipped_references(
    service: &str,
    operation: &str,
    plan: &super::super::super::OperationOverlay,
    names: &BTreeSet<&str>,
    callable: &BTreeSet<&str>,
    errors: &mut Vec<Violation>,
) {
    plan.skip.iter().for_each(|skipped| {
        validate_reference(
            service,
            operation,
            "skip",
            skipped.as_str(),
            names,
            callable,
            errors,
        )
    });
}

fn check_plan_bindings(
    service: &str,
    operation: &str,
    plan: &super::super::super::OperationOverlay,
    names: &BTreeSet<&str>,
    callable: &BTreeSet<&str>,
    errors: &mut Vec<Violation>,
) {
    crate::overlay::policy::effective_bindings(plan).for_each(|binding| {
        check_binding_reference(service, operation, binding, names, callable, errors)
    });
}

fn check_batch_reference(
    service: &str,
    operation: &str,
    plan: &super::super::super::OperationOverlay,
    names: &BTreeSet<&str>,
    callable: &BTreeSet<&str>,
    errors: &mut Vec<Violation>,
) {
    if let Some(batch) = &plan.batch {
        validate_reference(
            service,
            operation,
            "batch.replaces",
            batch.replaces.as_str(),
            names,
            callable,
            errors,
        );
    }
}

fn check_handle_references(
    service: &str,
    handle: &CallableOperation,
    names: &BTreeSet<&str>,
    callable: &BTreeSet<&str>,
    errors: &mut Vec<Violation>,
) {
    for binding in crate::overlay::policy::effective_bindings(&handle.plan) {
        check_binding_reference(
            service,
            handle.operation(),
            binding,
            names,
            callable,
            errors,
        );
    }
}

fn check_binding_reference(
    service: &str,
    operation: &str,
    binding: &Binding,
    names: &BTreeSet<&str>,
    callable: &BTreeSet<&str>,
    errors: &mut Vec<Violation>,
) {
    let binding = if let Binding::Transformed { binding, .. } = binding { binding.as_ref() } else { binding };
    if let Binding::FromAny(alternatives) = binding {
        alternatives.iter().for_each(|alternative| validate_reference(
            service, operation, "bind.from_any", alternative.operation.as_str(), names, callable, errors,
        ));
        return;
    }
    if let Binding::From { operation: parent, .. } | Binding::FromArn { operation: parent, .. } | Binding::FromUrl { operation: parent, .. }
        | Binding::Context { operation: parent, .. } | Binding::FromBranch { operation: parent, .. } | Binding::ContextFromBranch { operation: parent, .. } = binding {
        validate_reference(
            service,
            operation,
            "bind.from",
            parent.as_str(),
            names,
            callable,
            errors,
        );
    }
}
