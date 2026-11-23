use super::{expand, service::deferred};
use crate::{
    classes::DenyList,
    overlay::{OperationOverlay, OverlayClass, Scope, ServiceOverlay, merge::generated_wins},
    plans::{Model, Vocabulary},
};
use std::collections::BTreeMap;

type Entries<'a> = [(&'a str, OperationOverlay)];

pub(super) fn generate(
    model: &Model<'_>,
    scope: Option<Scope>,
    vocabulary: &Vocabulary,
    deny: &DenyList<'_>,
    entries: &mut Entries<'_>,
    reviewed: Option<&ServiceOverlay>,
) -> Result<(usize, usize), String> {
    let rules = candidates(model, deny, entries)?;
    (0..entries.len()).try_fold(true, |changed, _| {
        if changed {
            let parents = parents(entries, reviewed);
            pass(model, scope, vocabulary, deny, entries, &rules, &parents)
        } else {
            Ok(false)
        }
    })?;
    Ok(counts(&rules, entries))
}

fn counts(candidates: &BTreeMap<&str, bool>, entries: &Entries<'_>) -> (usize, usize) {
    let expanded = entries
        .iter()
        .filter(|(_, entry)| entry.class == Some(OverlayClass::Expand))
        .count();
    (
        candidates.values().filter(|eligible| **eligible).count(),
        expanded,
    )
}

fn candidates<'a>(
    model: &Model<'a>,
    deny: &DenyList<'_>,
    entries: &Entries<'_>,
) -> Result<BTreeMap<&'a str, bool>, String> {
    model
        .operation_names()
        .filter(|operation| entries.iter().any(|(name, _)| name == operation))
        .map(|operation| {
            let id = model
                .operation_id(operation)
                .ok_or("expand: missing operation")?;
            let facts = model.facts(id).map_err(|error| error.to_string())?;
            Ok((
                operation,
                expand::eligible(crate::classes::classify(&facts, deny), &facts),
            ))
        })
        .collect()
}

fn pass(
    model: &Model<'_>,
    scope: Option<Scope>,
    vocabulary: &Vocabulary,
    deny: &DenyList<'_>,
    entries: &mut Entries<'_>,
    candidates: &BTreeMap<&str, bool>,
    parents: &BTreeMap<String, OperationOverlay>,
) -> Result<bool, String> {
    entries
        .iter_mut()
        .try_fold(false, |changed, (operation, entry)| {
            if candidates.get(operation) != Some(&true) || entry.class == Some(OverlayClass::Expand)
            {
                return Ok(changed);
            }
            let result = child_entry(model, operation, scope, vocabulary, deny, parents);
            let callable = result.is_ok();
            *entry = match result {
                Ok(entry) => entry,
                Err(reason) => deferred(reason),
            };
            Ok(changed || callable)
        })
}

fn child_entry(
    model: &Model<'_>,
    operation: &str,
    scope: Option<Scope>,
    vocabulary: &Vocabulary,
    deny: &DenyList<'_>,
    parents: &BTreeMap<String, OperationOverlay>,
) -> super::Derived {
    scope.map_or_else(
        || Err("endpoint: not resolved".to_owned()),
        |scope| expand::derive(model, operation, scope, vocabulary, deny, parents),
    )
}

fn parents(
    entries: &Entries<'_>,
    reviewed: Option<&ServiceOverlay>,
) -> BTreeMap<String, OperationOverlay> {
    entries
        .iter()
        .filter_map(|(name, generated)| {
            let hand = reviewed.and_then(|policy| policy.operations().get(*name));
            let entry = if generated_wins(hand, generated) {
                generated
            } else {
                hand?
            };
            matches!(
                entry.class,
                Some(OverlayClass::Enumerate | OverlayClass::Expand)
            )
            .then(|| ((*name).to_owned(), entry.clone()))
        })
        .collect()
}
