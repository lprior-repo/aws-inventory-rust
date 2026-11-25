use super::{StoreError, matches_reference, scalar};
use crate::inventory::WorkLease;
use inventory_core::{
    catalog::runtime::{RelationPathMode, RelationSource, RelationTargetMode, RelationshipPlan},
    inventory::{
        Edge, Observation, ReferenceComponent, ReferenceTarget, ResourceId, UnresolvedReference,
    },
    record::{FieldId, MapRef, ValueRef},
};
use std::borrow::Cow;

fn request_root<'a>(bindings: &'a [u8]) -> Result<MapRef<'a>, StoreError> {
    if bindings.is_empty() {
        return Ok(MapRef::from_bytes_bounded(&[0xA0])?);
    }
    Ok(MapRef::from_bytes_bounded(bindings)?)
}

pub(super) fn source_value<'a>(
    source_mode: RelationSource,
    path_mode: RelationPathMode,
    path: &[FieldId],
    lease: &'a WorkLease,
    source: &'a Observation,
) -> Result<Option<Cow<'a, str>>, StoreError> {
    if source_mode == RelationSource::Parent {
        return Ok(lease
            .scope()
            .parent()
            .map(|parent| Cow::Borrowed(parent.id.as_str())));
    }
    let root = match source_mode {
        RelationSource::SelectedOutput => source.record().root(),
        RelationSource::Request => request_root(lease.bindings())?,
        RelationSource::Parent => return Ok(None),
    };
    if (path_mode == RelationPathMode::RootText) != path.is_empty() {
        return Ok(None);
    }
    Ok(match scalar(root, path)? {
        Some(ValueRef::Text(value)) => Some(Cow::Borrowed(value)),
        Some(ValueRef::Int(value)) if source_mode == RelationSource::SelectedOutput => {
            Some(Cow::Owned(value.to_string()))
        }
        _ => None,
    })
}

fn primary_matches(
    rule: &RelationshipPlan,
    lease: &WorkLease,
    source: &Observation,
    value: &str,
) -> Result<bool, StoreError> {
    if rule.source != RelationSource::Parent {
        if (rule.source_path_mode == RelationPathMode::RootText) != rule.source_path.is_empty() {
            return Ok(false);
        }
        let root = match rule.source {
            RelationSource::SelectedOutput => source.record().root(),
            RelationSource::Request => request_root(lease.bindings())?,
            RelationSource::Parent => return Ok(false),
        };
        if let Some(ValueRef::List(values)) = scalar(root, &rule.source_path)? {
            if !rule.components.is_empty() {
                return Ok(false);
            }
            let mut found = false;
            for member in values.iter()? {
                match member? {
                    ValueRef::Null => {}
                    ValueRef::Text(text) => found |= !text.is_empty() && text == value,
                    _ => {
                        return Err(StoreError::Invalid(
                            "relationship source list element is not modeled text",
                        ));
                    }
                }
            }
            return Ok(found);
        }
    }
    Ok(source_value(
        rule.source,
        rule.source_path_mode,
        &rule.source_path,
        lease,
        source,
    )?
    .is_some_and(|actual| actual == value))
}

pub(super) fn source_matches(
    rule: &RelationshipPlan,
    lease: &WorkLease,
    source: &Observation,
    reference: &UnresolvedReference,
) -> Result<bool, StoreError> {
    if rule.source == RelationSource::Parent
        && !lease.scope().parent().is_some_and(|parent| {
            parent.account == *reference.target.account()
                && parent.region == *reference.target.region()
                && parent.kind == *reference.target.kind()
                && parent.id == *reference.target.value()
        })
    {
        return Ok(false);
    }
    if let Some(guard) = &rule.guard
        && !source_value(guard.source, guard.path_mode, &guard.path, lease, source)?
            .is_some_and(|value| value == guard.equals)
    {
        return Ok(false);
    }
    if !primary_matches(rule, lease, source, reference.target.value().as_str())? {
        return Ok(false);
    }
    rule.components
        .iter()
        .zip(reference.target.components())
        .try_fold(true, |matches, (rule, term)| {
            if !matches {
                return Ok(false);
            }
            if rule.source == RelationSource::Parent
                && !lease.scope().parent().is_some_and(|parent| {
                    parent.account == *reference.target.account()
                        && parent.region == *reference.target.region()
                        && parent.kind == *reference.target.kind()
                        && parent.id == term.value
                })
            {
                return Ok(false);
            }
            Ok(source_value(
                rule.source,
                rule.source_path_mode,
                &rule.source_path,
                lease,
                source,
            )?
            .is_some_and(|value| value == term.value.as_str()))
        })
}

