use std::{borrow::Cow, sync::Arc};

use inventory_core::{
    catalog::runtime::{CallablePlan, ReferenceOwnerRule, RelationSource, RelationshipPlan, RuntimeCatalog},
    inventory::{
        DiscoveryContext, Observation, ReferenceTarget, Region, RegionId, ResourceId, ResourceKey,
        UnresolvedReference,
    },
    record::{Record, ValueRef},
};

use crate::sweep_runtime::{SweepError, record, resource_projection};

const MAX_PAGE_ROWS: usize = 100_000;
const MAX_PAGE_REFERENCES: usize = 10_000;


pub(super) fn observations(
    catalog: &RuntimeCatalog<'_>,
    identity: record::IdentityContext<'_>,
    plan: &CallablePlan<'_>,
    region: &Region,
    rows: &[Record],
    request: &Record,
    contexts: &[DiscoveryContext],
) -> Result<Vec<Observation>, SweepError> {
    if rows.len() > MAX_PAGE_ROWS {
        return Err(SweepError::Operation(
            "page exceeds the atomic observation bound",
        ));
    }
    if contexts.len() != rows.len() {
        return Err(SweepError::Operation("row contexts and observations have different lengths"));
    }
    let mut observations = Vec::new();
    observations.try_reserve(rows.len())?;
    for (row, ancestry) in rows.iter().zip(contexts) {
        observations.push(observation(
            catalog,
            record::IdentityContext { account:identity.account, region:identity.region },
            plan, region, row, request, ancestry,
        )?);
    }
    Ok(observations)
}

pub(super) fn tags(
    plan: &CallablePlan<'_>,
    rows: &[Record],
    observations: &[Observation],
) -> Result<Vec<inventory_core::inventory::TagObservation>, SweepError> {
    let Some(tags) = plan.tags() else {
        return Ok(Vec::new());
    };
    if rows.len() != observations.len() {
        return Err(SweepError::Operation("tag rows and observations have different lengths"));
    }
    rows.iter().zip(observations).try_fold(
        (Vec::new(), 0_usize),
        |(mut snapshots, count), (row, observation)| {
            let snapshot = record::tag_observation(row, observation.key(), tags)?;
            let count = count.checked_add(snapshot.tags().len())
                .filter(|count| *count <= 50_000)
                .ok_or(SweepError::Operation("page exceeds the atomic tag pair bound"))?;
            snapshots.try_reserve(1)?;
            snapshots.push(snapshot);
            Ok((snapshots, count))
        },
    ).map(|(snapshots, _)| snapshots)
}

pub(super) fn observation(
    catalog: &RuntimeCatalog<'_>,
    identity_context: record::IdentityContext<'_>,
    plan: &CallablePlan<'_>,
    region: &Region,
    row: &Record,
    request: &Record,
    ancestry: &DiscoveryContext,
) -> Result<Observation, SweepError> {
    let account = identity_context.account;
    let id = record::resource_id_with_context(
        row, plan.identity(), Some(identity_context), ancestry, request,
    )?;
    let key = ResourceKey {
        account: account.clone(),
        region: region.clone(),
        kind: resource_projection::classify(plan, row)?,
        id,
    };
    let name = resource_name(catalog, &key.kind, row)?;
    Observation::new(key, row.clone(), plan.hash_exclusions(), name)?
        .with_facet_precedence(plan.facet_precedence())
        .map_err(SweepError::from)
}

fn resource_name(
    catalog: &RuntimeCatalog<'_>,
    kind: &inventory_core::inventory::Kind,
    row: &Record,
) -> Result<Option<Arc<str>>, SweepError> {
    let Some(path) = catalog
        .name_paths()
        .iter()
        .find(|entry| &entry.kind == kind)
    else {
        return Ok(None);
    };
    match record::path_value(row.root(), &path.path)? {
        None | Some(ValueRef::Null) => Ok(None),
        Some(ValueRef::Text(value)) => Ok(Some(Arc::<str>::from(value))),
        Some(_) => Err(SweepError::Operation(
            "reviewed resource name has non-text value",
        )),
    }
}

