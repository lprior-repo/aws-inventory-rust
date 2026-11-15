use std::sync::Arc;

use inventory_core::{
    catalog::runtime::{CallablePlan, RelationScope, RelationshipPlan, RuntimeCatalog},
    inventory::{
        Observation, ReferenceTarget, Region, RegionId, ResourceId, ResourceKey,
        UnresolvedReference,
    },
    record::{Record, ValueRef},
};

use super::super::RuntimeContext;
use crate::sweep_runtime::{SweepError, record};

const MAX_PAGE_ROWS: usize = 1_000;
const MAX_PAGE_REFERENCES: usize = 10_000;

pub(super) fn items(plan: &CallablePlan<'_>, response: &Record) -> Result<Vec<Record>, SweepError> {
    record::item_records(response, plan.items())
}

pub(super) fn observations(
    context: &RuntimeContext<'_, '_>,
    plan: &CallablePlan<'_>,
    region: &Region,
    rows: &[Record],
) -> Result<Vec<Observation>, SweepError> {
    if rows.len() > MAX_PAGE_ROWS {
        return Err(SweepError::Operation(
            "page exceeds the atomic observation bound",
        ));
    }
    let mut observations = Vec::new();
    observations.try_reserve(rows.len())?;
    for row in rows {
        observations.push(observation(context, plan, region, row)?);
    }
    Ok(observations)
}

fn observation(
    context: &RuntimeContext<'_, '_>,
    plan: &CallablePlan<'_>,
    region: &Region,
    row: &Record,
) -> Result<Observation, SweepError> {
    let id = record::resource_id(row, plan.identity())?;
    let key = ResourceKey {
        account: context.account.clone(),
        region: region.clone(),
        kind: plan.kind().clone(),
        id,
    };
    let name = resource_name(context.bootstrap.catalog, plan, row)?;
    Observation::new(key, row.clone(), plan.hash_exclusions(), name).map_err(SweepError::from)
}

fn resource_name(
    catalog: &RuntimeCatalog<'_>,
    plan: &CallablePlan<'_>,
    row: &Record,
) -> Result<Option<Arc<str>>, SweepError> {
    let Some(path) = catalog
        .name_paths()
        .iter()
        .find(|entry| entry.kind == *plan.kind())
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

pub(super) fn references(
    catalog: &RuntimeCatalog<'_>,
    home: &RegionId,
    rows: &[Record],
    observations: &[Observation],
) -> Result<Vec<UnresolvedReference>, SweepError> {
    let mut output = Vec::new();
    for (row, observation) in rows.iter().zip(observations) {
        for relation in catalog
            .relationships()
            .iter()
            .filter(|relation| relation.source_kind == observation.key().kind)
        {
            if let Some(reference) = reference(row, observation, home, relation)? {
                if output.len() >= MAX_PAGE_REFERENCES {
                    return Err(SweepError::Operation(
                        "page exceeds the atomic reference bound",
                    ));
                }
                output.try_reserve(1)?;
                output.push(reference);
            }
        }
    }
    Ok(output)
}

fn reference(
    row: &Record,
    observation: &Observation,
    home: &RegionId,
    relation: &RelationshipPlan,
) -> Result<Option<UnresolvedReference>, SweepError> {
    let Some(value) = record::path_value(row.root(), &relation.source_path)? else {
        return Ok(None);
    };

    let Some(text) = reference_text(value)? else {
        return Ok(None);
    };
    let target_id = ResourceId::parse(&text).map_err(SweepError::from)?;
    let target_region = match relation.scope {
        RelationScope::AccountRegion => observation.key().region.clone(),
        RelationScope::Account => Region::Global,
        RelationScope::HomeRegion => Region::Regional(home.clone()),
    };
    let target = ReferenceTarget::new(
        observation.key().account.clone(),
        target_region,
        relation.target_kind.clone(),
        relation.target_path.to_vec(),
        target_id,
    )
    .map_err(SweepError::from)?;
    Ok(Some(UnresolvedReference {
        from: observation.key().clone(),
        relation: relation.relation.clone(),
        source: inventory_core::inventory::EdgeSource::ReviewedRule,
        target,
    }))
}

fn reference_text(value: ValueRef<'_>) -> Result<Option<String>, SweepError> {
    match value {
        ValueRef::Null => Ok(None),
        ValueRef::Text("") => Ok(None),
        ValueRef::Text(value) => Ok(Some(value.to_owned())),
        ValueRef::Int(value) => Ok(Some(value.to_string())),
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
