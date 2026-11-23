use crate::{
    overlay::{Identity, ItemSelector, MemberPath},
    plans::{
        Model, Pagination,
        model::{ShapeRef, shape_type, target},
    },
};
use serde_json::Value;

const MISSING: &str = "identity: no ARN member and no modeled identifiers";

pub(super) fn derive(
    model: &Model<'_>,
    operation: &str,
    items: Option<&ItemSelector>,
    pagination: &Pagination<'_>,
) -> Result<Identity, String> {
    let path = selected_path(items, pagination).ok_or(MISSING)?;
    let selected = model
        .resolve_path(operation, "output", path)
        .map_err(str::to_owned)?
        .ok_or(MISSING)?;
    let selected = model.unwrap_aggregates(selected).map_err(str::to_owned)?;
    if shape_type(selected.value) == Some("string") {
        return Ok(Identity::Key(vec![
            MemberPath::selected("$").map_err(|e| e.to_string())?,
        ]));
    }
    if let Some(arn) = arn(model, selected, operation) {
        return Ok(Identity::Arn(
            MemberPath::new(arn).map_err(|e| e.to_string())?,
        ));
    }
    modeled_identifiers(model, operation, selected).ok_or_else(|| MISSING.to_owned())
}

fn selected_path<'a>(
    items: Option<&'a ItemSelector>,
    pagination: &Pagination<'a>,
) -> Option<&'a str> {
    if let Some(ItemSelector::Member(path)) = items {
        return Some(path.as_str());
    }
    if let Pagination::Modeled { items, .. } = pagination {
        return *items;
    }
    None
}

fn arn<'a>(model: &Model<'a>, item: ShapeRef<'a>, operation: &str) -> Option<&'a str> {
    let members = item.value.get("members")?.as_object()?;
    let string = |member: &Value| is_string(model, member);
    if members.get("Arn").is_some_and(string) {
        return Some("Arn");
    }
    let noun = operation
        .strip_prefix("List")
        .or_else(|| operation.strip_prefix("Describe"))?;
    let noun = noun.strip_suffix('s').map_or(noun, |noun| noun);
    let mut candidates = members
        .iter()
        .filter(|(name, member)| name.starts_with(noun) && name.ends_with("Arn") && string(member));
    let (name, _) = candidates.next()?;
    candidates.next().is_none().then_some(name.as_str())
}

fn is_string(model: &Model<'_>, member: &Value) -> bool {
    target(member)
        .and_then(|id| model.shape(id))
        .is_some_and(|shape| shape_type(shape.value) == Some("string"))
}

fn modeled_identifiers(model: &Model<'_>, operation: &str, item: ShapeRef<'_>) -> Option<Identity> {
    let id = model.operation_id(operation)?;
    let members = item.value.get("members")?.as_object()?;
    let mut resources = model
        .of_type("resource")
        .filter(|(_, resource)| resource.get("list").and_then(target) == Some(id));
    let (_, resource) = resources.next()?;
    if resources.next().is_some() {
        return None;
    }
    let identifiers = resource.get("identifiers")?.as_object()?;
    if identifiers.is_empty() || identifiers.len() > 32 {
        return None;
    }
    let paths = identifiers
        .keys()
        .map(|key| {
            members
                .contains_key(key)
                .then(|| MemberPath::new(key).ok())
                .flatten()
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Identity::Key(paths))
}