pub(super) fn rule_reference(
    rule: &RelationshipPlan,
    lease: &WorkLease,
    source: &Observation,
    target: &Observation,
    edge: &Edge,
) -> Result<Option<UnresolvedReference>, StoreError> {
    let kind_matches = if rule.source == RelationSource::Parent {
        rule.target_kinds.contains(&edge.to.kind)
    } else {
        rule.target_kind == edge.to.kind && rule.target_kinds.len() == 1
    };
    if rule.source_kind != edge.from.kind || rule.relation != edge.relation || !kind_matches {
        return Ok(None);
    }
    let Some(value) = target_value(
        rule.target_mode,
        &rule.target_path,
        target,
        lease.bindings(),
    )?
    else {
        return Ok(None);
    };
    let components = rule
        .components
        .iter()
        .map(|term| {
            source_value(
                term.source,
                term.source_path_mode,
                &term.source_path,
                lease,
                source,
            )?
            .map(|value| {
                Ok(ReferenceComponent {
                    mode: term.target_mode,
                    path: term.target_path.clone(),
                    value: ResourceId::parse(&value)?,
                })
            })
            .transpose()
        })
        .collect::<Result<Option<Vec<_>>, StoreError>>()?;
    let Some(components) = components else {
        return Ok(None);
    };
    let target = ReferenceTarget::typed(
        edge.to.account.clone(),
        edge.to.region.clone(),
        edge.to.kind.clone(),
        rule.target_path.to_vec(),
        ResourceId::parse(value)?,
        rule.target_mode,
    )?
    .with_components(components)?;
    let target = if rule.target_kinds.len() > 1 {
        target.with_candidate_kinds(rule.target_kinds.to_vec())?
    } else {
        target
    };
    let reference = UnresolvedReference {
        from: edge.from.clone(),
        relation: edge.relation.clone(),
        source: edge.source,
        target,
    };
    if matches_reference(rule, &reference) && source_matches(rule, lease, source, &reference)? {
        Ok(Some(reference))
    } else {
        Ok(None)
    }
}

fn target_value<'a>(
    mode: RelationTargetMode,
    path: &[FieldId],
    target: &'a Observation,
    request: &'a [u8],
) -> Result<Option<&'a str>, StoreError> {
    let root = match mode {
        RelationTargetMode::CanonicalId => return Ok(Some(target.key().id.as_str())),
        RelationTargetMode::SelectedFields | RelationTargetMode::SelectedRootText => {
            target.record().root()
        }
        RelationTargetMode::BoundContextFields | RelationTargetMode::BoundContextRootText => {
            request_root(request)?
        }
    };
    Ok(match scalar(root, path)? {
        Some(ValueRef::Text(value)) => Some(value),
        _ => None,
    })
}

pub(super) fn reference_matches_observation(
    reference: &UnresolvedReference,
    target: &Observation,
    request: &[u8],
) -> Result<bool, StoreError> {
    let primary = (
        reference.target.mode(),
        reference.target.path(),
        reference.target.value(),
    );
    std::iter::once(primary)
        .chain(
            reference
                .target
                .components()
                .iter()
                .map(|term| (term.mode, term.path.as_ref(), &term.value)),
        )
        .try_fold(true, |matches, (mode, path, value)| {
            if !matches {
                return Ok(false);
            }
            Ok(target_value(mode, path, target, request)?
                .is_some_and(|text| text == value.as_str()))
        })
}
