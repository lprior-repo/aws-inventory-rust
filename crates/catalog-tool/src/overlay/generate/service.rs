use super::{Derived, derive};
use crate::{
    classes::{self, Class, DenyList},
    overlay::{OperationOverlay, Origin, OverlayClass, OverlayError, Scope, ServiceOverlay},
    plans::{Model, Vocabulary},
};
use std::collections::BTreeMap;

type Entries<'a> = Vec<(&'a str, OperationOverlay)>;

pub(super) struct GeneratedService {
    pub(super) policy: ServiceOverlay,
    pub(super) enumerate: usize,
    pub(super) callable: usize,
    pub(super) expand: usize,
    pub(super) expanded: usize,
    pub(super) reasons: BTreeMap<String, usize>,
}

pub(super) fn generate(
    service: &str,
    model: &Model<'_>,
    scope: Option<Scope>,
    vocabulary: &Vocabulary,
    deny: &DenyList<'_>,
    reviewed: Option<&ServiceOverlay>,
) -> Result<GeneratedService, OverlayError> {
    let (mut entries, mut stats) = initialize(model, scope, vocabulary, deny, reviewed)?;
    defer_kind_conflicts(model, &mut entries, &mut stats)?;
    let (expand, _) =
        super::children::generate(model, scope, vocabulary, deny, &mut entries, reviewed)
            .map_err(|reason| invalid(&reason))?;
    for (operation, entry) in &mut entries {
        retain_reviewed(operation, entry, reviewed);
    }
    let expanded = entries
        .iter()
        .filter(|(_, entry)| entry.class == Some(OverlayClass::Expand))
        .count();
    let mut generated = complete(service, entries, stats)?;
    generated.expand = expand;
    generated.expanded = expanded;
    Ok(generated)
}

fn initialize<'a>(
    model: &Model<'a>,
    scope: Option<Scope>,
    vocabulary: &Vocabulary,
    deny: &DenyList<'_>,
    reviewed: Option<&ServiceOverlay>,
) -> Result<(Entries<'a>, Counts), OverlayError> {
    let mut entries = Vec::new();
    let mut stats = Counts::default();
    for operation in model.operation_names() {
        let class = reviewed_class(model, operation, deny, reviewed)?;
        if let Some(mut entry) = entry(model, operation, class, scope, vocabulary, deny)? {
            retain_reviewed(operation, &mut entry, reviewed);
            record(class, &entry, &mut stats)?;
            append(&mut entries, operation, entry)?;
        }
    }
    Ok((entries, stats))
}

fn reviewed_class(
    model: &Model<'_>,
    operation: &str,
    deny: &DenyList<'_>,
    reviewed: Option<&ServiceOverlay>,
) -> Result<Class, OverlayError> {
    let id = model
        .operation_id(operation)
        .ok_or_else(|| invalid("missing operation"))?;
    let facts = model
        .facts(id)
        .map_err(|error| invalid(&error.to_string()))?;
    let class = classes::classify(&facts, deny);
    let entry = reviewed.and_then(|policy| policy.operations().get(operation));
    Ok(match entry.and_then(|entry| entry.class) {
        Some(OverlayClass::Deny(reason)) => Class::Denied(reason),
        _ => class,
    })
}

fn retain_reviewed(
    operation: &str,
    entry: &mut OperationOverlay,
    reviewed: Option<&ServiceOverlay>,
) {
    if let Some(hand) = reviewed.and_then(|policy| policy.operations().get(operation))
        && hand.class == Some(OverlayClass::Defer)
        && hand.generation_blocker.is_none()
    {
        entry.clone_from(hand);
    }
}

fn record(class: Class, entry: &OperationOverlay, stats: &mut Counts) -> Result<(), OverlayError> {
    if class != Class::Enumerate {
        return Ok(());
    }
    increment(&mut stats.enumerate)?;
    if entry.class == Some(OverlayClass::Enumerate) {
        increment(&mut stats.callable)
    } else {
        let reason = entry
            .defer_reason
            .as_ref()
            .ok_or_else(|| invalid("missing defer reason"))?;
        increment(stats.reasons.entry(reason.clone()).or_default())
    }
}

fn append<'a>(
    entries: &mut Vec<(&'a str, OperationOverlay)>,
    name: &'a str,
    entry: OperationOverlay,
) -> Result<(), OverlayError> {
    if entries.len() >= crate::overlay::MAX_OPERATIONS {
        return Err(invalid("operation limit"));
    }
    entries
        .try_reserve(1)
        .map_err(|_| invalid("operation allocation limit"))?;
    entries.push((name, entry));
    Ok(())
}

fn complete(
    service: &str,
    entries: Vec<(&str, OperationOverlay)>,
    stats: Counts,
) -> Result<GeneratedService, OverlayError> {
    let mut policy = ServiceOverlay::new(service, entries)?;
    policy.reviewer = "generated".to_owned();
    policy.origin = policy
        .operations
        .keys()
        .cloned()
        .map(|name| (name, Origin::Generated))
        .collect();
    Ok(GeneratedService {
        policy,
        enumerate: stats.enumerate,
        callable: stats.callable,
        expand: 0,
        expanded: 0,
        reasons: stats.reasons,
    })
}

#[derive(Default)]
struct Counts {
    enumerate: usize,
    callable: usize,
    reasons: BTreeMap<String, usize>,
}

fn defer_kind_conflicts(
    model: &Model<'_>,
    entries: &mut [(&str, OperationOverlay)],
    stats: &mut Counts,
) -> Result<(), OverlayError> {
    entries.sort_unstable_by_key(|(operation, _)| *operation);
    let mut producers = BTreeMap::new();
    for (operation, entry) in entries {
        if entry.class != Some(OverlayClass::Enumerate) {
            continue;
        }
        if let Err(reason) =
            crate::kind_map::generate::record_item(model, operation, entry, &mut producers)
        {
            stats.callable = stats
                .callable
                .checked_sub(1)
                .ok_or_else(|| invalid("callable count underflow"))?;
            increment(stats.reasons.entry(reason.clone()).or_default())?;
            *entry = deferred(reason);
        }
    }
    Ok(())
}

fn entry(
    model: &Model<'_>,
    operation: &str,
    class: Class,
    scope: Option<Scope>,
    vocabulary: &Vocabulary,
    deny: &DenyList<'_>,
) -> Result<Option<OperationOverlay>, OverlayError> {
    match class {
        Class::Denied(_) => Ok(None),
        Class::Expand => Ok(Some(deferred(
            "expand: bindings not generated yet".to_owned(),
        ))),
        Class::Unclassified => Ok(Some(deferred("unclassified read: not reviewed".to_owned()))),
        Class::Enumerate => {
            let result = scope.map_or_else(
                || Err("endpoint: not resolved".to_owned()),
                |scope| derive::enumerate(model, operation, scope, vocabulary, deny),
            );
            Ok(Some(enumerate(result)))
        }
    }
}

fn enumerate(result: Derived) -> OperationOverlay {
    match result {
        Ok(entry) => entry,
        Err(reason) => deferred(reason),
    }
}

pub(super) fn deferred(reason: String) -> OperationOverlay {
    OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some(reason),
        ..OperationOverlay::default()
    }
}

fn increment(value: &mut usize) -> Result<(), OverlayError> {
    *value = value
        .checked_add(1)
        .ok_or_else(|| invalid("candidate count overflow"))?;
    Ok(())
}

fn invalid(reason: &str) -> OverlayError {
    OverlayError::Invalid(reason.to_owned())
}
