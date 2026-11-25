use super::{encode, selected_path};
use crate::{
    overlay::{ContextSource, MissingSelected, RowOperand, RowPolicy, RowPredicate, RowValueType},
    runtime_catalog::{RuntimeCatalogCompileError, ServiceIndex},
};
use inventory_core::{
    catalog::{Operation, Service, Target},
    record::Value,
};

pub(super) fn compile(
    service: Service<'_>,
    operation: Operation<'_>,
    policy: &RowPolicy,
    items: Option<&crate::overlay::ItemSelector>,
    checked: &crate::overlay::CheckedOverlays,
    service_name: &str,
    services: &ServiceIndex<'_>,
) -> Result<Option<(u32, Value)>, RuntimeCatalogCompileError> {
    validate_assertion_count(policy)?;
    if is_empty_policy(policy) {
        return Ok(None);
    }
    let selected = resolve_selected(service, operation, policy, items)?;
    let owned = compile_owned(service, operation, policy, selected, services)?;
    let assertions = compile_assertions(service, operation, policy, selected, services)?;
    let ctx = RowPolicyCompileCtx {
        service,
        service_name,
        checked,
        services,
    };
    let child = compile_child(&ctx, policy)?;
    Ok(Some(encode_row_policy(assertions, owned, child)?))
}

fn validate_assertion_count(policy: &RowPolicy) -> Result<(), RuntimeCatalogCompileError> {
    if policy.assertions.len() > 8 {
        return Err(RuntimeCatalogCompileError::Path {
            path: "row policy assertions count".to_owned(),
        });
    }
    Ok(())
}

fn is_empty_policy(policy: &RowPolicy) -> bool {
    policy.owned.is_none() && policy.assertions.is_empty() && policy.child.is_none()
}

fn resolve_selected(
    service: Service<'_>,
    operation: Operation<'_>,
    policy: &RowPolicy,
    items: Option<&crate::overlay::ItemSelector>,
) -> Result<Target, RuntimeCatalogCompileError> {
    if policy.owned.is_none() && policy.assertions.is_empty() {
        return Ok(operation.output()?);
    }
    super::super::selection::target(service, operation.output()?, items)
}

fn compile_owned(
    service: Service<'_>,
    operation: Operation<'_>,
    policy: &RowPolicy,
    selected: Target,
    services: &ServiceIndex<'_>,
) -> Result<Option<Value>, RuntimeCatalogCompileError> {
    policy
        .owned
        .as_ref()
        .map(|predicate| predicate_value(service, operation, selected, predicate, services))
        .transpose()
}

fn compile_assertions(
    service: Service<'_>,
    operation: Operation<'_>,
    policy: &RowPolicy,
    selected: Target,
    services: &ServiceIndex<'_>,
) -> Result<Vec<Value>, RuntimeCatalogCompileError> {
    policy
        .assertions
        .iter()
        .map(|predicate| predicate_value(service, operation, selected, predicate, services))
        .collect()
}

struct RowPolicyCompileCtx<'a> {
    service: Service<'a>,
    service_name: &'a str,
    checked: &'a crate::overlay::CheckedOverlays,
    services: &'a ServiceIndex<'a>,
}

fn compile_child(
    ctx: &RowPolicyCompileCtx<'_>,
    policy: &RowPolicy,
) -> Result<Option<Value>, RuntimeCatalogCompileError> {
    policy
        .child
        .as_ref()
        .map(|child| compile_child_policy(ctx, child))
        .transpose()
}

fn compile_child_policy(
    ctx: &RowPolicyCompileCtx<'_>,
    child: &crate::overlay::ChildEligibility,
) -> Result<Value, RuntimeCatalogCompileError> {
    let parent = find_parent_service(ctx.checked, ctx.service_name, &child.operation)?;
    let parent_operation = super::catalog_operation(ctx.service, child.operation.as_str())?;
    let selected = super::super::selection::target(
        ctx.service,
        parent_operation.output()?,
        parent.plan().items.as_ref(),
    )?;
    let predicate = predicate_value(
        ctx.service,
        parent_operation,
        selected,
        &child.predicate,
        ctx.services,
    )?;
    encode::map([
        (1, Value::Text(child.operation.to_string())),
        (2, predicate),
    ])
}

