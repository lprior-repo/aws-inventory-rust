#[path = "work/transitions.rs"]
mod transitions;
use super::{BinaryStore, MAX_BINDING_BYTES, MAX_TOKEN_BYTES, PageCommit, StoreError};
use inventory_core::inventory::{ResourceKey, ScopeKey, ScopeOutcome};
use sqlx::{Row, Sqlite, Transaction};
use std::sync::Arc;
use transitions::*;
pub(super) use transitions::{
    decode_resource, encode_resource, resource_bytes, resume_sweep, ownership_scope_bytes,
};

pub(super) use super::validation::{validate_active_sweep, validate_page, validate_work};
const MAX_SWEEP_ID: usize = 128;
const MAX_SCOPE_BYTES: usize = 16_384;

pub(super) fn persisted_scope(bytes: &[u8]) -> Result<ScopeKey, StoreError> {
    decode_scope(bytes)
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SweepId(Arc<str>);

impl SweepId {
    pub fn new(value: String) -> Result<Self, StoreError> {
        if value.is_empty()
            || value.len() > MAX_SWEEP_ID
            || value.bytes().any(|byte| byte.is_ascii_control())
        {
            return Err(StoreError::Invalid("sweep identifier exceeds bounds"));
        }
        Ok(Self(Arc::from(value)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct WorkSpec {
    sweep: SweepId,
    scope: ScopeKey,
    token: Option<Vec<u8>>,
    bindings: Vec<u8>,
    context: Vec<u8>,
}

impl WorkSpec {
    pub fn new(
        sweep: SweepId,
        scope: ScopeKey,
        token: Option<Vec<u8>>,
        bindings: Vec<u8>,
    ) -> Result<Self, StoreError> {
        if token
            .as_ref()
            .is_some_and(|value| value.len() > MAX_TOKEN_BYTES)
            || bindings.len() > MAX_BINDING_BYTES
        {
            return Err(StoreError::Invalid("work token or bindings exceed bounds"));
        }
        Ok(Self {
            sweep,
            scope,
            token,
            bindings,
            context: Vec::new(),
        })
    }

    pub fn with_context(mut self, context: Vec<u8>) -> Result<Self, StoreError> {
        if context.len() > 1_048_576 { return Err(StoreError::Invalid("work discovery context exceeds bounds")); }
        self.context = context;
        Ok(self)
    }

    pub(super) fn validate_context(&self) -> Result<(), StoreError> {
        if self.context.len() > 1_048_576 { return Err(StoreError::Invalid("work discovery context exceeds bounds")); }
        let digest = if self.context.is_empty() { None } else {
            let context = inventory_core::inventory::DiscoveryContext::decode(&self.context)
                .map_err(|_| StoreError::Invalid("invalid work discovery context"))?;
            Some(context.discriminator().map_err(|_| StoreError::Invalid("invalid work discovery context digest"))?)
        };
        if self.scope.context() != digest { return Err(StoreError::Invalid("work discovery context digest mismatch")); }
        Ok(())
    }

    pub fn sweep(&self) -> &SweepId {
        &self.sweep
    }
    pub fn scope(&self) -> &ScopeKey {
        &self.scope
    }
}

#[derive(Clone, Debug)]
pub struct WorkLease {
    pub(crate) sweep: SweepId,
    pub(crate) scope: ScopeKey,
    pub(crate) generation: i64,
    pub(crate) token: Option<Vec<u8>>,
    pub(crate) bindings: Vec<u8>,
    pub(crate) context: Vec<u8>,
    pub(crate) lease_until_ms: i64,
}

impl WorkLease {
    pub fn sweep(&self) -> &SweepId {
        &self.sweep
    }
    pub fn scope(&self) -> &ScopeKey {
        &self.scope
    }
    pub fn generation(&self) -> i64 {
        self.generation
    }
    pub fn token(&self) -> Option<&[u8]> {
        self.token.as_deref()
    }
    pub fn bindings(&self) -> &[u8] {
        &self.bindings
    }
    pub fn context(&self) -> &[u8] {
        &self.context
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChangeStats {
    pub resources_inserted: u64,
    pub resources_changed: u64,
    pub resources_deleted: u64,
    pub edges_changed: u64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SweepSummary {
    pub complete_scopes: u64,
    pub partial_scopes: u64,
    pub inaccessible_scopes: u64,
    pub unsupported_scopes: u64,
    pub failed_scopes: u64,
    pub unobserved_scopes: u64,
    pub resources_observed: u64,
    pub work_pending: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitResult {
    pub stats: ChangeStats,
    pub next_generation: Option<i64>,
    pub repeated_token: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingProjection {
    pub key: Option<ResourceKey>,
    pub revision: i64,
    pub deleted: bool,
    pub full_rebuild: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionAck {
    pub key: Option<ResourceKey>,
    pub revision: i64,
}

impl ProjectionAck {
    pub fn new(key: ResourceKey, revision: i64) -> Result<Self, StoreError> {
        Self::checked(Some(key), revision)
    }
    pub fn full_rebuild(revision: i64) -> Result<Self, StoreError> {
        Self::checked(None, revision)
    }
    fn checked(key: Option<ResourceKey>, revision: i64) -> Result<Self, StoreError> {
        if revision < 1 {
            return Err(StoreError::Invalid("projection revision must be positive"));
        }
        Ok(Self { key, revision })
    }
}

pub(super) async fn seed(store: &BinaryStore, work: WorkSpec) -> Result<(), StoreError> {
    seed_works(store, std::slice::from_ref(&work)).await
}

pub(super) async fn seed_works(store: &BinaryStore, works: &[WorkSpec]) -> Result<(), StoreError> {
    let Some(first) = works.first() else {
        return Ok(());
    };
    for work in works {
        validate_work(store, work)?;
        if work.sweep != first.sweep {
            return Err(StoreError::Invalid("batch work items must share one sweep"));
        }
    }
    let mut tx = store.writer.begin().await?;
    validate_active_sweep(&mut tx, Some(&first.sweep)).await?;
    for work in works {
        let scope = encode_scope(&work.scope)?;
        sqlx::query("INSERT OR IGNORE INTO work_queue(sweep_id,scope_key,token,bindings,state,generation,lease_until,context) VALUES(?1,?2,?3,?4,0,0,0,?5)")
            .bind(work.sweep.as_str()).bind(scope).bind(&work.token).bind(&work.bindings).bind(&work.context).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(())
}

pub(super) async fn claim(
    store: &BinaryStore,
    sweep: &SweepId,
    now_ms: i64,
    lease_ms: i64,
) -> Result<Option<WorkLease>, StoreError> {
    if now_ms < 0 || lease_ms < 1 || now_ms.checked_add(lease_ms).is_none() {
        return Err(StoreError::Invalid("invalid lease interval"));
    }
    let mut tx = store.writer.begin().await?;
    validate_active_sweep(&mut tx, Some(sweep)).await?;
    let row = sqlx::query("SELECT scope_key,token,bindings,generation,authority_complete,context FROM work_queue WHERE sweep_id=?1 AND (state=0 OR (state=1 AND lease_until<=?2)) ORDER BY scope_key LIMIT 1")
        .bind(sweep.as_str()).bind(now_ms).fetch_optional(&mut *tx).await?;
    let Some(row) = row else {
        tx.commit().await?;
        return Ok(None);
    };
    let key: Vec<u8> = row.try_get("scope_key")?;
    let generation: i64 = row.try_get("generation")?;
    let next_generation = generation
        .checked_add(1)
        .ok_or(StoreError::Invalid("lease generation overflow"))?;
    let complete = row.try_get::<Option<&[u8]>, _>("token")?.is_none()
        || row.try_get::<bool, _>("authority_complete")?;
    let lease_until = now_ms
        .checked_add(lease_ms)
        .ok_or(StoreError::Invalid("lease interval overflow"))?;
    sqlx::query("UPDATE work_queue SET state=1,generation=?3,authority_generation=?3,lease_until=?4,authority_complete=?5 WHERE sweep_id=?1 AND scope_key=?2")
        .bind(sweep.as_str()).bind(&key).bind(next_generation).bind(lease_until).bind(complete).execute(&mut *tx).await?;
    let lease = WorkLease {
        sweep: sweep.clone(),
        scope: decode_scope(&key)?,
        generation: next_generation,
        token: row.try_get("token")?,
        bindings: row.try_get("bindings")?,
        context: row.try_get("context")?,
        lease_until_ms: lease_until,
    };
    tx.commit().await?;
    Ok(Some(lease))
}

pub(super) async fn commit_page(
    store: &BinaryStore,
    page: PageCommit,
) -> Result<CommitResult, StoreError> {
    validate_page(store, &page)?;
    let mut tx = store.writer.begin().await?;
    validate_lease(&mut tx, &page.lease, page.observed_at_ms).await?;
    let mut stats = super::commit::persist_page(store, &mut tx, &page).await?;
    persist_children(store, &mut tx, &page).await?;
    let repeated_token = record_continuation(&mut tx, &page).await?;
    let completed = persist_page_state(store, &mut tx, &page, repeated_token).await?;
    stats.resources_deleted = stats
        .resources_deleted
        .checked_add(completed.resources_deleted)
        .ok_or(StoreError::Invalid("deletion count overflow"))?;
    stats.resources_changed = stats.resources_changed.checked_add(completed.resources_changed)
        .ok_or(StoreError::Invalid("resource change count overflow"))?;
    stats.edges_changed = stats.edges_changed.checked_add(completed.edges_changed)
        .ok_or(StoreError::Invalid("edge count overflow"))?;
    tx.commit().await?;
    Ok(CommitResult {
        stats,
        next_generation: (!repeated_token).then_some(page.lease.generation),
        repeated_token,
    })
}

pub(super) async fn finish_scope(
    store: &BinaryStore,
    lease: WorkLease,
    outcome: ScopeOutcome,
    observed_at_ms: i64,
) -> Result<ChangeStats, StoreError> {
    if matches!(outcome, ScopeOutcome::Complete) {
        store.registry.permissions.validate_completion(&lease)?;
    }
    let mut tx = store.writer.begin().await?;
    validate_lease(&mut tx, &lease, observed_at_ms).await?;
    sqlx::query("INSERT INTO scope_outcomes(sweep_id,scope_key,outcome,observed_at) VALUES(?1,?2,?3,?4) ON CONFLICT(sweep_id,scope_key) DO UPDATE SET outcome=excluded.outcome,observed_at=excluded.observed_at")
        .bind(lease.sweep.as_str()).bind(encode_scope(&lease.scope)?).bind(outcome.as_str()).bind(observed_at_ms)
        .execute(&mut *tx).await?;
    sqlx::query("UPDATE work_queue SET state=2,lease_until=0 WHERE sweep_id=?1 AND scope_key=?2 AND generation=?3")
        .bind(lease.sweep.as_str()).bind(encode_scope(&lease.scope)?).bind(lease.generation).execute(&mut *tx).await?;
    let stats = super::completion::drain_sweep(store, &mut tx, lease.sweep(), observed_at_ms).await?;
    tx.commit().await?;
    Ok(stats)
}
