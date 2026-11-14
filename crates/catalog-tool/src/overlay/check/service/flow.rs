use super::super::{CallableOperation, ReviewedService, Violation, ViolationRule};
use super::{ReportRow, ServiceCheck, binding_cycle_operations, validate_reference};
use crate::{
    classes::{Class, DenyList},
    overlay::{Binding, ServiceOverlay},
    plans::{self, Model},
};
use std::collections::{BTreeMap, BTreeSet};
mod operation;
type OperationRows = (Vec<CallableOperation>, Vec<ReportRow>);

pub(super) fn check_service(
    overlay: &ServiceOverlay,
    model: &Model<'_>,
    classes: &BTreeMap<String, Class>,
    deny: &DenyList<'_>,
    vocabulary: &plans::Vocabulary,
) -> ServiceCheck {
    let names: BTreeSet<&str> = model.operation_names().collect();
    let (plans, mut errors) = review_data(overlay, &names);
    let (callable, rows) = check_operations(
        overlay,
        model,
        classes,
        deny,
        vocabulary,
        &plans,
        &mut errors,
    );
    check_references_and_cycles(overlay, &callable, &names, &mut errors);
    finish(overlay.service(), callable, rows, errors)
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
    overlay: &ServiceOverlay,
    model: &Model<'_>,
    classes: &BTreeMap<String, Class>,
    deny: &DenyList<'_>,
    vocabulary: &plans::Vocabulary,
    plan_map: &BTreeMap<String, super::super::super::OperationOverlay>,
    errors: &mut Vec<Violation>,
) -> OperationRows {
    let mut callable = Vec::new();
    let mut rows = Vec::new();
    for name in model.operation_names() {
        let class = classes.get(name).copied().unwrap_or(Class::Unclassified);
        let result =
            operation::check_operation(overlay, model, deny, vocabulary, name, class, plan_map);
        errors.extend(result.2);
        if let Some(handle) = result.0 {
            callable.push(handle);
        }
        rows.push(result.1);
    }
    (callable, rows)
}

fn check_references(
    overlay: &ServiceOverlay,
    callable: &[CallableOperation],
    names: &BTreeSet<&str>,
    errors: &mut Vec<Violation>,
) {
    let callable_names = callable_names(callable);
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
    plan.bind.values().for_each(|binding| {
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
    for binding in handle.plan.bind.values() {
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
    if let Binding::From {
        operation: parent, ..
    } = binding
    {
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

fn append_cycles(
    overlay: &ServiceOverlay,
    callable: &[CallableOperation],
    errors: &mut Vec<Violation>,
) {
    for operation in binding_cycle_operations(overlay, callable) {
        errors.push(super::violation(
            overlay.service(),
            &operation,
            "bind",
            &operation,
            ViolationRule::BindingCycle,
        ));
    }
}

fn finish(
    service: &str,
    mut callable: Vec<CallableOperation>,
    rows: Vec<ReportRow>,
    mut errors: Vec<Violation>,
) -> ServiceCheck {
    callable.sort_by(|left, right| left.operation.cmp(&right.operation));
    errors.sort();
    errors.dedup();
    (
        ReviewedService {
            service: service.to_owned(),
            callable,
        },
        rows,
        errors,
    )
}
