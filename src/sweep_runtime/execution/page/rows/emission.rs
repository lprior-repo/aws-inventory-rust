use super::{ResourceKey, Observation, Record, Region, DiscoveryContext, RuntimeCatalog, CallablePlan, SweepError, record, resource_name};
use inventory_core::{catalog::runtime::ResourceEmissionPlan, inventory::TagObservation};

pub(in crate::sweep_runtime) fn emission_observation(catalog: &RuntimeCatalog<'_>,
    identity_context: record::IdentityContext<'_>, plan: &CallablePlan<'_>,
    emission: &ResourceEmissionPlan, region: &Region, row: &Record,
    request: &Record, ancestry: &DiscoveryContext,
) -> Result<Observation, SweepError> {
    let account = identity_context.account.clone();
    let validation = record::IdentityTextValidation::Modeled(plan.identity_text(emission.key())?);
    let id = record::resource_id_validated(row, emission.identity(), Some(identity_context), ancestry, request, &validation)?;
    let kind = match emission.classification() {
        Some(classification) => classification.classify(row)?.clone(),
        None => emission.kind().clone(),
    };
    record::trusted_arn::validate_kind_arn(&kind, id.as_str())?;
    let key = ResourceKey { account, region: region.clone(), kind, id };
    let name = resource_name(catalog, &key.kind, row)?;
    Ok(Observation::new(key, row.clone(), emission.hash_exclusions(), name)?
        .with_facet_precedence(plan.facet_precedence()))
}

pub(in crate::sweep_runtime) fn emission_tags(emission: &ResourceEmissionPlan,
    rows: &[Record], observations: &[Observation],
) -> Result<Vec<TagObservation>, SweepError> {
    let Some(tags) = emission.tags() else { return Ok(Vec::new()); };
    if rows.len() != observations.len() {
        return Err(SweepError::Operation("tag rows and observations have different lengths"));
    }
    rows.iter().zip(observations).try_fold((Vec::new(), 0_usize), |(mut snapshots, count), (row, observation)| {
        let snapshot = record::tag_observation(row, observation.key(), tags)?;
        let count = count.checked_add(snapshot.tags().len()).filter(|count| *count <= 50_000)
            .ok_or(SweepError::Operation("page exceeds the atomic tag pair bound"))?;
        snapshots.try_reserve(1)?;
        snapshots.push(snapshot);
        Ok((snapshots, count))
    }).map(|(snapshots, _)| snapshots)
}
