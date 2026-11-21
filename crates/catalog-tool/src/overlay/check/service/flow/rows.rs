use super::*;

pub(super) fn append_cycles(
    overlay: &ServiceOverlay,
    callable: &[CallableOperation],
    errors: &mut Vec<Violation>,
) {
    for operation in binding_cycle_operations(overlay, callable) {
        errors.push(super::super::violation(
            overlay.service(),
            &operation,
            "bind",
            &operation,
            ViolationRule::BindingCycle,
        ));
    }
}

pub(super) fn finish(
    overlay: &ServiceOverlay,
    mut callable: Vec<CallableOperation>,
    mut decisions: Vec<ReviewedDecision>,
    rows: Vec<ReportRow>,
    mut errors: Vec<Violation>,
) -> ServiceCheck {
    callable.sort_by(|left, right| left.operation.cmp(&right.operation));
    decisions.sort_by(|left, right| left.operation.cmp(&right.operation));
    errors.sort();
    errors.dedup();
    (
        ReviewedService {
            service: overlay.service().to_owned(),
            reviewer: overlay.reviewer().to_owned(),
            reviewed: overlay.reviewed().to_owned(),
            evidence: *overlay.evidence(),
            callable,
            decisions,
        },
        rows,
        errors,
    )
}
