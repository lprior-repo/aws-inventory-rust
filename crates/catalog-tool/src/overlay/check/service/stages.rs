use super::{Binding, CallableOperation, ServiceOverlay, Violation, ViolationRule};
use crate::overlay::{ExecutionBranch, OperationOverlay};
use std::collections::{BTreeMap, BTreeSet};

type Stage<'a> = (&'a str, Option<&'a str>);

fn plans<'a>(
    overlay: &'a ServiceOverlay,
    callable: &'a [CallableOperation],
) -> BTreeMap<&'a str, &'a OperationOverlay> {
    overlay
        .operations()
        .iter()
        .map(|(name, plan)| (name.as_str(), plan))
        .chain(
            callable
                .iter()
                .map(|handle| (handle.operation(), &handle.plan)),
        )
        .collect()
}
fn branches(plan: &OperationOverlay) -> impl Iterator<Item = Option<&ExecutionBranch>> {
    plan.branches
        .iter()
        .map(Some)
        .chain(std::iter::once(None).filter(|_| plan.branches.is_empty()))
}
fn effective<'a>(
    plan: &'a OperationOverlay,
    branch: Option<&'a ExecutionBranch>,
) -> impl Iterator<Item = &'a Binding> {
    plan.bind
        .iter()
        .filter(move |(path, _)| {
            !branch.is_some_and(|branch| {
                branch.bind.contains_key(*path) || branch.fixed.contains_key(*path)
            })
        })
        .map(|(_, binding)| binding)
        .chain(branch.into_iter().flat_map(|branch| branch.bind.values()))
}
fn source(binding: &Binding) -> &Binding {
    match binding {
        Binding::Transformed { binding, .. } => binding,
        _ => binding,
    }
}

pub(super) fn references(
    overlay: &ServiceOverlay,
    callable: &[CallableOperation],
    errors: &mut Vec<Violation>,
) {
    let plans = plans(overlay, callable);
    for (operation, plan) in &plans {
        for branch in branches(plan) {
            for binding in effective(plan, branch) {
                if let Binding::FromBranch {
                    operation: producer,
                    branch,
                    ..
                }
                | Binding::ContextFromBranch {
                    operation: producer,
                    branch,
                    ..
                } = source(binding)
                    && !plans.get(producer.as_str()).is_some_and(|plan| {
                        plan.branches
                            .iter()
                            .any(|stage| stage.discriminator == *branch)
                    })
                {
                    errors.push(super::violation(
                        overlay.service(),
                        operation,
                        "bind.from_branch",
                        branch,
                        ViolationRule::InvalidValue,
                    ));
                }
            }
        }
    }
}

pub(super) fn cycles(overlay: &ServiceOverlay, callable: &[CallableOperation]) -> BTreeSet<String> {
    let plans = plans(overlay, callable);
    let mut edges = BTreeMap::<Stage<'_>, Vec<Stage<'_>>>::new();
    for (operation, plan) in &plans {
        for branch in branches(plan) {
            let node = (
                *operation,
                branch.map(|branch| branch.discriminator.as_str()),
            );
            let mut parents = Vec::new();
            for binding in effective(plan, branch) {
                match source(binding) {
                    Binding::FromBranch {
                        operation, branch, ..
                    }
                    | Binding::ContextFromBranch {
                        operation, branch, ..
                    } => {
                        parents.push((operation.as_str(), Some(branch.as_str())));
                    }
                    Binding::FromAny(alternatives) => {
                        for alternative in alternatives {
                            add(&plans, alternative.operation.as_str(), &mut parents);
                        }
                    }
                    Binding::From { operation, .. }
                    | Binding::FromArn { operation, .. }
                    | Binding::FromUrl { operation, .. }
                    | Binding::Context { operation, .. } => {
                        add(&plans, operation.as_str(), &mut parents)
                    }
                    _ => {}
                }
            }
            edges.insert(node, parents);
        }
    }
    let mut cycles = BTreeSet::new();
    for (start, parents) in &edges {
        for parent in parents {
            if reaches(&edges, *parent, *start) {
                cycles.insert(start.0.to_owned());
                cycles.insert(parent.0.to_owned());
            }
        }
    }
    cycles
}
fn add<'a>(
    plans: &BTreeMap<&'a str, &'a OperationOverlay>,
    operation: &'a str,
    parents: &mut Vec<Stage<'a>>,
) {
    if let Some(plan) = plans.get(operation) {
        parents.extend(
            branches(plan)
                .map(|branch| (operation, branch.map(|stage| stage.discriminator.as_str()))),
        );
    }
}
fn reaches<'a>(
    edges: &BTreeMap<Stage<'a>, Vec<Stage<'a>>>,
    start: Stage<'a>,
    goal: Stage<'a>,
) -> bool {
    let mut pending = vec![start];
    let mut seen = BTreeSet::new();
    while let Some(current) = pending.pop() {
        if current == goal {
            return true;
        }
        if seen.insert(current) {
            pending.extend(edges.get(&current).into_iter().flatten().copied());
        }
    }
    false
}
