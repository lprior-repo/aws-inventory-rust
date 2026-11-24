use std::sync::Arc;

use inventory_core::{
    catalog::runtime::{CallablePlan, RuntimeCatalog},
    inventory::{
        DiscoveryContext, Observation, Region, RegionId, ResourceKey, UnresolvedReference,
    },
    record::{Record, ValueRef},
};

use crate::{SweepError, record, resource_projection};

const MAX_PAGE_ROWS: usize = 100_000;
const MAX_PAGE_REFERENCES: usize = 10_000;

mod emission;
mod relations;

pub(crate) use emission::{EmissionScope, emission_identity, emission_tags};

#[cfg(test)]
use inventory_core::{
    catalog::runtime::RelationSource,
    inventory::{ReferenceTarget, ResourceId},
};

pub(crate) struct RowIdentity {
    key: ResourceKey,
    name: Option<Arc<str>>,
}

impl AsRef<ResourceKey> for RowIdentity {
    fn as_ref(&self) -> &ResourceKey {
        &self.key
    }
}

pub(super) fn identities(
    catalog: &RuntimeCatalog<'_>,
    identity: record::IdentityContext<'_>,
    plan: &CallablePlan<'_>,
    region: &Region,
    rows: &[Record],
    request: &Record,
    contexts: &[DiscoveryContext],
) -> Result<Vec<RowIdentity>, SweepError> {
    check_rows(rows, contexts)?;
    let mut identities = Vec::new();
    identities.try_reserve(rows.len())?;
    rows.iter().zip(contexts).try_for_each(|(row, ancestry)| {
        let key = primary_key(
            record::IdentityContext { ..identity },
            plan,
            region,
            row,
            request,
            ancestry,
        )?;
        let name = resource_name(catalog, &key.kind, row)?;
        identities.push(RowIdentity { key, name });
        Ok::<_, SweepError>(())
    })?;
    Ok(identities)
}

pub(super) fn observations(
    rows: Vec<Record>,
    identities: Vec<RowIdentity>,
    exclusions: &inventory_core::record::Exclusions,
    precedence: u16,
) -> Result<Vec<Observation>, SweepError> {
    if rows.len() > MAX_PAGE_ROWS || rows.len() != identities.len() {
        return Err(SweepError::Operation(
            "rows and observation identities exceed bounds or have different lengths",
        ));
    }
    let mut observations = Vec::new();
    observations.try_reserve(rows.len())?;
    rows.into_iter()
        .zip(identities)
        .try_for_each(|(row, identity)| {
            observations.push(
                Observation::new(identity.key, row, exclusions, identity.name)?
                    .with_facet_precedence(precedence),
            );
            Ok::<_, SweepError>(())
        })?;
    Ok(observations)
}

fn check_rows(rows: &[Record], contexts: &[DiscoveryContext]) -> Result<(), SweepError> {
    if rows.len() > MAX_PAGE_ROWS {
        return Err(SweepError::Operation(
            "page exceeds the atomic observation bound",
        ));
    }
    if contexts.len() != rows.len() {
        return Err(SweepError::Operation(
            "row contexts and observations have different lengths",
        ));
    }
    Ok(())
}

pub(super) fn tags(
    plan: &CallablePlan<'_>,
    rows: &[Record],
    observations: &[impl AsRef<ResourceKey>],
) -> Result<Vec<inventory_core::inventory::TagObservation>, SweepError> {
    let Some(tags) = plan.tags() else {
        return Ok(Vec::new());
    };
    emission::tag_observations(tags, rows, observations)
}

#[cfg(test)]
pub(super) fn observation(
    catalog: &RuntimeCatalog<'_>,
    identity_context: record::IdentityContext<'_>,
    plan: &CallablePlan<'_>,
    region: &Region,
    row: Record,
    request: &Record,
    ancestry: &DiscoveryContext,
) -> Result<Observation, SweepError> {
    let key = primary_key(identity_context, plan, region, &row, request, ancestry)?;
    let name = resource_name(catalog, &key.kind, &row)?;
    Ok(Observation::new(key, row, plan.hash_exclusions(), name)?
        .with_facet_precedence(plan.facet_precedence()))
}

fn primary_key(
    identity: record::IdentityContext<'_>,
    plan: &CallablePlan<'_>,
    region: &Region,
    row: &Record,
    request: &Record,
    ancestry: &DiscoveryContext,
) -> Result<ResourceKey, SweepError> {
    let account = identity.account;
    let id = primary_id(row, plan, identity, ancestry, request)?;
    let kind = resource_projection::classify(plan, row)?;
    record::trusted_arn::validate_kind_arn(&kind, id.as_str())?;
    Ok(ResourceKey {
        account: account.clone(),
        region: region.clone(),
        kind,
        id,
    })
}

fn primary_id(
    row: &Record,
    plan: &CallablePlan<'_>,
    identity: record::IdentityContext<'_>,
    ancestry: &DiscoveryContext,
    request: &Record,
) -> Result<inventory_core::inventory::ResourceId, SweepError> {
    let validation = record::IdentityTextValidation::Modeled(plan.identity_text("primary")?);
    record::resource_id_validated(
        row,
        plan.identity(),
        Some(identity),
        ancestry,
        request,
        &validation,
    )
}

fn resource_name(
    catalog: &RuntimeCatalog<'_>,
    kind: &inventory_core::inventory::Kind,
    row: &Record,
) -> Result<Option<Arc<str>>, SweepError> {
    let Some(path) = catalog.name_path(kind) else {
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

pub(crate) fn references(
    catalog: &RuntimeCatalog<'_>,
    plan: &CallablePlan<'_>,
    home: &RegionId,
    rows: &[Record],
    observations: &[impl AsRef<ResourceKey>],
    actual_request: &Record,
    parent: Option<&ResourceKey>,
) -> Result<Vec<UnresolvedReference>, SweepError> {
    let mut output = Vec::new();
    rows.iter()
        .zip(observations)
        .try_for_each(|(row, observation)| {
            relations::Context {
                row,
                request: actual_request,
                observation,
                home,
                parent,
            }
            .append_plan(catalog, plan, &mut output, MAX_PAGE_REFERENCES)?;
            append_trust(&mut output, catalog, plan, observation.as_ref(), row)
        })?;
    Ok(output)
}

fn append_trust(
    output: &mut Vec<UnresolvedReference>,
    catalog: &RuntimeCatalog<'_>,
    plan: &CallablePlan<'_>,
    key: &ResourceKey,
    row: &Record,
) -> Result<(), SweepError> {
    let trust = resource_projection::trust_references(catalog, plan, key, row)?;
    if output
        .len()
        .checked_add(trust.len())
        .is_none_or(|count| count > MAX_PAGE_REFERENCES)
    {
        return Err(SweepError::Operation(
            "page exceeds the atomic reference bound",
        ));
    }
    output.try_reserve(trust.len())?;
    output.extend(trust);
    Ok(())
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

#[cfg(test)]
#[path = "rows/ownership_tests.rs"]
mod ownership_tests;
