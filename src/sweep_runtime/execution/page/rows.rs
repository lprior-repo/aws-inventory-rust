use std::sync::Arc;

use inventory_core::{
    catalog::runtime::{CallablePlan, RuntimeCatalog},
    inventory::{
        DiscoveryContext, Observation, Region, RegionId, ResourceKey, UnresolvedReference,
    },
    record::{Record, ValueRef},
};

use crate::sweep_runtime::{SweepError, record, resource_projection};

const MAX_PAGE_ROWS: usize = 100_000;
const MAX_PAGE_REFERENCES: usize = 10_000;

mod relations;

#[cfg(test)]
use inventory_core::{catalog::runtime::RelationSource, inventory::{ReferenceTarget, ResourceId}};


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
    Ok(Observation::new(key, row.clone(), plan.hash_exclusions(), name)?
        .with_facet_precedence(plan.facet_precedence()))
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
    parent: Option<&ResourceKey>,
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
            relations::Context { row, request: actual_request, observation, home, parent }
                .append(relation, &mut output, MAX_PAGE_REFERENCES)?;
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
