mod arrays;
mod facet_write;
mod facets;
mod grants;
mod keys;
mod kind_rows;
mod queue;
mod references;
mod resource_write;
mod resources;
mod tags;
mod validation;

use super::{PgStore, outbox};
use crate::inventory::relation_permissions::{SourceAuthority, page_authority};
use crate::inventory::{ChangeStats, CommitResult, PageCommit, StoreError};
use facets::Context;
use grants::Grant;
use inventory_core::inventory::{Observation, ResourceKey};
use resources::State;
use sqlx::{Postgres, Transaction};
use std::collections::{BTreeMap, BTreeSet};

impl PgStore {
    pub async fn commit_page(&self, page: PageCommit) -> Result<CommitResult, StoreError> {
        validation::page(&self.registry, &page)?;
        if page.done {
            return Err(StoreError::Invalid("completion not implemented"));
        }
        let mut tx = self.pool.begin().await?;
        validation::lease(&mut tx, &page).await?;
        let stats = persist(self, &mut tx, &page).await?;
        queue::children(&mut tx, &page).await?;
        let repeated_token = queue::advance(&mut tx, &page).await?;
        tx.commit().await?;
        Ok(CommitResult {
            stats,
            next_generation: (!repeated_token).then_some(page.lease.generation()),
            repeated_token,
        })
    }
}

async fn persist(
    store: &PgStore,
    tx: &mut Transaction<'_, Postgres>,
    page: &PageCommit,
) -> Result<ChangeStats, StoreError> {
    let direct = crate::inventory::commit::edges::references(&store.registry, page)?;
    let context = Context::new(&store.registry, page, &direct)?;
    let mut states = resources::states(page);
    resources::load(tx, &mut states, &context).await?;
    let grants = apply(&context, &mut states)?;
    for state in states.values_mut().filter(|state| state.facet_changed) {
        resources::prepare(&store.registry, state, page.observed_at_ms)?;
    }
    write(tx, &context, &states, &grants).await?;
    let tags = tags::persist(tx, page).await?;
    outbox::enqueue(tx, &projection(&states, tags)).await?;
    stats(&states)
}

fn apply<'a>(
    context: &Context<'a>,
    states: &mut BTreeMap<&'a ResourceKey, State<'a>>,
) -> Result<Vec<Grant<'a>>, StoreError> {
    let authority = page_authority(context.registry, &context.page.lease)?;
    let mut grants = Vec::new();
    for observation in &context.page.observations {
        let source = authority
            .as_ref()
            .map(|auth| auth.source(observation))
            .transpose()?;
        let state = states
            .get_mut(observation.key())
            .ok_or(StoreError::Invalid("resource key is absent"))?;
        observe(context, state, observation, source.as_ref(), &mut grants)?;
    }
    Ok(grants)
}

fn observe<'a>(
    context: &Context<'a>,
    state: &mut State<'a>,
    observation: &'a Observation,
    authority: Option<&SourceAuthority<'_, '_>>,
    grants: &mut Vec<Grant<'a>>,
) -> Result<(), StoreError> {
    if facets::replace(&mut state.facets, context, observation, authority)? {
        state.facet_changed = true;
        if let Some(authority) = authority {
            grants::collect(context, observation, authority, grants)?;
        }
    }
    Ok(())
}

async fn write(
    tx: &mut Transaction<'_, Postgres>,
    context: &Context<'_>,
    states: &BTreeMap<&ResourceKey, State<'_>>,
    grants: &[Grant<'_>],
) -> Result<(), StoreError> {
    resource_write::persist(tx, states, context.page, &context.identity.source).await?;
    kind_rows::persist(tx, context.registry, states).await?;
    facet_write::persist(tx, states, context).await?;
    grants::persist(tx, grants, &context.identity.source).await?;
    references::persist(tx, context.registry, states).await
}

fn projection<'a>(
    states: &BTreeMap<&'a ResourceKey, State<'a>>,
    tags: BTreeSet<&'a ResourceKey>,
) -> BTreeMap<&'a ResourceKey, bool> {
    let mut entries: BTreeMap<_, _> = tags.into_iter().map(|key| (key, false)).collect();
    for state in states.values().filter(|state| state.facet_changed) {
        entries.insert(state.key, false);
    }
    entries
}

fn stats(states: &BTreeMap<&ResourceKey, State<'_>>) -> Result<ChangeStats, StoreError> {
    states
        .values()
        .filter_map(|state| state.prepared.as_ref())
        .filter(|row| row.content_changed)
        .try_fold(ChangeStats::default(), |mut stats, row| {
            let counter = if row.new {
                &mut stats.resources_inserted
            } else {
                &mut stats.resources_changed
            };
            *counter = counter
                .checked_add(1)
                .ok_or(StoreError::Invalid("resource count overflow"))?;
            Ok(stats)
        })
}
