use super::{Derived, identity, items, pagination};
use crate::{
    classes::{self, Class, DenyList},
    overlay::{OperationOverlay, OverlayClass, Scope, check::validation},
    plans::{self, Model, Pagination, Vocabulary},
};
use std::collections::BTreeMap;

pub(crate) fn enumerate(
    model: &Model<'_>,
    operation: &str,
    scope: Scope,
    vocabulary: &Vocabulary,
    deny: &DenyList<'_>,
) -> Derived {
    candidate(model, operation, scope, vocabulary, deny).map_err(bound_reason)
}

fn candidate(
    model: &Model<'_>,
    operation: &str,
    scope: Scope,
    vocabulary: &Vocabulary,
    deny: &DenyList<'_>,
) -> Result<OperationOverlay, String> {
    let id = model
        .operation_id(operation)
        .ok_or("not an enumerate candidate")?;
    let facts = model.facts(id).map_err(|error| error.to_string())?;
    if classes::classify(&facts, deny) != Class::Enumerate {
        return Err("not an enumerate candidate".to_owned());
    }
    owner_filter(model, id)?;
    let pagination = plans::pagination(model, id, vocabulary).map_err(|error| error.to_string())?;
    let entry = build_entry(model, operation, scope, &pagination, vocabulary)?;
    dry_run(model, operation, &facts, vocabulary, deny, entry)
}

fn build_entry(
    model: &Model<'_>,
    operation: &str,
    scope: Scope,
    pagination: &Pagination<'_>,
    vocabulary: &Vocabulary,
) -> Result<OperationOverlay, String> {
    let items = items::select(model, operation, pagination)?;
    let identity = identity::derive(model, operation, items.as_ref(), pagination)?;
    Ok(OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(scope),
        items,
        identity: Some(identity),
        paginator: pagination::derive(model, operation, pagination, vocabulary)?,
        ..OperationOverlay::default()
    })
}

fn owner_filter(model: &Model<'_>, id: &str) -> Result<(), String> {
    let members = model
        .members(id, "input")
        .map_err(|error| error.to_string())?;
    if let Some(members) = members {
        for name in include_str!("../../../../../catalog/overlay/defer-inputs.txt").lines() {
            if members.contains_key(name) {
                return Err(format!("input {name} needs a reviewed fixed value"));
            }
        }
    }
    Ok(())
}

fn dry_run(
    model: &Model<'_>,
    operation: &str,
    facts: &classes::OperationFacts<'_>,
    vocabulary: &Vocabulary,
    deny: &DenyList<'_>,
    entry: OperationOverlay,
) -> Result<OperationOverlay, String> {
    let overlays = BTreeMap::new();
    let result = validation::operation(validation::StateArgs {
        service: facts.service,
        name: operation,
        item: Some(entry.clone()),
        class: Class::Enumerate,
        model,
        deny,
        vocabulary,
        overlays: &overlays,
    });
    accept(result)?;
    Ok(entry)
}

fn accept(result: validation::OperationResult) -> Result<(), String> {
    if let Some(violation) = result.violations.first() {
        return Err(format!("{}: {}", violation.key(), violation.value()));
    }
    if result.callable.is_none() {
        return Err("checker: entry is not callable".to_owned());
    }
    Ok(())
}

pub(super) fn bound_reason(mut reason: String) -> String {
    if reason.len() > 1024 {
        reason.truncate(reason.floor_char_boundary(1024));
    }
    reason
}
