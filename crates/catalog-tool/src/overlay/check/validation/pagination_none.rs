use super::paths::violation;
use super::{Violation, ViolationRule};

/// Validate that `Paginator::None` is not used on operations with modeled pagination.
/// Returns `true` if the overlay paginator is valid for the model; false otherwise.
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