use super::{InputContext, State, ViolationRule, check_fixed_input, paths, validate_binding};
use crate::overlay::ExecutionBranch;
use std::collections::BTreeSet;

pub(super) fn apply(state: &mut State<'_, '_, '_>) {
    if state.item.branches.len() > 16 {
        state.violations.push(paths::violation(
            state.service,
            state.name,
            "branches",
            "count",
            ViolationRule::InvalidValue,
        ));
        return;
    }
    let mut names = BTreeSet::new();
    let shared = state.item.branches.iter().fold(None, |shared, branch| {
        let mut provided = BTreeSet::new();
        let mut context = InputContext {
            service: state.service,
            name: state.name,
            model: state.model,
            overlays: state.overlays,
            vocabulary: state.vocabulary,
            provided: &mut provided,
            violations: &mut state.violations,
        };
        validate_branch(&mut context, branch, names.insert(&branch.discriminator));
        branch
            .fixed
            .iter()
            .for_each(|(path, value)| check_fixed_input(&mut context, path, value));
        branch
            .bind
            .iter()
            .for_each(|(path, value)| validate_binding(&mut context, path, value));
        let supplied = state
            .item
            .fixed
            .keys()
            .chain(state.item.bind.keys())
            .chain(state.derived_bindings.keys())
            .chain(branch.fixed.keys())
            .chain(branch.bind.keys())
            .map(|path| path.as_str().to_owned())
            .collect();
        complete_required(&mut context, &state.provided, &supplied, state.candidate);
        Some(match shared {
            Some(shared) => provided.intersection(&shared).cloned().collect(),
            None => provided,
        })
    });
    if let Some(shared) = shared {
        state.provided.extend(shared);
    }
}

fn validate_branch(context: &mut InputContext<'_, '_>, branch: &ExecutionBranch, unique: bool) {
    let name = &branch.discriminator;
    if !unique
        || name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        context.violations.push(paths::violation(
            context.service,
            context.name,
            "branches",
            name,
            ViolationRule::InvalidValue,
        ));
    }
    if let Some(region) = &branch.request_region
        && inventory_core::inventory::RegionId::parse(region).is_err()
    {
        context.violations.push(paths::violation(
            context.service,
            context.name,
            "branches.region",
            region,
            ViolationRule::InvalidValue,
        ));
    }
}

fn complete_required(
    context: &mut InputContext<'_, '_>,
    common: &BTreeSet<String>,
    supplied: &BTreeSet<String>,
    candidate: bool,
) {
    if !candidate {
        return;
    }
    let required = match context.model.required_paths(context.name, supplied) {
        Ok(required) => required,
        Err(reason) => {
            context.violations.push(paths::violation(
                context.service,
                context.name,
                "input",
                reason,
                ViolationRule::InvalidPath,
            ));
            return;
        }
    };
    required.iter().for_each(|path| {
        let covered = common.contains(path)
            || context.provided.contains(path)
            || supplied.iter().any(|parent| {
                path.strip_prefix(parent)
                    .is_some_and(|suffix| suffix.starts_with('.'))
                    && (common.contains(parent) || context.provided.contains(parent))
            });
        if covered {
            paths::mark_provided(context.provided, path);
        } else {
            context.violations.push(paths::violation(
                context.service,
                context.name,
                "required",
                path,
                ViolationRule::IncompletePlan,
            ));
        }
    });
}
