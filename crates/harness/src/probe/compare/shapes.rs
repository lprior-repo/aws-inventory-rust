use super::super::ProbeError;
use inventory_core::catalog::{
    Service, Shape, Target,
    runtime::{CallablePlan, ItemPlan, SelectionKind},
};
use std::collections::BTreeSet;

type Task = (Target, Target, String);

pub(super) fn missing(
    parent: &CallablePlan<'_>,
    child: &CallablePlan<'_>,
) -> Result<Option<String>, ProbeError> {
    let service = parent.service();
    let list = item(parent)?;
    let describe = item(child)?;
    if shape(service, describe)?.kind()? != "structure" {
        return Err(ProbeError::Invalid(
            "comparison child must select one structure",
        ));
    }
    let mut pending = vec![(list, describe, "$".to_owned())];
    let mut visited = BTreeSet::new();
    (0..65_536)
        .find_map(|_| {
            let Some((list, describe, path)) = pending.pop() else {
                return Some(Ok(None));
            };
            if let (Target::Shape(left), Target::Shape(right)) = (list, describe)
                && !visited.insert((left, right))
            {
                return None;
            }
            match visit(service, list, describe, &path, &mut pending) {
                Ok(Some(path)) => Some(Ok(Some(path))),
                Ok(None) => None,
                Err(error) => Some(Err(error)),
            }
        })
        .ok_or(ProbeError::Invalid(
            "comparison model traversal exceeds bound",
        ))?
}

fn visit(
    service: Service<'_>,
    list: Target,
    describe: Target,
    path: &str,
    pending: &mut Vec<Task>,
) -> Result<Option<String>, ProbeError> {
    if list == describe {
        return Ok(None);
    }
    let (Target::Shape(_), Target::Shape(_)) = (list, describe) else {
        return Ok(Some(path.to_owned()));
    };
    let list = shape(service, list)?;
    let describe = shape(service, describe)?;
    if list.kind()? != describe.kind()? {
        return Ok(Some(path.to_owned()));
    }
    describe
        .members()?
        .try_fold(None, |difference, member| {
            let member = member?;
            if difference.is_some() {
                return Ok(difference);
            }
            let matching = list
                .members()?
                .find_map(|candidate| match candidate {
                    Ok(candidate) if candidate.id == member.id => Some(Ok(candidate)),
                    Ok(_) => None,
                    Err(error) => Some(Err(error)),
                })
                .transpose()?;
            let path = format!("{path}.{}", member.name()?);
            match matching {
                Some(candidate) => {
                    push(pending, (candidate.target, member.target, path))?;
                    Ok(None)
                }
                None => Ok(Some(path)),
            }
        })
        .and_then(|difference| containers(list, describe, path, pending, difference))
}

fn containers(
    list: Shape<'_>,
    describe: Shape<'_>,
    path: &str,
    pending: &mut Vec<Task>,
    difference: Option<String>,
) -> Result<Option<String>, ProbeError> {
    if difference.is_some() {
        return Ok(difference);
    }
    [
        (list.element()?, describe.element()?),
        (list.map_key()?, describe.map_key()?),
        (list.map_value()?, describe.map_value()?),
    ]
    .into_iter()
    .try_for_each(|(list, describe)| {
        if let (Some(list), Some(describe)) = (list, describe) {
            push(pending, (list, describe, format!("{path}[]")))?;
        }
        Ok::<_, ProbeError>(())
    })?;
    Ok(None)
}

fn push(tasks: &mut Vec<Task>, task: Task) -> Result<(), ProbeError> {
    if tasks.len() >= 65_536 {
        return Err(ProbeError::Invalid("comparison model task bound exceeded"));
    }
    tasks.try_reserve(1)?;
    tasks.push(task);
    Ok(())
}

