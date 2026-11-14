use super::{
    Error, Model,
    model::{required, target, trait_value},
};
use crate::classes::{Class, DenyList, classify};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExpansionReason {
    NotInResource,

    RequiredMemberNotIdentifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Expansion<'a> {
    Modeled {
        resource: &'a str,
        identifiers: BTreeMap<&'a str, &'a str>,
        list_source: bool,
        ambiguous_resource: bool,
    },

    NeedsOverlay(ExpansionReason),
}

pub fn expansion<'a>(
    model: &Model<'a>,
    id: &str,
    deny: &DenyList<'_>,
) -> Result<Expansion<'a>, Error> {
    let members = model.members(id, "input")?;
    let candidates = candidates(model, id, members)?;
    let Some((resource, shape)) = candidates.selected else {
        return Ok(Expansion::NeedsOverlay(candidates.reason()));
    };
    let identifiers = bindings(shape, members);
    Ok(Expansion::Modeled {
        resource,
        identifiers,
        list_source: list_source(model, shape, deny)?,
        ambiguous_resource: candidates.qualifying > 1,
    })
}

fn candidates<'a>(
    model: &Model<'a>,
    id: &str,
    members: Option<&serde_json::Map<String, Value>>,
) -> Result<Candidates<'a>, Error> {
    model
        .of_type("resource")
        .filter(|(_, r)| references(r, id))
        .try_fold(Candidates::default(), |mut c, (rid, r)| {
            c.record(rid, r, members)?;
            Ok(c)
        })
}

#[derive(Default)]
struct Candidates<'a> {
    references: u64,
    qualifying: u64,
    selected: Option<(&'a str, &'a Value)>,
}
impl<'a> Candidates<'a> {
    fn reason(&self) -> ExpansionReason {
        if self.references == 0 {
            ExpansionReason::NotInResource
        } else {
            ExpansionReason::RequiredMemberNotIdentifier
        }
    }
    fn record(
        &mut self,
        id: &'a str,
        resource: &'a Value,
        members: Option<&serde_json::Map<String, Value>>,
    ) -> Result<(), Error> {
        self.references = self.references.checked_add(1).ok_or(Error::CountMismatch)?;
        let binds = members.is_none_or(|m| {
            m.iter()
                .filter(|(_, v)| required(v))
                .all(|(k, v)| identifier(resource, k, v).is_some())
        });
        if binds {
            self.qualifying = self.qualifying.checked_add(1).ok_or(Error::CountMismatch)?;
            if self.selected.is_none_or(|(old, _)| id < old) {
                self.selected = Some((id, resource));
            }
        }
        Ok(())
    }
}
fn bindings<'a>(
    resource: &'a Value,
    members: Option<&'a serde_json::Map<String, Value>>,
) -> BTreeMap<&'a str, &'a str> {
    members
        .into_iter()
        .flat_map(|m| m.iter())
        .filter(|(_, v)| required(v))
        .filter_map(|(k, v)| identifier(resource, k, v).map(|i| (k.as_str(), i)))
        .collect()
}

fn references(resource: &Value, id: &str) -> bool {
    ["read", "list"]
        .into_iter()
        .any(|key| resource.get(key).and_then(target) == Some(id))
        || ["operations", "collectionOperations"]
            .into_iter()
            .any(|key| {
                resource
                    .get(key)
                    .and_then(Value::as_array)
                    .is_some_and(|ops| ops.iter().any(|op| target(op) == Some(id)))
            })
}
fn identifier<'a>(resource: &'a Value, member: &'a str, value: &Value) -> Option<&'a str> {
    let identifiers = resource.get("identifiers")?.as_object()?;
    if identifiers.contains_key(member) {
        return Some(member);
    }
    let key = trait_value(value, "smithy.api#resourceIdentifier")?.as_str()?;
    identifiers.get_key_value(key).map(|(k, _)| k.as_str())
}
fn list_source(model: &Model<'_>, resource: &Value, deny: &DenyList<'_>) -> Result<bool, Error> {
    let Some(id) = resource.get("list").and_then(target) else {
        return Ok(false);
    };
    let Some((id, _)) = model.shapes.get_key_value(id) else {
        return Err(model.invalid(id, "resource list"));
    };
    Ok(matches!(
        classify(&model.facts(id)?, deny),
        Class::Enumerate | Class::Expand
    ))
}