fn find_parent_service<'a>(
    checked: &'a crate::overlay::CheckedOverlays,
    service_name: &'a str,
    operation_name: &'a crate::overlay::OperationName,
) -> Result<&'a crate::overlay::CallableOperation, RuntimeCatalogCompileError> {
    checked
        .services()
        .iter()
        .filter(|review| review.service() == service_name)
        .flat_map(|review| review.callable())
        .find(|parent| parent.operation() == operation_name.as_str())
        .ok_or_else(|| RuntimeCatalogCompileError::Path {
            path: operation_name.as_str().to_owned(),
        })
}

fn encode_row_policy(
    assertions: Vec<Value>,
    owned: Option<Value>,
    child: Option<Value>,
) -> Result<(u32, Value), RuntimeCatalogCompileError> {
    let fields = [(2, Value::List(assertions))];
    Ok((
        22,
        encode::map(
            fields
                .into_iter()
                .chain(owned.map(|value| (1, value)))
                .chain(child.map(|value| (3, value))),
        )?,
    ))
}

fn predicate_value(
    service: Service<'_>,
    operation: Operation<'_>,
    selected: Target,
    predicate: &RowPredicate,
    services: &ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    match predicate {
        RowPredicate::Absent(operand) => encode::map([
            (1, Value::Int(0)),
            (
                2,
                operand_value(service, operation, selected, operand, services)?,
            ),
        ]),
        RowPredicate::BooleanEquals { operand, value } => {
            if operand.value_type != RowValueType::Boolean {
                return Err(RuntimeCatalogCompileError::Path {
                    path: operand.path.as_str().to_owned(),
                });
            }
            encode::map([
                (1, Value::Int(1)),
                (
                    2,
                    operand_value(service, operation, selected, operand, services)?,
                ),
                (3, Value::Bool(*value)),
            ])
        }
        RowPredicate::SelectedRequestEquals {
            selected: path,
            request,
            value_type,
            missing,
        } => {
            if *value_type != RowValueType::String {
                return Err(RuntimeCatalogCompileError::Path {
                    path: path.as_str().to_owned(),
                });
            }
            validate(service, selected, path.as_str(), *value_type)?;
            validate(service, operation.input()?, request.as_str(), *value_type)?;
            encode::map([
                (1, Value::Int(2)),
                (2, selected_path(services, path.as_str())?),
                (3, selected_path(services, request.as_str())?),
                (4, Value::Int(type_tag(*value_type))),
                (
                    5,
                    Value::Int(match missing {
                        MissingSelected::Reject => 0,
                        MissingSelected::AllowAbsent => 1,
                    }),
                ),
            ])
        }
    }
}

fn operand_value(
    service: Service<'_>,
    operation: Operation<'_>,
    selected: Target,
    operand: &RowOperand,
    services: &ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let target = match operand.source {
        ContextSource::SelectedItem => selected,
        ContextSource::Request => operation.input()?,
        ContextSource::Response => operation.output()?,
    };
    validate(service, target, operand.path.as_str(), operand.value_type)?;
    encode::map([
        (1, Value::Int(super::selection::source_tag(operand.source))),
        (2, selected_path(services, operand.path.as_str())?),
        (3, Value::Int(type_tag(operand.value_type))),
    ])
}

fn validate(
    service: Service<'_>,
    target: Target,
    path: &str,
    value_type: RowValueType,
) -> Result<(), RuntimeCatalogCompileError> {
    let target = if path == "$" {
        target
    } else {
        super::super::model_path::resolve(service, target, path)?.1
    };
    let kind = super::super::model_path::kind(service, target)?;
    match (value_type, kind) {
        (RowValueType::String, "string" | "enum")
        | (RowValueType::Boolean, "boolean")
        | (RowValueType::Structure, "structure" | "union") => Ok(()),
        _ => Err(RuntimeCatalogCompileError::Path {
            path: path.to_owned(),
        }),
    }
}

fn type_tag(value: RowValueType) -> i64 {
    match value {
        RowValueType::String => 0,
        RowValueType::Boolean => 1,
        RowValueType::Structure => 2,
    }
}
