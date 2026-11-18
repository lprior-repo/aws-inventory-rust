use super::super::super::types::{IdentityPatternCache, IdentityTextOperand, IdentityTextPlan};
use super::{RuntimeCatalogError, tags};
use crate::{
    catalog::{
        Operation, Service, Target, ValueConstraints,
        runtime::{
            ContextSource, IdentityChoicePlan, IdentityFieldPlan, IdentityLeafPlan, IdentityPlan,
            ItemPlan, SelectionKind,
        },
    },
    record::FieldId,
};

pub(super) fn compile<'a>(
    service: Service<'a>,
    operation: Operation<'a>,
    items: &ItemPlan,
    identity: &IdentityPlan,
    cache: &mut IdentityPatternCache<'a>,
) -> Result<IdentityTextPlan<'a>, RuntimeCatalogError> {
    let mut lists = Vec::new();
    match identity {
        IdentityPlan::Choice(IdentityChoicePlan::PreferArn { fallback, .. }) => {
            leaf(&mut lists, fallback)
        }
        IdentityPlan::Choice(IdentityChoicePlan::Discriminated(branches)) => {
            for branch in branches {
                leaf(&mut lists, &branch.identity);
            }
        }
        IdentityPlan::CorrelatedKey(fields) => fields_lists(&mut lists, fields),
        IdentityPlan::Arn(_)
        | IdentityPlan::MapArn { .. }
        | IdentityPlan::Template(_)
        | IdentityPlan::Key(_) => {}
    }
    if lists.is_empty() {
        return Ok(IdentityTextPlan::default());
    }
    let (selected, item_member) = selected_model(service, operation, items)?;
    let compiled = lists
        .into_iter()
        .map(|path| {
            let (target, _) = resolve(service, selected, path, item_member)?;
            let list = tags::shape(service, target)?;
            if list.kind()? != "list" {
                return Err(RuntimeCatalogError::Capability);
            }
            let element = list.element()?.ok_or(RuntimeCatalogError::Capability)?;
            let shape = match element {
                Target::Shape(index) => Some(
                    service
                        .shape(index)?
                        .ok_or(RuntimeCatalogError::Capability)?,
                ),
                Target::Prelude("string") => None,
                _ => return Err(RuntimeCatalogError::Capability),
            };
            IdentityTextOperand::compile(
                ContextSource::SelectedItem,
                path,
                shape,
                Some(list.element_constraints()?),
                cache,
            )
        })
        .collect::<Result<Vec<_>, RuntimeCatalogError>>()?;
    Ok(IdentityTextPlan::new(compiled))
}

fn leaf<'p>(lists: &mut Vec<&'p [FieldId]>, identity: &'p IdentityLeafPlan) {
    if let IdentityLeafPlan::Fields(fields) = identity {
        fields_lists(lists, fields);
    }
}

fn fields_lists<'p>(lists: &mut Vec<&'p [FieldId]>, fields: &'p [IdentityFieldPlan]) {
    for field in fields {
        if let IdentityFieldPlan::SelectedStringList { path, .. } = field {
            lists.push(path);
        }
    }
}

fn resolve<'a>(
    service: Service<'a>,
    root: Target,
    path: &[FieldId],
    root_member: Option<ValueConstraints<'a>>,
) -> Result<(Target, Option<ValueConstraints<'a>>), RuntimeCatalogError> {
    if path.len() > 8 {
        return Err(RuntimeCatalogError::Bounds);
    }
    let Some((last, prefix)) = path.split_last() else {
        return Ok((root, root_member));
    };
    let parent = tags::resolve(service, root, prefix)?;
    let member = tags::member(tags::shape(service, parent)?, *last)?;
    Ok((member.target, Some(member.value_constraints()?)))
}

fn selected_model<'a>(
    service: Service<'a>,
    operation: Operation<'a>,
    items: &ItemPlan,
) -> Result<(Target, Option<ValueConstraints<'a>>), RuntimeCatalogError> {
    let selected = tags::selected_item(service, operation, items)?;
    let (path, kind) = match items {
        ItemPlan::Root => return Ok((selected, None)),
        ItemPlan::Path(path) => (path.as_ref(), None),
        ItemPlan::Select { path, kind } => (path.as_ref(), Some(*kind)),
    };
    let (container, member) = resolve(service, operation.output()?, path, None)?;
    let constraints = match kind {
        Some(SelectionKind::MapKeys) => {
            Some(tags::shape(service, container)?.map_key_constraints()?)
        }
        Some(SelectionKind::MapEntries | SelectionKind::MapValues) => {
            Some(tags::shape(service, container)?.map_value_constraints()?)
        }
        Some(SelectionKind::List) => Some(tags::shape(service, container)?.element_constraints()?),
        None if tags::target_kind(service, container)? == "list" => {
            Some(tags::shape(service, container)?.element_constraints()?)
        }
        _ => member,
    };
    Ok((selected, constraints))
}
