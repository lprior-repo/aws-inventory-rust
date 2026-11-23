use super::{
    CallablePlan, DiscoveryContext, Observation, Record, Region, ResourceKey, RuntimeCatalog,
    SweepError, record, resource_name,
};
use inventory_core::{catalog::runtime::ResourceEmissionPlan, inventory::TagObservation};

pub(crate) struct EmissionScope<'a> {
    pub catalog: &'a RuntimeCatalog<'a>,
    pub identity_context: record::IdentityContext<'a>,
    pub plan: &'a CallablePlan<'a>,
    pub emission: &'a ResourceEmissionPlan,
    pub region: &'a Region,
    pub request: &'a Record,
    pub ancestry: &'a DiscoveryContext,
}

pub(crate) fn emission_observation(
    scope: EmissionScope<'_>,
    row: &Record,
) -> Result<Observation, SweepError> {
    let key = emission_key(&scope, row)?;
    let name = resource_name(scope.catalog, &key.kind, row)?;
    Ok(
        Observation::new(key, row.clone(), scope.emission.hash_exclusions(), name)?
            .with_facet_precedence(scope.plan.facet_precedence()),
    )
}

pub(crate) fn emission_tags(
    emission: &ResourceEmissionPlan,
    rows: &[Record],
    observations: &[Observation],
) -> Result<Vec<TagObservation>, SweepError> {
    let Some(tags) = emission.tags() else {
        return Ok(Vec::new());
    };
    tag_observations(tags, rows, observations)
}

fn emission_key(scope: &EmissionScope<'_>, row: &Record) -> Result<ResourceKey, SweepError> {
    let account = scope.identity_context.account.clone();
    let id = emission_id(scope, row)?;
    let kind = match scope.emission.classification() {
        Some(classification) => classification.classify(row)?.clone(),
        None => scope.emission.kind().clone(),
    };
    record::trusted_arn::validate_kind_arn(&kind, id.as_str())?;
    Ok(ResourceKey {
        account,
        region: scope.region.clone(),
        kind,
        id,
    })
}

fn emission_id(
    scope: &EmissionScope<'_>,
    row: &Record,
) -> Result<inventory_core::inventory::ResourceId, SweepError> {
    let validation =
        record::IdentityTextValidation::Modeled(scope.plan.identity_text(scope.emission.key())?);
    record::resource_id_validated(
        row,
        scope.emission.identity(),
        Some(record::IdentityContext {
            ..scope.identity_context
        }),
        scope.ancestry,
        scope.request,
        &validation,
    )
}

pub(super) fn tag_observations(
    tags: &inventory_core::catalog::runtime::TagPlan,
    rows: &[Record],
    observations: &[Observation],
) -> Result<Vec<TagObservation>, SweepError> {
    if rows.len() != observations.len() {
        return Err(SweepError::Operation(
            "tag rows and observations have different lengths",
        ));
    }
    rows.iter()
        .zip(observations)
        .try_fold((Vec::new(), 0_usize), |state, (row, observation)| {
            append_tags(state, row, observation, tags)
        })
        .map(|(snapshots, _)| snapshots)
}

fn append_tags(
    (mut snapshots, count): (Vec<TagObservation>, usize),
    row: &Record,
    observation: &Observation,
    tags: &inventory_core::catalog::runtime::TagPlan,
) -> Result<(Vec<TagObservation>, usize), SweepError> {
    let snapshot = record::tag_observation(row, observation.key(), tags)?;
    let count = count
        .checked_add(snapshot.tags().len())
        .filter(|count| *count <= 50_000)
        .ok_or(SweepError::Operation(
            "page exceeds the atomic tag pair bound",
        ))?;
    snapshots.try_reserve(1)?;
    snapshots.push(snapshot);
    Ok((snapshots, count))
}
