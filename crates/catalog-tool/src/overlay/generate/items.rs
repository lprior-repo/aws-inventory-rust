use crate::{
    overlay::ItemSelector,
    plans::{
        Model, Pagination,
        model::{shape_type, target},
    },
};
use serde_json::Value;

pub(super) fn select(
    model: &Model<'_>,
    operation: &str,
    pagination: &Pagination<'_>,
) -> Result<Option<ItemSelector>, String> {
    if matches!(pagination, Pagination::Modeled { items: Some(_), .. }) {
        return Ok(None);
    }
    let members = members(model, operation)?;
    let mut lists = members.iter().filter(|(_, member)| {
        target(member)
            .and_then(|id| model.shape(id))
            .is_some_and(|shape| matches!(shape_type(shape.value), Some("list" | "map")))
    });
    let (name, _) = lists.next().ok_or("items: no single list member")?;
    if lists.next().is_some() {
        return Err("items: no single list member".to_owned());
    }
    ItemSelector::parse(name)
        .map(Some)
        .map_err(|error| error.to_string())
}

fn members<'a>(
    model: &Model<'a>,
    operation: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    model
        .side_root(operation, "output")
        .and_then(|root| root.value.get("members"))
        .and_then(Value::as_object)
        .ok_or_else(|| "items: no single list member".to_owned())
}
