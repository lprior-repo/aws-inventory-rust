use crate::{
    overlay::ItemSelector,
    plans::{
        Model, Pagination,
        model::{shape_type, target},
    },
};
use serde_json::Value;

type Members = serde_json::Map<String, Value>;

pub(super) fn select(
    model: &Model<'_>,
    operation: &str,
    pagination: &Pagination<'_>,
) -> Result<Option<ItemSelector>, String> {
    if matches!(pagination, Pagination::Modeled { items: Some(_), .. }) {
        return Ok(None);
    }
    let members = members(model, operation)?;
    let mut lists = members
        .iter()
        .filter(|(_, member)| collection(model, member));
    let (name, _) = lists.next().ok_or("items: no single list member")?;
    if lists.next().is_some() {
        return Err("items: no single list member".to_owned());
    }
    selector(name)
}

pub(super) fn expand(
    model: &Model<'_>,
    operation: &str,
    pagination: &Pagination<'_>,
) -> Result<Option<ItemSelector>, String> {
    if matches!(pagination, Pagination::Modeled { items: Some(_), .. }) {
        return Ok(None);
    }
    let members = members(model, operation)?;
    let collections = members
        .values()
        .filter(|member| collection(model, member))
        .count();
    if collections > 1 {
        return Err("items: no single list member".to_owned());
    }
    if super::identity::derive(model, operation, Some(&ItemSelector::Root), pagination).is_ok() {
        return Ok(Some(ItemSelector::Root));
    }
    if collections == 1 {
        return select(model, operation, pagination);
    }
    structure(model, members)
}

fn structure(model: &Model<'_>, members: &Members) -> Result<Option<ItemSelector>, String> {
    let mut structures = members.iter().filter(|(_, member)| {
        target(member)
            .and_then(|id| model.shape(id))
            .is_some_and(|shape| shape_type(shape.value) == Some("structure"))
    });
    match (structures.next(), structures.next()) {
        (Some((name, _)), None) => selector(name),
        (None, None) => Ok(Some(ItemSelector::Root)),
        _ => Err("items: ambiguous resource structures".to_owned()),
    }
}

fn collection(model: &Model<'_>, member: &Value) -> bool {
    target(member)
        .and_then(|id| model.shape(id))
        .is_some_and(|shape| matches!(shape_type(shape.value), Some("list" | "map")))
}

fn selector(name: &str) -> Result<Option<ItemSelector>, String> {
    ItemSelector::parse(name)
        .map(Some)
        .map_err(|error| error.to_string())
}

fn members<'a>(model: &Model<'a>, operation: &str) -> Result<&'a Members, String> {
    model
        .side_root(operation, "output")
        .and_then(|root| root.value.get("members"))
        .and_then(Value::as_object)
        .ok_or_else(|| "items: no single list member".to_owned())
}
