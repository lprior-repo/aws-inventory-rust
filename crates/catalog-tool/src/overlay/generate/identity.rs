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
    let selected = selected(model, operation, items, pagination)?;
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

fn selected<'a>(
    model: &Model<'a>,
    operation: &str,
    items: Option<&ItemSelector>,
    pagination: &Pagination<'_>,
) -> Result<ShapeRef<'a>, String> {
    let shape = match items {
        Some(ItemSelector::Root) => model.side_root(operation, "output").ok_or(MISSING)?,
        _ => {
            let path = selected_path(items, pagination).ok_or(MISSING)?;
            model
                .resolve_path(operation, "output", path)
                .map_err(str::to_owned)?
                .ok_or(MISSING)?
        }
    };
    let item = model.unwrap_aggregates(shape).map_err(str::to_owned)?;
    if shape_type(shape.value) == Some("map") && shape_type(item.value) == Some("string") {
        return Err(MISSING.to_owned());
    }
    Ok(item)
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
    let noun = noun(operation)?;
    let generic = members.keys().any(|name| name.eq_ignore_ascii_case("Arn"));
    let mut candidates = members
        .iter()
        .filter(|(name, member)| arn_name(name, noun, generic) && is_string(model, member));
    let (name, _) = candidates.next()?;
    candidates.next().is_none().then_some(name.as_str())
}

fn noun(operation: &str) -> Option<&str> {
    let noun = operation
        .strip_prefix("List")
        .or_else(|| operation.strip_prefix("Describe"))
        .or_else(|| operation.strip_prefix("Get"))?;
    Some(noun.strip_suffix('s').map_or(noun, |noun| noun))
}

fn arn_name(name: &str, noun: &str, generic: bool) -> bool {
    if generic {
        return name.eq_ignore_ascii_case("Arn");
    }
    name.get(..noun.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(noun))
        && name
            .get(name.len().saturating_sub(3)..)
            .is_some_and(|suffix| suffix.eq_ignore_ascii_case("Arn"))
}

fn is_string(model: &Model<'_>, member: &Value) -> bool {
    target(member)
        .and_then(|id| model.shape(id))
        .is_some_and(|shape| shape_type(shape.value) == Some("string"))
}

fn modeled_identifiers(model: &Model<'_>, operation: &str, item: ShapeRef<'_>) -> Option<Identity> {
    let resource = resource(model, operation)?;
    let identifiers = resource.get("identifiers")?.as_object()?;
    if identifiers.is_empty() || identifiers.len() > 32 {
        return None;
    }
    let members = item.value.get("members")?.as_object()?;
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

fn resource<'a>(model: &Model<'a>, operation: &str) -> Option<&'a Value> {
    let id = model.operation_id(operation)?;
    let mut resources = model.of_type("resource").filter(|(_, resource)| {
        ["list", "read"]
            .iter()
            .any(|key| resource.get(*key).and_then(target) == Some(id))
    });
    let (_, resource) = resources.next()?;
    resources.next().is_none().then_some(resource)
}
