use super::paths::violation;
use super::{Violation, ViolationRule};

pub(super) fn validate_none(
    service: &str,
    operation: &str,
    has_modeled_pagination: bool,
    violations: &mut Vec<Violation>,
) -> bool {
    if has_modeled_pagination {
        violations.push(violation(
            service,
            operation,
            "paginator",
            "Paginator::None",
            ViolationRule::InvalidValue,
        ));
    }
    !has_modeled_pagination
}
