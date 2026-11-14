mod flow;
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
    let mut edges = overlay_binding_edges(overlay);
    add_callable_edges(&mut edges, callable);
    find_cycle_operations(&edges)
}

fn overlay_binding_edges(overlay: &ServiceOverlay) -> BTreeMap<&str, Vec<&str>> {
    overlay
        .operations()
        .iter()
        .map(|(name, plan)| (name.as_str(), binding_parents(plan.bind.values())))
        .collect()
}

fn add_callable_edges<'a>(
    edges: &mut BTreeMap<&'a str, Vec<&'a str>>,
    callable: &'a [CallableOperation],
) {
    for handle in callable {
        edges.insert(
            handle.operation(),
            binding_parents(handle.plan.bind.values()),
        );
    }
}

fn binding_parents<'a>(bindings: impl Iterator<Item = &'a Binding>) -> Vec<&'a str> {
    bindings
        .filter_map(|binding| match binding {
            Binding::From { operation, .. } => Some(operation.as_str()),
            _ => None,
        })
        .collect()
}

fn find_cycle_operations(edges: &BTreeMap<&str, Vec<&str>>) -> BTreeSet<String> {
    let mut cycle = BTreeSet::new();
    for (start, targets) in edges {
        for target in targets {
            insert_cycle_if_reachable(&mut cycle, edges, start, target);
        }
    }
    cycle
}

fn insert_cycle_if_reachable(
    cycle: &mut BTreeSet<String>,
    edges: &BTreeMap<&str, Vec<&str>>,
    start: &str,
    target: &str,
) {
    if reaches(edges, target, start) {
        cycle.insert(start.to_owned());
        cycle.insert(target.to_owned());
    }
}

fn reaches(edges: &BTreeMap<&str, Vec<&str>>, start: &str, goal: &str) -> bool {
    let mut pending = vec![start];
    let mut seen = BTreeSet::new();
    while let Some(current) = pending.pop() {
        if current == goal {
            return true;
        }
        if seen.insert(current) {
            pending.extend(edges.get(current).into_iter().flatten().copied());
        }
    }
    false
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
