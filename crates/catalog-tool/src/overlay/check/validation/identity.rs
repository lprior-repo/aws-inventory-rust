use super::paths::violation;
use super::{Violation, ViolationRule};
use crate::{
    overlay::Identity,
    plans::{self, Model},
};

pub(super) fn validate_identity(
    service: &str,
    operation: &str,
    identity: Option<&Identity>,
    model: &Model<'_>,
    item: Option<crate::plans::model::ShapeRef<'_>>,
    violations: &mut Vec<Violation>,
) -> bool {
    let Some(item) = item else {
        return identity_failure(service, operation, "item selection unavailable", violations);
    };
    let Some(identity) = identity else {
        return identity_failure(service, operation, "missing", violations);
    };
    identity_paths(service, operation, model, item, identity, violations)
}

fn identity_failure(
    service: &str,
    operation: &str,
    value: &str,
    violations: &mut Vec<Violation>,
) -> bool {
    violations.push(violation(
        service,
        operation,
        "identity",
        value,
        ViolationRule::IncompletePlan,
    ));
    false
}

fn identity_paths(
    service: &str,
    operation: &str,
    model: &Model<'_>,
    item: crate::plans::model::ShapeRef<'_>,
    identity: &Identity,
    violations: &mut Vec<Violation>,
) -> bool {
    let paths = match identity {
        Identity::Arn(path) => vec![path.as_str()],
        Identity::Key(paths) => paths.iter().map(|path| path.as_str()).collect(),
        Identity::Template(template) => template_paths(template),
    };
    let mut valid = true;
    for path in &paths {
        let path_valid = validate_identity_path(service, operation, model, item, path, violations);
        valid &= path_valid;
    }
    valid
}

fn validate_identity_path(
    service: &str,
    operation: &str,
    model: &Model<'_>,
    item: crate::plans::model::ShapeRef<'_>,
    path: &str,
    violations: &mut Vec<Violation>,
) -> bool {
    let valid = model.resolve_path_from(item, path).is_ok_and(|shape| {
        matches!(
            plans::model::shape_type(shape.value),
            Some("string" | "integer" | "boolean")
        )
    });
    if !valid {
        violations.push(violation(
            service,
            operation,
            "identity",
            path,
            ViolationRule::InvalidPath,
        ));
    }
    valid
}

fn template_paths(template: &str) -> Vec<&str> {
    template
        .split('{')
        .skip(1)
        .filter_map(|part| part.split_once('}').map(|(path, _)| path))
        .filter(|path| !matches!(*path, "partition" | "account" | "region"))
        .collect()
}
