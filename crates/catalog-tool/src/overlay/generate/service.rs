use super::{Derived, derive};
use crate::{
    classes::{self, Class, DenyList},
    overlay::{OperationOverlay, Origin, OverlayClass, OverlayError, Scope, ServiceOverlay},
    plans::{Model, Vocabulary},
};
use std::collections::BTreeMap;

pub(super) struct GeneratedService {
    pub(super) policy: ServiceOverlay,
    pub(super) enumerate: usize,
    pub(super) callable: usize,
    pub(super) reasons: BTreeMap<String, usize>,
}

pub(super) fn generate(
    service: &str,
    model: &Model<'_>,
    scope: Option<Scope>,
    vocabulary: &Vocabulary,
    deny: &DenyList<'_>,
) -> Result<GeneratedService, OverlayError> {
    let mut entries = Vec::new();
    let mut stats = Counts::default();
    for operation in model.operation_names() {
        let id = model.operation_id(operation).ok_or_else(|| invalid("missing operation"))?;
        let facts = model.facts(id).map_err(|error| invalid(&error.to_string()))?;
        let class = classes::classify(&facts, deny);
        if let Some(entry) = entry(model, operation, class, scope, vocabulary, deny, &mut stats)? {
            append(&mut entries, operation, entry)?;
        }
    }
    complete(service, entries, stats)
}

fn append<'a>(
    entries: &mut Vec<(&'a str, OperationOverlay)>,
    name: &'a str,
    entry: OperationOverlay,
) -> Result<(), OverlayError> {
    if entries.len() >= crate::overlay::MAX_OPERATIONS {
        return Err(invalid("operation limit"));
    }
    entries.try_reserve(1).map_err(|_| invalid("operation allocation limit"))?;
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
    policy.origin = policy.operations.keys().cloned().map(|name| (name, Origin::Generated)).collect();
    Ok(GeneratedService {
        policy,
        enumerate: stats.enumerate,
        callable: stats.callable,
        reasons: stats.reasons,
    })
}

#[derive(Default)]
struct Counts {
    enumerate: usize,
    callable: usize,
    reasons: BTreeMap<String, usize>,
}

fn entry(
    model: &Model<'_>,
    operation: &str,
    class: Class,
    scope: Option<Scope>,
    vocabulary: &Vocabulary,
    deny: &DenyList<'_>,
    stats: &mut Counts,
) -> Result<Option<OperationOverlay>, OverlayError> {
    match class {
        Class::Denied(_) => Ok(None),
        Class::Expand => Ok(Some(deferred("expand: bindings not generated yet".to_owned()))),
        Class::Unclassified => Ok(Some(deferred("unclassified read: not reviewed".to_owned()))),
        Class::Enumerate => {
            increment(&mut stats.enumerate)?;
            let result = scope.map_or_else(|| Err("endpoint: not resolved".to_owned()),
                |scope| derive::enumerate(model, operation, scope, vocabulary, deny));
            enumerate(result, stats).map(Some)
        }
    }
}

fn enumerate(result: Derived, stats: &mut Counts) -> Result<OperationOverlay, OverlayError> {
    match result {
        Ok(entry) => {
            increment(&mut stats.callable)?;
            Ok(entry)
        }
        Err(reason) => {
            increment(stats.reasons.entry(reason.clone()).or_default())?;
            Ok(deferred(reason))
        }
    }
}

fn deferred(reason: String) -> OperationOverlay {
    OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some(reason),
        ..OperationOverlay::default()
    }
}

fn increment(value: &mut usize) -> Result<(), OverlayError> {
    *value = value.checked_add(1).ok_or_else(|| invalid("candidate count overflow"))?;
    Ok(())
}

fn invalid(reason: &str) -> OverlayError {
    OverlayError::Invalid(reason.to_owned())
}
