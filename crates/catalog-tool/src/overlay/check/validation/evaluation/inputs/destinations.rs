use super::{State, ViolationRule, page_size_provided, paths, plans};
use crate::overlay::MemberPath;

pub(super) fn validate(state: &mut State<'_, '_, '_>) {
    let common = state.item.fixed.keys().chain(state.item.bind.keys())
        .chain(state.derived_bindings.keys()).map(MemberPath::as_str).collect::<Vec<_>>();
    let dynamic = dynamic_paths(state.item.paginator.as_ref(), state.pagination.as_ref(), page_size_provided(state));
    let base = common.iter().chain(&dynamic).copied().collect::<Vec<_>>();
    report(state.service, state.name, &base, &mut state.violations);
    state.item.branches.iter().for_each(|branch| {
        let overrides = branch.fixed.keys().chain(branch.bind.keys())
            .map(MemberPath::as_str).collect::<Vec<_>>();
        let effective = common.iter().copied().filter(|path| !overrides.contains(path))
            .chain(overrides.iter().copied()).chain(dynamic.iter().copied()).collect::<Vec<_>>();
        report(state.service, state.name, &effective, &mut state.violations);
    });
}

fn dynamic_paths<'a>(paginator: Option<&'a crate::overlay::Paginator>,
    pagination: Option<&'a plans::Pagination<'_>>, page_size: bool) -> Vec<&'a str> {
    let mut destinations = Vec::new();
    match paginator {
        Some(crate::overlay::Paginator::Token(pair)) => destinations.push(pair.input.as_str()),
        Some(crate::overlay::Paginator::Tokens { pairs, .. }) =>
            destinations.extend(pairs.iter().map(|pair| pair.input.as_str())),
        Some(crate::overlay::Paginator::LastListItem { input, .. }) => destinations.push(input.as_str()),
        None => {
            if let Some(plans::Pagination::Modeled { input_token, .. }) = pagination {
                destinations.push(*input_token);
            }
        }
        Some(crate::overlay::Paginator::None) => {}
    }
    if page_size {
        if let Some(plans::Pagination::Modeled { page_size: Some(path), .. }) = pagination {
            destinations.push(*path);
        }
    }
    destinations
}

fn report(service: &str, name: &str, destinations: &[&str], violations: &mut Vec<super::super::super::Violation>) {
    destinations.iter().enumerate().filter(|(index, path)| {
        destinations.iter().take(*index).any(|previous| overlapping(previous, path))
    }).for_each(|(_, path)| {
        violations.push(paths::violation(service, name, "input", path, ViolationRule::InvalidPath));
    });
}
fn overlapping(left: &str, right: &str) -> bool {
    left == right || left.strip_prefix(right).is_some_and(|suffix| suffix.starts_with('.'))
        || right.strip_prefix(left).is_some_and(|suffix| suffix.starts_with('.'))
}
