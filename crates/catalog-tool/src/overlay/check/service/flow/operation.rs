use super::super::super::{report, validation};
use super::*;
use crate::{
    classes::{Class, DenyReason},
    overlay::{OperationOverlay, OverlayClass},
};

type OperationOutput = (
    Option<CallableOperation>,
    ReviewedDecision,
    (String, String, String, String),
    Vec<Violation>,
);
pub(super) fn check_operation(
    overlay: &ServiceOverlay,
    model: &Model<'_>,
    deny: &DenyList<'_>,
    vocabulary: &plans::Vocabulary,
    name: &str,
    class: Class,
    plans: &BTreeMap<String, OperationOverlay>,
) -> OperationOutput {
    let context = OperationContext {
        overlay,
        model,
        deny,
        vocabulary,
        name,
        class,
        plan: plans.get(name),
    };
    let result = operation_result(&context, plans);
    package_operation(context, result)
}

fn operation_result(
    context: &OperationContext<'_, '_, '_>,
    plans: &BTreeMap<String, OperationOverlay>,
) -> validation::OperationResult {
    validation::operation(validation::StateArgs {
        service: context.overlay.service(),
        name: context.name,
        item: context.plan.cloned(),
        class: context.class,
        model: context.model,
        deny: context.deny,
        vocabulary: context.vocabulary,
        overlays: plans,
    })
}

struct OperationContext<'a, 'm, 'd> {
    overlay: &'a ServiceOverlay,
    model: &'a Model<'m>,
    deny: &'a DenyList<'d>,
    vocabulary: &'a plans::Vocabulary,
    name: &'a str,
    class: Class,
    plan: Option<&'a OperationOverlay>,
}

type PackageRow = (String, String, String, String);

fn package_operation(
    context: OperationContext<'_, '_, '_>,
    result: validation::OperationResult,
) -> (
    Option<CallableOperation>,
    ReviewedDecision,
    PackageRow,
    Vec<Violation>,
) {
    let callable = result.callable;
    let decision = reviewed_decision(&context, result.effective, callable.is_some());
    let errors = package_errors(
        &context,
        result.effective,
        result.violations,
        callable.is_some(),
    );
    let row = package_row(&context, result.effective);
    (callable, decision, row, errors)
}

fn reviewed_decision(
    context: &OperationContext<'_, '_, '_>,
    effective: &str,
    callable: bool,
) -> ReviewedDecision {
    let disposition = if callable {
        ReviewDisposition::Callable
    } else if effective == "deferred" {
        deferred_disposition(context)
    } else if effective == "denied" {
        denied_disposition(context)
    } else {
        ReviewDisposition::Unclassified
    };
    ReviewedDecision {
        operation: context.name.to_owned(),
        disposition,
    }
}

fn deferred_disposition(context: &OperationContext<'_, '_, '_>) -> ReviewDisposition {
    context
        .plan
        .and_then(|plan| plan.defer_reason.as_ref())
        .map_or(ReviewDisposition::Unclassified, |reason| {
            ReviewDisposition::Deferred {
                reason: reason.clone(),
            }
        })
}

fn denied_disposition(context: &OperationContext<'_, '_, '_>) -> ReviewDisposition {
    deny_reason(context).map_or(ReviewDisposition::Unclassified, |reason| {
        ReviewDisposition::Denied { reason }
    })
}

fn deny_reason(context: &OperationContext<'_, '_, '_>) -> Option<String> {
    let reason = context.plan.and_then(|plan| match plan.class {
        Some(OverlayClass::Deny(reason)) => Some(reason),
        _ => None,
    });
    reason
        .or(match context.class {
            Class::Denied(reason) => Some(reason),
            _ => None,
        })
        .map(deny_reason_text)
}

fn deny_reason_text(reason: DenyReason) -> String {
    match reason {
        DenyReason::Content => "content",
        DenyReason::Secret => "secret",
        DenyReason::Paid => "paid",
        DenyReason::Mutating => "mutating",
        DenyReason::ReferenceData => "reference-data",
    }
    .to_owned()
}

fn package_errors(
    context: &OperationContext<'_, '_, '_>,
    effective: &str,
    violations: Vec<Violation>,
    callable: bool,
) -> Vec<Violation> {
    operation_errors(
        context.overlay.service(),
        context.name,
        context.class,
        effective,
        violations,
        callable,
        context.plan.is_some(),
    )
}

fn package_row(
    context: &OperationContext<'_, '_, '_>,
    effective: &str,
) -> (String, String, String, String) {
    operation_row(context, effective)
}

fn operation_errors(
    service: &str,
    operation: &str,
    class: Class,
    effective: &str,
    mut errors: Vec<Violation>,
    callable: bool,
    reviewed: bool,
) -> Vec<Violation> {
    if incomplete_review(class, effective, callable, reviewed) {
        errors.push(super::super::violation(
            service,
            operation,
            "review",
            "non-callable candidate",
            ViolationRule::IncompleteReview,
        ));
    }
    errors
}

fn incomplete_review(class: Class, effective: &str, callable: bool, reviewed: bool) -> bool {
    !callable
        && matches!(
            class,
            Class::Enumerate | Class::Expand | Class::Unclassified
        )
        && (!reviewed || effective == "unclassified")
        && !matches!(effective, "denied" | "deferred")
}

fn operation_row(
    context: &OperationContext<'_, '_, '_>,
    effective: &str,
) -> (String, String, String, String) {
    let source = report::plan_sources(
        context.plan,
        context.class,
        effective,
        context.model,
        context.model.operation_id(context.name),
        context.deny,
        context.vocabulary,
    );
    (
        context.overlay.service().to_owned(),
        context.name.to_owned(),
        effective.to_owned(),
        source,
    )
}
