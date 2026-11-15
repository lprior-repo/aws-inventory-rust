use super::{Model, Violation, check_token_shapes, plans, resolve_token_pair, validate_truncated};
use crate::overlay::MemberPath;

pub(super) fn validate(
    service: &str,
    operation: &str,
    model: &Model<'_>,
    paths: (&MemberPath, &MemberPath, &MemberPath),
    violations: &mut Vec<Violation>,
) -> bool {
    let (input, output, truncated) = paths;
    let (request, response) = resolve_token_pair(
        service, operation, model, input.as_str(), output.as_str(), violations,
    );
    let element = response.and_then(|shape| {
        let element = (plans::model::shape_type(shape.value) == Some("list"))
            .then(|| shape.value.get("member").and_then(plans::model::target))
            .flatten()
            .and_then(|target| model.shape(target));
        if element.is_none() {
            super::invalid_token(service, operation, output.as_str(), violations);
        }
        element
    });
    let valid_token = check_token_shapes(
        service, operation, model, input.as_str(), request, element, violations,
    );
    let valid_flag = validate_truncated(service, operation, model, Some(truncated), violations);
    valid_token && valid_flag
}
