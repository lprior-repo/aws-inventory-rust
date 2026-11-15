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
    identity_paths((service, operation), model, item, identity, violations)
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
    origin: (&str, &str),
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
    let context = IdentityContext { model, item, identity };
    paths.iter().fold(true, |valid, path| {
        validate_identity_path(origin, &context, path, violations) & valid
    })
}

struct IdentityContext<'a, 'm> {
    model: &'a Model<'m>,
    item: crate::plans::model::ShapeRef<'m>,
    identity: &'a Identity,
}

fn validate_identity_path(
    origin: (&str, &str),
    context: &IdentityContext<'_, '_>,
    path: &str,
    violations: &mut Vec<Violation>,
) -> bool {
    let valid = identity_scalar(context, path);
    if !valid {
        violations.push(violation(
            origin.0,
            origin.1,
            "identity",
            path,
            ViolationRule::InvalidPath,
        ));
    }
    valid
}

fn identity_scalar(context: &IdentityContext<'_, '_>, path: &str) -> bool {
    super::paths::resolve_selected(context.model, context.item, path).is_some_and(|shape| {
        match plans::model::shape_type(shape.value) {
            Some("string" | "enum") => true,
            Some("byte" | "short" | "integer" | "long" | "boolean") => {
                !matches!(context.identity, Identity::Arn(_))
            }
            _ => false,
        }
    })
}

fn template_paths(template: &str) -> Vec<&str> {
    template
        .split('{')
        .skip(1)
        .filter_map(|part| part.split_once('}').map(|(path, _)| path))
        .filter(|path| !matches!(*path, "partition" | "account" | "region"))
        .collect()
}
