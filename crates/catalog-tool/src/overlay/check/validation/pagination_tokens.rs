use super::paths::violation;
use super::{Violation, ViolationRule, compatibility};
use crate::plans;

pub(super) fn validate_token_pair(
    service: &str,
    operation: &str,
    model: &plans::Model<'_>,
    input: &str,
    output: &str,
    violations: &mut Vec<Violation>,
) -> bool {
    let (input_shape, output_shape) =
        resolve_token_pair(service, operation, model, input, output, violations);
    check_token_shapes(
        service,
        operation,
        model,
        input,
        input_shape,
        output_shape,
        violations,
    )
}

pub(super) fn resolve_token_pair<'a>(
    service: &str,
    operation: &str,
    model: &'a plans::Model<'_>,
    input: &str,
    output: &str,
    violations: &mut Vec<Violation>,
) -> (
    Option<plans::model::ShapeRef<'a>>,
    Option<plans::model::ShapeRef<'a>>,
) {
    let [input_shape, output_shape] = [
        ("paginator.input", input, "input"),
        ("paginator.output", output, "output"),
    ]
    .map(|(key, path, side)| {
        resolve_token_path(service, operation, model, key, path, side, violations)
    });
    (input_shape, output_shape)
}

fn resolve_token_path<'a>(
    service: &str,
    operation: &str,
    model: &'a plans::Model<'_>,
    key: &str,
    path: &str,
    side: &str,
    violations: &mut Vec<Violation>,
) -> Option<plans::model::ShapeRef<'a>> {
    match model.resolve_path(operation, side, path) {
        Ok(Some(shape)) => Some(shape),
        _ => {
            violations.push(violation(
                service,
                operation,
                key,
                path,
                ViolationRule::InvalidPath,
            ));
            None
        }
    }
}

pub(super) fn check_token_shapes(
    service: &str,
    operation: &str,
    model: &plans::Model<'_>,
    input: &str,
    input_shape: Option<plans::model::ShapeRef<'_>>,
    output_shape: Option<plans::model::ShapeRef<'_>>,
    violations: &mut Vec<Violation>,
) -> bool {
    let (Some(input_shape), Some(output_shape)) = (input_shape, output_shape) else {
        return false;
    };
    let valid = token_shapes_valid(model, &input_shape, &output_shape);
    if !valid {
        invalid_token(service, operation, input, violations);
    }
    valid
}

fn token_shapes_valid(
    model: &plans::Model<'_>,
    input: &plans::model::ShapeRef<'_>,
    output: &plans::model::ShapeRef<'_>,
) -> bool {
    supported_token(input)
        && supported_token(output)
        && compatibility::compatible(model, input, output)
}

pub(super) fn invalid_token(
    service: &str,
    operation: &str,
    input: &str,
    violations: &mut Vec<Violation>,
) {
    violations.push(violation(
        service,
        operation,
        "paginator.token",
        input,
        ViolationRule::InvalidValue,
    ));
}

fn supported_token(shape: &plans::model::ShapeRef<'_>) -> bool {
    matches!(
        plans::model::shape_type(shape.value),
        Some(
            "string"
                | "enum"
                | "map"
                | "boolean"
                | "integer"
                | "long"
                | "short"
                | "byte"
                | "intEnum"
        )
    ) && shape
        .value
        .get("traits")
        .and_then(|traits| traits.get("smithy.api#streaming"))
        .is_none()
}
