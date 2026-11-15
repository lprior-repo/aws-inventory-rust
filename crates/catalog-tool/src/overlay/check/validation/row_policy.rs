use super::{paths, Violation, ViolationRule};
use crate::{overlay::{Binding, ContextSource, OperationOverlay, RowOperand, RowPolicy, RowPredicate, RowValueType}, plans::{Model, Vocabulary, model::{ShapeRef, shape_type}}};
use std::collections::BTreeMap;

pub(super) fn validate(service: &str, operation: &str, overlay: &OperationOverlay,
    model: &Model<'_>, vocabulary: &Vocabulary, overlays: &BTreeMap<String,OperationOverlay>,
    violations: &mut Vec<Violation>) {
    let valid = valid_policy(operation,&overlay.row_policy,overlay,model,vocabulary,overlays,
        overlay.bind.values().chain(overlay.branches.iter().flat_map(|branch| branch.bind.values())))
        && overlay.branches.iter().all(|branch| branch.row_policy.as_ref().is_none_or(|policy| {
            let consistent_parent = policy.child.as_ref().is_none_or(|child| overlay.row_policy.child.as_ref()
                .is_none_or(|base| base.operation == child.operation));
            consistent_parent && valid_policy(operation,policy,overlay,model,vocabulary,overlays,
                overlay.bind.values().chain(branch.bind.values()))
        }));
    if !valid { violations.push(paths::violation(service,operation,"row_policy","invalid bounded modeled row policy",ViolationRule::InvalidPath)); }
}

fn valid_policy<'a>(operation: &str, policy: &RowPolicy, overlay: &OperationOverlay,
    model: &Model<'_>, vocabulary: &Vocabulary, overlays: &BTreeMap<String,OperationOverlay>,
    mut bindings: impl Iterator<Item = &'a Binding>) -> bool {
    if policy.owned.is_none() && policy.assertions.is_empty() && policy.child.is_none() { return true; }
    let selected = if policy.owned.is_none() && policy.assertions.is_empty() { None }
        else { paths::selected_for_parent(model,operation,Some(overlay),vocabulary) };
    policy.assertions.len() <= 8 && policy.owned.iter().chain(policy.assertions.iter())
        .all(|predicate| valid_predicate(model,operation,selected,predicate))
        && policy.child.as_ref().is_none_or(|child| {
            let source = child.operation.as_str();
            let selected = paths::selected_for_parent(model,source,overlays.get(source),vocabulary);
            model.operation_id(source).is_some() && bindings.any(|binding| consumes(binding,source))
                && valid_predicate(model,source,selected,&child.predicate)
        })
}

fn valid_predicate(model: &Model<'_>, operation: &str, selected: Option<ShapeRef<'_>>, predicate: &RowPredicate) -> bool {
    match predicate {
        RowPredicate::Absent(operand) => valid_operand(model,operation,selected,operand),
        RowPredicate::BooleanEquals { operand,.. } => operand.value_type == RowValueType::Boolean && valid_operand(model,operation,selected,operand),
        RowPredicate::SelectedRequestEquals { selected:path,request,value_type,.. } => *value_type == RowValueType::String
            && matches_type(model,selected,path.as_str(),*value_type)
            && matches_type(model,model.side_root(operation,"input"),request.as_str(),*value_type),
    }
}

fn valid_operand(model: &Model<'_>, operation: &str, selected: Option<ShapeRef<'_>>, operand: &RowOperand) -> bool {
    let root = match operand.source { ContextSource::SelectedItem => selected,
        ContextSource::Request => model.side_root(operation,"input"), ContextSource::Response => model.side_root(operation,"output") };
    matches_type(model,root,operand.path.as_str(),operand.value_type)
}

fn matches_type(model: &Model<'_>, root: Option<ShapeRef<'_>>, path: &str, value_type: RowValueType) -> bool {
    let target = root.and_then(|root| if path == "$" { Some(root) } else {
        path.split('.').try_fold(root,|current,segment| {
            if !matches!(shape_type(current.value),Some("structure" | "union")) { return None; }
            let target = current.value.get("members")?.get(segment)?.get("target")?.as_str()?;
            model.shape(target)
        })
    });
    matches!((value_type,target.and_then(|target| shape_type(target.value))),
        (RowValueType::String,Some("string" | "enum")) | (RowValueType::Boolean,Some("boolean"))
        | (RowValueType::Structure,Some("structure" | "union")))
}

fn consumes(binding: &Binding, operation: &str) -> bool {
    let binding = match binding { Binding::Transformed { binding,.. } => binding.as_ref(), binding => binding };
    match binding {
        Binding::From { operation:source,.. } | Binding::FromArn { operation:source,.. }
        | Binding::FromUrl { operation:source,.. } | Binding::Context { operation:source,.. }
        | Binding::FromBranch { operation:source,.. } | Binding::ContextFromBranch { operation:source,.. } => source.as_str() == operation,
        Binding::FromAny(alternatives) => alternatives.iter().any(|alternative| alternative.operation.as_str() == operation),
        _ => false,
    }
}
