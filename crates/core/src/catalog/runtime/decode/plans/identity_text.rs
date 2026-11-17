use super::{RuntimeCatalogError, tags};
use crate::{
    catalog::{Operation, Service, Target, ValueConstraints, runtime::{ContextSource, IdentityChoicePlan, IdentityFieldPlan, IdentityLeafPlan, IdentityPlan, ItemPlan, SelectionKind}},
    record::FieldId,
};
use super::super::super::types::{IdentityPatternCache, IdentityTextOperand, IdentityTextPlan};

enum OperandKind { Text, MapValue, ListElement }
struct Operand<'p> { source: ContextSource, path: &'p [FieldId], kind: OperandKind }

pub(super) fn compile<'a>(
    service: Service<'a>, operation: Operation<'a>, items: &ItemPlan, identity: &IdentityPlan,
    cache: &mut IdentityPatternCache<'a>,
) -> Result<IdentityTextPlan<'a>, RuntimeCatalogError> {
    let mut operands = Vec::new();
    match identity {
        IdentityPlan::Arn(path) => selected(&mut operands, path),
        IdentityPlan::Choice(IdentityChoicePlan::PreferArn { path, fallback, .. }) => {
            selected(&mut operands, path);
            leaf(&mut operands, fallback);
        }
        IdentityPlan::Choice(IdentityChoicePlan::Discriminated(branches)) => {
            for branch in branches { leaf(&mut operands, &branch.identity); }
        }
        IdentityPlan::CorrelatedKey(fields) => fields_operands(&mut operands, fields),
        IdentityPlan::MapArn { path, .. } => operands.push(Operand { source: ContextSource::SelectedItem, path, kind: OperandKind::MapValue }),
        IdentityPlan::Template(_) | IdentityPlan::Key(_) => {},
    }
    if operands.is_empty() { return Ok(IdentityTextPlan::default()); }
    let selected_root = if operands.iter().any(|operand| operand.source == ContextSource::SelectedItem) {
        Some(selected_model(service, operation, items)?)
    } else { None };
    let compiled = operands.into_iter().map(|operand| {
        let (root, root_member) = match operand.source {
            ContextSource::SelectedItem => selected_root.ok_or(RuntimeCatalogError::Capability)?,
            ContextSource::Request => (operation.input()?, None),
            ContextSource::Response => return Err(RuntimeCatalogError::Capability),
        };
        let (mut target, mut member) = resolve(service, root, operand.path, root_member)?;
        match operand.kind {
            OperandKind::Text => {},
            OperandKind::MapValue => {
                let map = tags::shape(service, target)?;
                if map.kind()? != "map" { return Err(RuntimeCatalogError::Capability); }
                target = map.map_value()?.ok_or(RuntimeCatalogError::Capability)?;
                member = Some(map.map_value_constraints()?);
            },
            OperandKind::ListElement => {
                let list = tags::shape(service, target)?;
                if list.kind()? != "list" { return Err(RuntimeCatalogError::Capability); }
                target = list.element()?.ok_or(RuntimeCatalogError::Capability)?;
                member = Some(list.element_constraints()?);
            },
        }
        let shape = match target {
            Target::Shape(index) => Some(service.shape(index)?.ok_or(RuntimeCatalogError::Capability)?),
            Target::Prelude("string") => None,
            _ => return Err(RuntimeCatalogError::Capability),
        };
        IdentityTextOperand::compile(operand.source, operand.path, shape, member, cache)
    }).collect::<Result<Vec<_>, RuntimeCatalogError>>()?;
    Ok(IdentityTextPlan::new(compiled))
}

fn selected<'p>(operands: &mut Vec<Operand<'p>>, path: &'p [FieldId]) {
    operands.push(Operand { source: ContextSource::SelectedItem, path, kind: OperandKind::Text });
}

fn leaf<'p>(operands: &mut Vec<Operand<'p>>, identity: &'p IdentityLeafPlan) {
    match identity {
        IdentityLeafPlan::Arn(path) => selected(operands, path),
        IdentityLeafPlan::Fields(fields) => fields_operands(operands, fields),
        IdentityLeafPlan::Template(_) | IdentityLeafPlan::Key(_) => {},
    }
}

fn fields_operands<'p>(operands: &mut Vec<Operand<'p>>, fields: &'p [IdentityFieldPlan]) {
    for field in fields {
        match field {
            IdentityFieldPlan::SelectedArn(path) => selected(operands, path),
            IdentityFieldPlan::RequestArn(path) => operands.push(Operand { source: ContextSource::Request, path, kind: OperandKind::Text }),
            IdentityFieldPlan::SelectedStringList { path, .. } =>
                operands.push(Operand { source: ContextSource::SelectedItem, path, kind: OperandKind::ListElement }),
            _ => {},
        }
    }
}

fn resolve<'a>(
    service: Service<'a>, root: Target, path: &[FieldId], root_member: Option<ValueConstraints<'a>>,
) -> Result<(Target, Option<ValueConstraints<'a>>), RuntimeCatalogError> {
    if path.len() > 8 { return Err(RuntimeCatalogError::Bounds); }
    let Some((last, prefix)) = path.split_last() else { return Ok((root, root_member)); };
    let parent = tags::resolve(service, root, prefix)?;
    let member = tags::member(tags::shape(service, parent)?, *last)?;
    Ok((member.target, Some(member.value_constraints()?)))
}

fn selected_model<'a>(service: Service<'a>, operation: Operation<'a>, items: &ItemPlan)
    -> Result<(Target, Option<ValueConstraints<'a>>), RuntimeCatalogError>
{
    let selected = tags::selected_item(service, operation, items)?;
    let (path, kind) = match items {
        ItemPlan::Root => return Ok((selected, None)),
        ItemPlan::Path(path) => (path.as_ref(), None),
        ItemPlan::Select { path, kind } => (path.as_ref(), Some(*kind)),
    };
    let (container, member) = resolve(service, operation.output()?, path, None)?;
    let constraints = match kind {
        Some(SelectionKind::MapKeys) => Some(tags::shape(service, container)?.map_key_constraints()?),
        Some(SelectionKind::MapEntries | SelectionKind::MapValues) => Some(tags::shape(service, container)?.map_value_constraints()?),
        Some(SelectionKind::List) => Some(tags::shape(service, container)?.element_constraints()?),
        None if tags::target_kind(service, container)? == "list" => Some(tags::shape(service, container)?.element_constraints()?),
        _ => member,
    };
    Ok((selected, constraints))
}