fn shape(service: Service<'_>, target: Target) -> Result<Shape<'_>, ProbeError> {
    let Target::Shape(index) = target else {
        return Err(ProbeError::Invalid("comparison requires a structure"));
    };
    service
        .shape(index)?
        .ok_or(ProbeError::Invalid("comparison shape is missing"))
}

fn item(plan: &CallablePlan<'_>) -> Result<Target, ProbeError> {
    let (path, kind) = match plan.items() {
        ItemPlan::Root => return Ok(plan.operation().output()?),
        ItemPlan::Path(path) => (path.as_ref(), None),
        ItemPlan::Select { path, kind } => (path.as_ref(), Some(kind)),
        ItemPlan::Nested { parents, items, .. } => return nested_item(plan, parents, *items),
    };
    let target = path_target(plan, path)?;
    let selected = shape(plan.service(), target)?;
    match kind {
        Some(SelectionKind::MapValues | SelectionKind::MapEntries) => selected
            .map_value()?
            .ok_or(ProbeError::Invalid("comparison map value missing")),
        Some(SelectionKind::MapKeys) => selected
            .map_key()?
            .ok_or(ProbeError::Invalid("comparison map key missing")),
        Some(SelectionKind::List) => selected
            .element()?
            .ok_or(ProbeError::Invalid("comparison list element missing")),
        None if selected.kind()? == "list" => selected
            .element()?
            .ok_or(ProbeError::Invalid("comparison list element missing")),
        _ => Ok(target),
    }
}
fn path_target(
    plan: &CallablePlan<'_>,
    path: &[inventory_core::record::FieldId],
) -> Result<Target, ProbeError> {
    path.iter()
        .try_fold(plan.operation().output()?, |target, id| {
            shape(plan.service(), target)?
                .members()?
                .find_map(|member| match member {
                    Ok(member) if member.id == *id => Some(Ok(member.target)),
                    Ok(_) => None,
                    Err(error) => Some(Err(ProbeError::from(error))),
                })
                .ok_or(ProbeError::Invalid("comparison selected member is missing"))?
        })
}

fn nested_item(
    plan: &CallablePlan<'_>,
    parents: &[inventory_core::record::FieldId],
    items: inventory_core::record::FieldId,
) -> Result<Target, ProbeError> {
    let parent_list = path_target(plan, parents)?;
    let parent_element = shape(plan.service(), parent_list)?
        .element()?
        .ok_or(ProbeError::Invalid("nested parent element is missing"))?;
    let parent = shape(plan.service(), parent_element)?;
    let item_list = parent
        .members()?
        .find_map(|member| match member {
            Ok(member) if member.id == items => Some(Ok(member.target)),
            Ok(_) => None,
            Err(error) => Some(Err(ProbeError::from(error))),
        })
        .ok_or(ProbeError::Invalid("nested item member is missing"))??;
    shape(plan.service(), item_list)?
        .element()?
        .ok_or(ProbeError::Invalid("nested item element is missing"))
}

#[cfg(test)]
#[path = "../../../../aws-wire/src/response/json/fixture_catalog.rs"]
mod fixture_catalog;

#[cfg(test)]
mod tests {
    use super::*;
    use inventory_core::catalog::Catalog;
    #[test]
    fn given_extra_describe_member_when_selected_item_models_are_checked_then_missing_reports_member()
    -> Result<(), Box<dyn std::error::Error>> {
        let bytes = super::super::tests::extra_member_catalog()?;
        let catalog = inventory_core::catalog::runtime::RuntimeCatalog::from_bytes(&bytes)?;
        let parent = catalog
            .lookup("bindings", "ListParents0")
            .ok_or("parent missing")?;
        let child = catalog
            .lookup("bindings", "ListChildren")
            .ok_or("child missing")?;
        assert_eq!(missing(parent, child)?, Some("$.Count".to_owned()));
        Ok(())
    }

    #[test]
    fn given_extra_modeled_describe_member_when_values_omit_it_then_shape_guard_still_refuses_equality()
    -> Result<(), Box<dyn std::error::Error>> {
        let model = serde_json::json!({"smithy":"2.0","shapes":{
            "n#Compare":{"type":"service","version":"1","operations":[{"target":"n#Read"}]},
            "n#Read":{"type":"operation","input":{"target":"n#Parent"},"output":{"target":"n#Child"}},
            "n#Parent":{"type":"structure","members":{"Id":{"target":"smithy.api#String"}}},
            "n#Child":{"type":"structure","members":{"Id":{"target":"smithy.api#String"},"Extra":{"target":"smithy.api#String"}}}
        }});
        let bytes = fixture_catalog::compile_service(&model, "n#Compare")?;
        let catalog = Catalog::borrow(&bytes)?;
        let service = catalog.service("Compare")?.ok_or("service missing")?;
        let operation = service.operations()?.next().ok_or("operation missing")??;
        let mut pending = Vec::new();
        assert_eq!(
            visit(
                service,
                operation.input()?,
                operation.output()?,
                "$",
                &mut pending
            )?,
            Some("$.Extra".to_owned())
        );
        Ok(())
    }
}
