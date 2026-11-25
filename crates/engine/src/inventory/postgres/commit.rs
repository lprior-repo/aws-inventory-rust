mod arrays;
mod facet_write;
pub(super) mod facets;
mod grants;
pub(super) mod keys;
pub(super) mod kind_rows;
mod queue;
pub(super) mod references;
pub(super) mod resource_write;
pub(super) mod resources;
mod scope_members;
mod tags;
mod validation;

use super::config::DirectCopy;
use super::{PgStore, completion, outbox, references as reconciliation};
use crate::inventory::relation_permissions::{SourceAuthority, page_authority};
use crate::inventory::{ChangeStats, CommitResult, PageCommit, StoreError, WorkLease};
use facets::Context;
use grants::Grant;
use inventory_core::inventory::{Observation, ResourceKey};
use resources::State;
use sqlx::{Acquire, Postgres, Transaction};
use std::collections::{BTreeMap, BTreeSet};

impl PgStore {
    pub async fn commit_page(&self, page: PageCommit) -> Result<CommitResult, StoreError> {
        Ok(self.commit(page, None).await?.0)
    }

    pub async fn commit_page_keeping(
        &self,
        page: PageCommit,
        lease_ms: i64,
    ) -> Result<(CommitResult, Option<WorkLease>), StoreError> {
        let lease_ms = crate::inventory::work::keep_interval(lease_ms)?;
        self.commit(page, Some(lease_ms)).await
    }

    async fn commit(
        &self,
        page: PageCommit,
        keep_ms: Option<i64>,
    ) -> Result<(CommitResult, Option<WorkLease>), StoreError> {
        validation::page(&self.registry, &page)?;
        let mut tx = self.pool.begin().await?;
        validation::lease(
            &mut tx,
            &page,
            self.registry.permissions.catalog().is_some(),
        )
        .await?;
        let mut stats = persist(self, &mut tx, &page).await?;
        queue::children(&mut tx, &page).await?;
        let deadline = keep_ms
            .filter(|_| !page.done)
            .map(|lease_ms| crate::inventory::work::kept_until(&page, lease_ms));
        let bound = deadline
            .as_ref()
            .and_then(|until| until.as_ref().ok().copied());
        let repeated_token = queue::advance(&mut tx, &page, bound).await?;
        let keep_until = match deadline {
            Some(until) if !repeated_token => Some(until?),
            _ => None,
        };
        if page.done || repeated_token {
            let completed =
                completion::drain(self, &mut tx, page.lease.sweep(), page.observed_at_ms).await?;
            completion::add(&mut stats, completed)?;
        }
        tx.commit().await?;
        let result = CommitResult {
            stats,
            next_generation: (!page.done && !repeated_token).then_some(page.lease.generation()),
            repeated_token,
        };
        let kept = keep_until.map(|until| page.lease.continued(page.next_token, until));
        Ok((result, kept))
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
    write(
        tx,
        &context,
        &states,
        &grants,
        store.config.copy_min_rows(),
        store.config.direct_copy(),
    )
    .await?;
    let tags = tags::persist(tx, page).await?;
    scope_members::persist(tx, page).await?;
    outbox::enqueue(tx, &projection(&states, tags)).await?;
    let changed: Vec<_> = states
        .values()
        .filter(|state| state.facet_changed)
        .map(|state| state.key)
        .collect();
    let mut stats = stats(&states)?;
    stats.edges_changed = reconciliation::reconcile(store, tx, &changed).await?;
    Ok(stats)
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
    copy_min_rows: Option<std::num::NonZeroUsize>,
    direct_copy: DirectCopy,
) -> Result<(), StoreError> {
    let direct = if direct_copy == DirectCopy::Enabled
        && copy_min_rows.is_some_and(|min| changed_rows(states) >= min.get())
        && states.values().all(|state| state.old_hash.is_none())
    {
        try_direct(tx, context, states).await?
    } else {
        false
    };
    if !direct {
        resource_write::persist(
            tx,
            states,
            context.page,
            &context.identity.source,
            copy_min_rows,
        )
        .await?;
        kind_rows::persist(tx, context.registry, states, copy_min_rows).await?;
    }
    facet_write::persist(tx, states, context).await?;
    grants::persist(tx, grants, &context.identity.source).await?;
    references::persist(tx, context.registry, states).await
}

fn changed_rows(states: &BTreeMap<&ResourceKey, State<'_>>) -> usize {
    states
        .values()
        .filter_map(|state| state.prepared.as_ref())
        .filter(|row| row.content_changed)
        .count()
}

async fn try_direct(
    tx: &mut Transaction<'_, Postgres>,
    context: &Context<'_>,
    states: &BTreeMap<&ResourceKey, State<'_>>,
) -> Result<bool, StoreError> {
    let mut savepoint = tx.begin().await?;
    let direct = async {
        let changed: Vec<_> = states
            .values()
            .filter_map(|state| state.prepared.as_ref())
            .filter(|row| row.content_changed)
            .collect();
        resource_write::direct(
            &mut savepoint,
            &changed,
            context.page,
            &context.identity.source,
        )
        .await?;
        kind_rows::direct(&mut savepoint, context.registry, states).await
    }
    .await;
    match direct {
        Ok(()) => {
            savepoint.commit().await?;
            Ok(true)
        }
        Err(error) => {
            let unique = unique_violation(&error);
            savepoint.rollback().await?;
            if unique { Ok(false) } else { Err(error) }
        }
    }
}

fn unique_violation(error: &StoreError) -> bool {
    matches!(
        error,
        StoreError::Sqlite(sqlx::Error::Database(database))
            if database.code().as_deref() == Some("23505")
    )
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
