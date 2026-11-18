mod flow;
mod stages;
use super::{CallableOperation, ReviewedService, Violation, ViolationRule};
use crate::{
    classes::{Class, DenyList},
    overlay::{Binding, ServiceOverlay},
    plans::{self, Model},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) type ReportRow = (String, String, String, String);
pub(super) type ServiceCheck = (ReviewedService, Vec<ReportRow>, Vec<Violation>);

pub(super) fn check_service(
    overlay: &ServiceOverlay,
    model: &Model<'_>,
    classes: &BTreeMap<String, Class>,
    deny: &DenyList<'_>,
    vocabulary: &plans::Vocabulary,
) -> ServiceCheck {
    flow::check_service(overlay, model, classes, deny, vocabulary)
}

fn validate_reference(
    service: &str,
    operation: &str,
    key: &str,
    target: &str,
    names: &BTreeSet<&str>,
    callable: &BTreeSet<&str>,
    violations: &mut Vec<Violation>,
) {
    let rule = reference_rule(target, names, callable);
    if let Some(rule) = rule {
        violations.push(violation(service, operation, key, target, rule));
    }
}

fn reference_rule(
    target: &str,
    names: &BTreeSet<&str>,
    callable: &BTreeSet<&str>,
) -> Option<ViolationRule> {
    if !names.contains(target) {
        Some(ViolationRule::UnknownOperation)
    } else if !callable.contains(target) {
        Some(ViolationRule::BindingSourceNotCallable)
    } else {
        None
    }
}

fn binding_cycle_operations<'a>(
    overlay: &'a ServiceOverlay,
    callable: &'a [CallableOperation],
) -> BTreeSet<String> {
    stages::cycles(overlay, callable)
}

pub(super) fn violation(
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