pub(in crate::sweep_runtime) fn references(
    catalog: &RuntimeCatalog<'_>,
    plan: &CallablePlan<'_>,
    home: &RegionId,
    rows: &[Record],
    observations: &[Observation],
    actual_request: &Record,
) -> Result<Vec<UnresolvedReference>, SweepError> {
    let mut output = Vec::new();
    for (row, observation) in rows.iter().zip(observations) {
        for relation in catalog
            .relationships()
            .iter()
            .filter(|relation| relation.source_kind == observation.key().kind)
        {
            if !plan.supports_relationship(relation)? {
                continue;
            }
            if let Some(reference) = reference(catalog, row, actual_request, observation, home, relation)? {
                if output.len() >= MAX_PAGE_REFERENCES {
                    return Err(SweepError::Operation(
                        "page exceeds the atomic reference bound",
                    ));
                }
                output.try_reserve(1)?;
                output.push(reference);
            }
        }
        let trust = resource_projection::trust_references(catalog, plan, observation)?;
        if output.len().checked_add(trust.len()).is_none_or(|count| count > MAX_PAGE_REFERENCES) {
            return Err(SweepError::Operation("page exceeds the atomic reference bound"));
        }
        output.try_reserve(trust.len())?;
        output.extend(trust);
    }
    Ok(output)
}

fn reference(
    catalog: &RuntimeCatalog<'_>,
    row: &Record,
    actual_request: &Record,
    observation: &Observation,
    home: &RegionId,
    relation: &RelationshipPlan,
) -> Result<Option<UnresolvedReference>, SweepError> {
    let source = match relation.source {
        RelationSource::SelectedOutput => row,
        RelationSource::Request => actual_request,
    };
    let Some(value) = record::path_value(source.root(), &relation.source_path)? else {
        return Ok(None);
    };

    if relation.source == RelationSource::Request
        && !matches!(value, ValueRef::Text(_) | ValueRef::Null)
    {
        return Err(SweepError::Operation("request relationship source is not modeled text"));
    }
    let Some(text) = reference_text(value)? else {
        return Ok(None);
    };
    if relation.source == RelationSource::Request
        && matches!(relation.owner, ReferenceOwnerRule::ExecutionAccount)
        && text.starts_with("arn:")
    {
        record::trusted_arn::validate_arn(
            &text,
            Some(&record::IdentityContext { account: &observation.key().account, region: home }),
        )?;
    }
    let target_id = ResourceId::parse(&text).map_err(SweepError::from)?;
    let (target_account, target_region) = resource_projection::reference_owner(relation, observation, &text, home)?;
    let target = if relation.target_path.is_empty() {
        let schema = catalog.schemas().iter().find(|schema| schema.kind() == &relation.target_kind)
            .ok_or(SweepError::Operation("reference target kind has no checked schema"))?;
        ReferenceTarget::root_text(target_account, target_region, schema, target_id)?
    } else {
        ReferenceTarget::new(target_account, target_region, relation.target_kind.clone(),
            relation.target_path.to_vec(), target_id)?
    };
    let target = if relation.target_kinds.len() > 1 {
        target.with_candidate_kinds(relation.target_kinds.to_vec())?
    } else { target };
    Ok(Some(UnresolvedReference {
        from: observation.key().clone(),
        relation: relation.relation.clone(),
        source: inventory_core::inventory::EdgeSource::ReviewedRule,
        target,
    }))
}

fn reference_text(value: ValueRef<'_>) -> Result<Option<Cow<'_, str>>, SweepError> {
    match value {
        ValueRef::Null => Ok(None),
        ValueRef::Text("") => Ok(None),
        ValueRef::Text(value) => Ok(Some(Cow::Borrowed(value))),
        ValueRef::Int(value) => Ok(Some(Cow::Owned(value.to_string()))),
        _ => Err(SweepError::Operation(
            "relationship source is not a text or integer reference",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_null_and_empty_relationships_are_skipped() -> Result<(), Box<dyn std::error::Error>>
    {
        assert_eq!(reference_text(ValueRef::Null)?, None);
        assert_eq!(reference_text(ValueRef::Text(""))?, None);
        Ok(())
    }
}

#[cfg(test)]
#[path = "rows/facet_tests.rs"]
mod facet_tests;

#[cfg(test)]
#[path = "rows/request_relation_tests.rs"]
mod request_relation_tests;

#[cfg(test)]
#[path = "rows/trust_tests.rs"]
mod trust_tests;

#[cfg(test)]
#[path = "rows/constraint_dispatch_tests.rs"]
mod constraint_dispatch_tests;

#[cfg(test)]
#[path = "rows/typed_capabilities_tests.rs"]
mod typed_capabilities_tests;
