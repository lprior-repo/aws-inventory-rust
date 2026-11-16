use super::super::{ReviewDecision, ReviewDisposition};
use super::{RuntimeCatalogError, integer_value, lookup, map, text_value};
use crate::record::{FieldId, KeyRef, ListRef, MapRef, ValueRef};
use std::collections::BTreeSet;

pub(super) const MAX_ITEMS: usize = 65_536;
pub(super) const MAX_COLUMNS: usize = 4096;

pub(super) fn integer(map_value: MapRef<'_>, key: u32) -> Result<i64, RuntimeCatalogError> {
    integer_value(lookup(map_value, key)?)
}

pub(super) fn text(map_value: MapRef<'_>, key: u32) -> Result<String, RuntimeCatalogError> {
    text_value(lookup(map_value, key)?)
}

pub(super) fn list(map_value: MapRef<'_>, key: u32) -> Result<ListRef<'_>, RuntimeCatalogError> {
    match lookup(map_value, key)? {
        ValueRef::List(value) => Ok(value),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

pub(super) fn map_field(
    map_value: MapRef<'_>,
    key: u32,
) -> Result<MapRef<'_>, RuntimeCatalogError> {
    map(lookup(map_value, key)?)
}

pub(super) fn field(map_value: MapRef<'_>, key: u32) -> Result<ValueRef<'_>, RuntimeCatalogError> {
    lookup(map_value, key)
}

pub(super) fn bounded<T>(
    list: ListRef<'_>,
    mut parse: impl FnMut(ValueRef<'_>) -> Result<T, RuntimeCatalogError>,
    limit: usize,
) -> Result<Vec<T>, RuntimeCatalogError> {
    let mut values = Vec::new();
    list.iter()?.try_for_each(|entry| {
        if values.len() >= limit {
            return Err(RuntimeCatalogError::Bounds);
        }
        values.push(parse(entry?)?);
        Ok::<_, RuntimeCatalogError>(())
    })?;
    Ok(values)
}

pub(super) fn only_fields(
    map_value: MapRef<'_>,
    allowed: &[u32],
) -> Result<(), RuntimeCatalogError> {
    map_value.iter()?.try_for_each(|entry| {
        let (key, _) = entry?;
        match key {
            KeyRef::Field(id) if allowed.contains(&id.value()) => Ok(()),
            _ => Err(RuntimeCatalogError::Capability),
        }
    })
}

pub(super) fn id_path(list: ListRef<'_>) -> Result<Box<[FieldId]>, RuntimeCatalogError> {
    let fields = selected_path(list)?;
    if fields.is_empty() {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(fields)
}

pub(super) fn selected_path(list: ListRef<'_>) -> Result<Box<[FieldId]>, RuntimeCatalogError> {
    let fields = bounded(
        list,
        |value| {
            let raw = integer_value(value)?;
            let number = u32::try_from(raw).map_err(|_| RuntimeCatalogError::Capability)?;
            FieldId::new(number).map_err(RuntimeCatalogError::Record)
        },
        8,
    )?;
    Ok(fields.into_boxed_slice())
}

pub(super) fn relations(
    list: ListRef<'_>,
) -> Result<Vec<super::super::RelationshipPlan>, RuntimeCatalogError> {
    bounded(list, relation, MAX_ITEMS)
}

fn relation(value: ValueRef<'_>) -> Result<super::super::RelationshipPlan, RuntimeCatalogError> {
    let fields = map(value)?;
    let target_kind = crate::inventory::Kind::parse(&text(fields, 3)?)?;
    let target_kinds = match fields.get_field(FieldId::new(9)?)? {
        None => vec![target_kind.clone()],
        Some(ValueRef::List(values)) => bounded(values, |value| {
            crate::inventory::Kind::parse(&super::text_value(value)?).map_err(RuntimeCatalogError::from)
        }, 64)?,
        Some(_) => return Err(RuntimeCatalogError::Capability),
    };
    if target_kinds.is_empty() || !target_kinds.contains(&target_kind)
        || target_kinds.iter().enumerate().any(|(index, kind)| target_kinds.iter().take(index).any(|prior| prior == kind)) {
        return Err(RuntimeCatalogError::Capability);
    }
    let result = super::super::RelationshipPlan {
        source_kind: crate::inventory::Kind::parse(&text(fields, 1)?)?,
        source_path: selected_path(list(fields, 2)?)?,
        source_path_mode: relation_modes::source_mode(fields)?,
        target_mode: relation_modes::target_mode(fields)?,
        guard: relation_modes::guard(fields)?,
        components: relation_modes::components(fields)?,
        source: match fields.get_field(FieldId::new(10)?)? {
            None | Some(ValueRef::Int(0)) => super::super::RelationSource::SelectedOutput,
            Some(ValueRef::Int(1)) => super::super::RelationSource::Request,
            Some(ValueRef::Int(2)) => super::super::RelationSource::Parent,
            Some(_) => return Err(RuntimeCatalogError::Capability),
        },
        target_kind,
        target_kinds: target_kinds.into_boxed_slice(),
        target_path: selected_path(list(fields, 4)?)?,
        relation: crate::inventory::Relation::parse(&text(fields, 5)?)?,
        scope: match integer(fields, 6)? {
            0 => super::super::RelationScope::AccountRegion,
            1 => super::super::RelationScope::Account,
            2 => super::super::RelationScope::HomeRegion,
            _ => return Err(RuntimeCatalogError::Capability),
        },
        owner: super::owners::read_owner(fields)?,
    };
    relation_modes::validate(&result)?;
    only_fields(fields, &[1, 2, 3, 4, 5, 6, 8, 9, 10, 11, 12, 13, 14])?;
    Ok(result)
}

mod relation_modes;

pub(super) fn coverage(
    list: ListRef<'_>,
) -> Result<Vec<super::super::KindCoverage>, RuntimeCatalogError> {
    let entries = bounded(list, coverage_entry, MAX_ITEMS)?;
    let mut names = BTreeSet::new();
    let unique = entries.iter().all(|entry| {
        entry
            .cfn_type
            .as_ref()
            .is_none_or(|name| names.insert(name.clone()))
    });
    if !unique {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(entries)
}

fn coverage_entry(value: ValueRef<'_>) -> Result<super::super::KindCoverage, RuntimeCatalogError> {
    let fields = map(value)?;
    let cfn_type = optional_text(&fields, 1)?;
    let kind = optional_text(&fields, 2)?
        .map(|value| crate::inventory::Kind::parse(&value))
        .transpose()?;
    let mapped = boolean(&fields, 3)?;
    let reason = optional_text(&fields, 4)?;
    let api_only = boolean(&fields, 5)?;
    only_fields(fields, &[1, 2, 3, 4, 5])?;
    if mapped != kind.is_some() || mapped == reason.is_some() || api_only != cfn_type.is_none() {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(super::super::KindCoverage {
        cfn_type,
        kind,
        mapped,
        reason,
        api_only,
    })
}
pub(super) fn decisions(
    list: ListRef<'_>,
    plans: &[super::super::CallablePlan<'_>],
    services: &super::services::Services<'_>,
) -> Result<Vec<ReviewDecision>, RuntimeCatalogError> {
    let decisions = bounded(list, decision, MAX_ITEMS)?;
    decisions.iter().map(decision_key).try_for_each(|(service, operation)| {
        services.operation(service, operation).map(|_| ())
    })?;
    validate_decision_plans(&decisions, plans)?;
    Ok(decisions)
}

fn validate_decision_plans(
    decisions: &[ReviewDecision],
    plans: &[super::super::CallablePlan<'_>],
) -> Result<(), RuntimeCatalogError> {
    let plan_keys = plans
        .iter()
        .map(|plan| (plan.service_name(), plan.operation_name()))
        .collect::<Vec<_>>();
    if decisions_match_plan_keys(decisions, &plan_keys) {
        Ok(())
    } else {
        Err(RuntimeCatalogError::Capability)
    }
}

fn decisions_match_plan_keys(decisions: &[ReviewDecision], plan_keys: &[(&str, &str)]) -> bool {
    let decision_keys = decisions.iter().map(decision_key).collect::<BTreeSet<_>>();
    let callable_keys = decisions
        .iter()
        .filter(|value| value.disposition == ReviewDisposition::Callable)
        .map(decision_key)
        .collect::<BTreeSet<_>>();
    let callable_count = decisions
        .iter()
        .filter(|value| value.disposition == ReviewDisposition::Callable)
        .count();
    let plan_key_set = plan_keys.iter().copied().collect::<BTreeSet<_>>();
    decision_keys.len() == decisions.len()
        && callable_keys.len() == callable_count
        && plan_key_set.len() == plan_keys.len()
        && plan_key_set == callable_keys
}

fn decision_key(value: &ReviewDecision) -> (&str, &str) {
    (value.service.as_str(), value.operation.as_str())
}

fn decision(value: ValueRef<'_>) -> Result<ReviewDecision, RuntimeCatalogError> {
    let fields = map(value)?;
    let disposition = match integer(fields, 3)? {
        0 => ReviewDisposition::Callable,
        1 => ReviewDisposition::Denied {
            reason: required_reason(fields)?,
        },
        2 => ReviewDisposition::Deferred {
            reason: required_reason(fields)?,
        },
        3 => ReviewDisposition::Unclassified,
        _ => return Err(RuntimeCatalogError::Capability),
    };
    if matches!(
        disposition,
        ReviewDisposition::Callable | ReviewDisposition::Unclassified
    ) && optional_text(&fields, 4)?.is_some()
    {
        return Err(RuntimeCatalogError::Capability);
    }
    only_fields(fields, &[1, 2, 3, 4])?;
    Ok(ReviewDecision {
        service: text(fields, 1)?,
        operation: text(fields, 2)?,
        disposition,
    })
}

fn required_reason(fields: MapRef<'_>) -> Result<String, RuntimeCatalogError> {
    optional_text(&fields, 4)?.ok_or(RuntimeCatalogError::Capability)
}

fn optional_text(map_value: &MapRef<'_>, key: u32) -> Result<Option<String>, RuntimeCatalogError> {
    match map_value.get_field(FieldId::new(key)?)? {
        Some(ValueRef::Text(value)) if !value.is_empty() && value.len() <= 16_384 => {
            Ok(Some(value.to_owned()))
        }
        Some(ValueRef::Null) | None => Ok(None),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn boolean(map_value: &MapRef<'_>, key: u32) -> Result<bool, RuntimeCatalogError> {
    match map_value.get_field(FieldId::new(key)?)? {
        Some(ValueRef::Bool(value)) => Ok(value),
        _ => Err(RuntimeCatalogError::Capability),
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callable_plan_keys_must_equal_callable_decision_keys() {
        let decisions = [
            ReviewDecision {
                service: "svc".to_owned(),
                operation: "A".to_owned(),
                disposition: ReviewDisposition::Callable,
            },
            ReviewDecision {
                service: "svc".to_owned(),
                operation: "B".to_owned(),
                disposition: ReviewDisposition::Callable,
            },
            ReviewDecision {
                service: "svc".to_owned(),
                operation: "C".to_owned(),
                disposition: ReviewDisposition::Denied {
                    reason: "denied".to_owned(),
                },
            },
        ];
        assert!(!decisions_match_plan_keys(
            &decisions,
            &[("svc", "A"), ("svc", "C")]
        ));
    }

    #[test]
    fn duplicate_plan_keys_are_rejected() {
        let decisions = [
            ReviewDecision {
                service: "svc".to_owned(),
                operation: "A".to_owned(),
                disposition: ReviewDisposition::Callable,
            },
            ReviewDecision {
                service: "svc".to_owned(),
                operation: "B".to_owned(),
                disposition: ReviewDisposition::Callable,
            },
        ];
        assert!(!decisions_match_plan_keys(
            &decisions,
            &[("svc", "A"), ("svc", "A")]
        ));
    }
}

#[cfg(test)]
mod relationship_source_tests;
