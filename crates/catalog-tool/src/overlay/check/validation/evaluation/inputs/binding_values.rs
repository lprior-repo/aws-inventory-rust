use super::{Binding, InputContext, Model, State, ViolationRule, pagination, paths};
use crate::{
    overlay::{BindingAlternative, ContextSource},
    plans::model::{ShapeRef, shape_type},
};
use std::collections::BTreeSet;

pub(super) fn region_list(model: &Model<'_>, target: &ShapeRef<'_>) -> bool {
    shape_type(target.value) == Some("list")
        && target
            .value
            .get("member")
            .and_then(crate::plans::model::target)
            .and_then(|id| model.shape(id))
            .is_some_and(|item| matches!(shape_type(item.value), Some("string" | "enum")))
}

pub(super) fn alternatives(
    context: &InputContext<'_, '_>,
    alternatives: &[BindingAlternative],
    target: &ShapeRef<'_>,
) -> bool {
    if alternatives.is_empty() || alternatives.len() > 8 {
        return false;
    }
    let mut seen = BTreeSet::new();
    alternatives.iter().all(|alternative| {
        seen.insert(alternative.operation.as_str())
            && alternative_valid(context, alternative, target)
    })
}

fn alternative_valid(
    context: &InputContext<'_, '_>,
    alternative: &BindingAlternative,
    target: &ShapeRef<'_>,
) -> bool {
    let operation = &alternative.operation;
    if context.model.operation_id(operation.as_str()).is_none() {
        return false;
    }
    match alternative.source {
        ContextSource::SelectedItem => super::source_binding_valid(
            context.model,
            context.overlays,
            context.vocabulary,
            operation,
            &alternative.path,
            target,
        ),
        ContextSource::Response | ContextSource::Request => {
            let side = if alternative.source == ContextSource::Response {
                "output"
            } else {
                "input"
            };
            context
                .model
                .resolve_path(operation.as_str(), side, alternative.path.as_str())
                .ok()
                .flatten()
                .is_some_and(|source| pagination::compatible(context.model, &source, target))
        }
    }
}

pub(super) fn producer_sets(state: &mut State<'_, '_, '_>) {
    if state.item.branches.is_empty() {
        return producer_sets_for(
            state.service,
            state.name,
            state.item.bind.iter(),
            &mut state.violations,
        );
    }
    state.item.branches.iter().for_each(|branch| {
        let effective = state
            .item
            .bind
            .iter()
            .filter(|(path, _)| {
                !branch.fixed.contains_key(*path) && !branch.bind.contains_key(*path)
            })
            .chain(branch.bind.iter());
        producer_sets_for(state.service, state.name, effective, &mut state.violations);
    });
}

fn producer_sets_for<'a>(
    service: &str,
    name: &str,
    bindings: impl Iterator<Item = (&'a crate::overlay::MemberPath, &'a Binding)>,
    violations: &mut Vec<super::super::Violation>,
) {
    let mut expected = None;
    bindings.for_each(|(path, binding)| {
        let Binding::FromAny(alternatives) = binding else {
            return;
        };
        let producers = alternatives
            .iter()
            .map(|alternative| alternative.operation.as_str())
            .collect::<BTreeSet<_>>();
        match &expected {
            None => expected = Some(producers),
            Some(expected) if *expected == producers => (),
            Some(_) => violations.push(paths::violation(
                service,
                name,
                "bind",
                path.as_str(),
                ViolationRule::InvalidValue,
            )),
        }
    });
}
