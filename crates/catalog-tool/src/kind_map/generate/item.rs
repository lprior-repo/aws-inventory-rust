use super::{Candidate, Failure};
use crate::{
    overlay::{ItemSelector, OperationOverlay},
    plans::{
        Model, Pagination, Vocabulary,
        model::{ShapeRef, name, shape_type, target},
    },
};
use serde_json::Value;
use std::collections::BTreeMap;

pub(super) fn candidate<'a>(
    model: &Model<'a>,
    service: &'a str,
    operation: &'a str,
    plan: &'a OperationOverlay,
) -> Result<Candidate<'a>, Failure> {
    let (item, resource) = selected(model, operation, plan)?;
    let metadata = metadata(model)?;
    let sdk_id = metadata
        .get("sdkId")
        .and_then(Value::as_str)
        .ok_or("kind: missing SDK ID")?;
    Ok(Candidate {
        service,
        operation,
        shape_id: item.id,
        resource,
        sdk_id,
        cfn_service: metadata.get("cloudFormationName").and_then(Value::as_str),
        display: display(model, item, resource),
    })
}

fn metadata<'a>(model: &Model<'a>) -> Result<&'a Value, Failure> {
    model
        .of_type("service")
        .next()
        .and_then(|(_, value)| value.get("traits"))
        .and_then(|traits| traits.get("aws.api#service"))
        .ok_or_else(|| "kind: missing aws.api#service metadata".into())
}

fn selected<'a>(
    model: &Model<'a>,
    operation: &str,
    plan: &'a OperationOverlay,
) -> Result<(ShapeRef<'a>, &'a str), Failure> {
    let path = selected_path(model, operation, plan)?;
    let output = model
        .side_root(operation, "output")
        .ok_or("kind: output absent")?;
    let aggregate = path.map_or(Ok(output), |path| model.resolve_path_from(output, path))?;
    let item = model.unwrap_aggregates(aggregate)?;
    let resource = if matches!(shape_type(item.value), Some("structure" | "union")) {
        strip_suffix(name(item.id))
    } else {
        path.and_then(|path| path.rsplit('.').next())
            .map_or(name(aggregate.id), |path| path)
    };
    Ok((item, resource))
}
fn selected_path<'a>(
    model: &Model<'a>,
    operation: &str,
    plan: &'a OperationOverlay,
) -> Result<Option<&'a str>, Failure> {
    match &plan.items {
        Some(ItemSelector::Root) => Ok(None),
        Some(ItemSelector::Member(path) | ItemSelector::Explicit { path, .. }) => {
            Ok(Some(path.as_str()))
        }
        None => {
            let id = model
                .operation_id(operation)
                .ok_or("kind: operation absent")?;
            match crate::plans::pagination(model, id, &Vocabulary::default())? {
                Pagination::Modeled { items, .. } => Ok(items),
                _ => Err("kind: no selected item".into()),
            }
        }
    }
}

fn strip_suffix(name: &str) -> &str {
    ["Summary", "Description", "Info", "Entry", "Item", "Detail"]
        .into_iter()
        .find_map(|suffix| name.strip_suffix(suffix).filter(|name| !name.is_empty()))
        .map_or(name, |name| name)
}

fn display<'a>(model: &Model<'_>, item: ShapeRef<'a>, resource: &str) -> Option<&'a str> {
    let members = item.value.get("members")?.as_object()?;
    let string = |value: &Value| {
        target(value)
            .and_then(|id| model.shape(id))
            .is_some_and(|shape| shape_type(shape.value) == Some("string"))
    };
    if let Some((name, _)) = members
        .get_key_value("Name")
        .filter(|(_, value)| string(value))
    {
        return Some(name);
    }
    members
        .iter()
        .find(|(name, value)| name.strip_suffix("Name") == Some(resource) && string(value))
        .map(|(name, _)| name.as_str())
}

pub(crate) fn record(
    model: &Model<'_>,
    operation: &str,
    plan: &OperationOverlay,
    producers: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let (item, resource) = selected(model, operation, plan).map_err(|error| error.to_string())?;
    if producers
        .get(resource)
        .is_some_and(|shape| shape != item.id)
    {
        return Err(super::CONFLICT.to_owned());
    }
    producers
        .entry(resource.to_owned())
        .or_insert_with(|| item.id.to_owned());
    Ok(())
}
