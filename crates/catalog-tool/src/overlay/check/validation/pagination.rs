use super::paths::violation;
use super::{Violation, ViolationRule};
use crate::{
    overlay::{OperationOverlay, Paginator},
    overlay::check::validation::compatibility::compatible,
    plans::{self, Model, Pagination},
};

pub(super) fn validate_paginator(
    service: &str,
    operation: &str,
    item: &OperationOverlay,
    model: &Model<'_>,
    pagination: Option<&Pagination<'_>>,
    violations: &mut Vec<Violation>,
) -> bool {
    match item.paginator.as_ref() {
        Some(paginator) => overlay_paginator(service, operation, model, pagination, paginator, violations),
        None => modeled_paginator(service, operation, pagination, violations),
    }
}

fn overlay_paginator(
    service: &str,
    operation: &str,
    model: &Model<'_>,
    pagination: Option<&Pagination<'_>>,
    paginator: &Paginator,
    violations: &mut Vec<Violation>,
) -> bool {
    match paginator {
        Paginator::None => {
            let has_modeled_pagination =
                pagination.is_some_and(|p| matches!(p, Pagination::Modeled { .. }));
            super::pagination_none::validate_none(
                service,
                operation,
                has_modeled_pagination,
                violations,
            )
        }
        Paginator::Token(pair) => {
            validate_single_token(service, operation, model, pair, violations)
        }
        Paginator::LastListItem {
            input,
            output,
            truncated,
        } => {
            let request = resolve_request_path(model, operation, input.as_str());
            let response = model
                .resolve_path(operation, "output", output.as_str())
                .ok()
                .flatten();
            let element = response
                .as_ref()
                .filter(|shape| plans::model::shape_type(shape.value) == Some("list"))
                .and_then(|shape| shape.value.get("member"))
                .and_then(plans::model::target)
                .and_then(|target| model.shape(target));
            let valid = request
                .as_ref()
                .zip(element.as_ref())
                .is_some_and(|(request, element)| {
                    matches!(
                        plans::model::shape_type(request.value),
                        Some("string" | "enum")
                    ) && matches!(
                        plans::model::shape_type(element.value),
                        Some("string" | "enum")
                    ) && compatible(model, request, element)
                });
            if !valid {
                super::pagination_tokens::invalid_token(
                    service, operation, input.as_str(), violations,
                );
            }
            let flag_valid =
                validate_truncated(service, operation, model, Some(truncated), violations);
            valid && flag_valid
        }
        Paginator::Tokens { pairs, truncated } => validate_tokens(
            service,
            operation,
            model,
            pairs,
            truncated.as_ref(),
            violations,
        ),
    }
}

fn resolve_request_path<'m>(
    model: &Model<'m>,
    operation: &str,
    path: &str,
) -> Option<crate::plans::model::ShapeRef<'m>> {
    let root = model.side_root(operation, "input")?;
    path.split('.').try_fold(root, |shape, segment| {
        // Native request cursor ancestors must be structures, not projected rows.
        (plans::model::shape_type(shape.value) == Some("structure"))
            .then_some(shape)
            .and_then(|shape| model.resolve_path_from(shape, segment).ok())
    })
}

fn validate_single_token(
    service: &str,
    operation: &str,
    model: &Model<'_>,
    pair: &crate::overlay::TokenPair,
    violations: &mut Vec<Violation>,
) -> bool {
    super::pagination_tokens::validate_token_pair(
        service,
        operation,
        model,
        pair.input.as_str(),
        pair.output.as_str(),
        violations,
    )
}

fn validate_tokens(
    service: &str,
    operation: &str,
    model: &Model<'_>,
    pairs: &[crate::overlay::TokenPair],
    truncated: Option<&crate::overlay::MemberPath>,
    violations: &mut Vec<Violation>,
) -> bool {
    let valid_pairs = pairs
        .iter()
        .map(|pair| {
            super::pagination_tokens::validate_token_pair(
                service,
                operation,
                model,
                pair.input.as_str(),
                pair.output.as_str(),
                violations,
            )
        })
        .all(|valid| valid);
    let valid_truncated = validate_truncated(service, operation, model, truncated, violations);
    valid_pairs && valid_truncated
}

pub(super) fn validate_truncated(
    service: &str,
    operation: &str,
    model: &Model<'_>,
    truncated: Option<&crate::overlay::MemberPath>,
    violations: &mut Vec<Violation>,
) -> bool {
    let Some(path) = truncated else { return true };
    let valid = model
        .resolve_path(operation, "output", path.as_str())
        .ok()
        .flatten()
        .is_some_and(|target| plans::model::shape_type(target.value) == Some("boolean"));
    if !valid {
        violations.push(violation(
            service,
            operation,
            "truncated",
            path.as_str(),
            ViolationRule::InvalidPath,
        ));
    }
    valid
}

fn modeled_paginator(
    service: &str,
    operation: &str,
    pagination: Option<&Pagination<'_>>,
    violations: &mut Vec<Violation>,
) -> bool {
    match pagination {
        Some(Pagination::Modeled { .. }) => true,
        Some(Pagination::Unpaginated) => {
            missing_paginator(service, operation, "missing explicit none", violations)
        }
        Some(Pagination::NeedsOverlay(_)) | None => {
            missing_paginator(service, operation, "missing", violations)
        }
    }
}

fn missing_paginator(
    service: &str,
    operation: &str,
    value: &str,
    violations: &mut Vec<Violation>,
) -> bool {
    violations.push(violation(
        service,
        operation,
        "paginator",
        value,
        ViolationRule::IncompletePlan,
    ));
    false
}


