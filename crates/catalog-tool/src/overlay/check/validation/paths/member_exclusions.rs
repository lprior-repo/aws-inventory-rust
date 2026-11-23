use super::*;

pub(super) fn validate(
    service: &str,
    operation: &str,
    item: &OperationOverlay,
    model: &Model<'_>,
    violations: &mut Vec<Violation>,
) {
    check_paths(
        service,
        operation,
        "exclude_response_members",
        &item.exclude_response_members,
        model,
        root_selection(model, operation).as_ref(),
        violations,
    );
    check_count(service, operation, item, violations);
}

fn check_count(
    service: &str,
    operation: &str,
    item: &OperationOverlay,
    violations: &mut Vec<Violation>,
) {
    if item.exclude_response_members.len() > 8 {
        violations.push(violation(
            service,
            operation,
            "exclude_response_members",
            "more than 8 paths",
            ViolationRule::InvalidPath,
        ));
    }
}
