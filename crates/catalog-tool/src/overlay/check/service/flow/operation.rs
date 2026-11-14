use super::super::super::{report, validation};
use super::*;
use crate::overlay::OperationOverlay;

type OperationOutput = (
    Option<CallableOperation>,
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

fn package_operation(
    context: OperationContext<'_, '_, '_>,
    result: validation::OperationResult,
) -> (
    Option<CallableOperation>,
    (String, String, String, String),
    Vec<Violation>,
) {
    let callable = result.callable;
    let errors = package_errors(
        &context,
        result.effective,
        result.violations,
        callable.is_some(),
    );
    let row = package_row(&context, result.effective);
    (callable, row, errors)
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
