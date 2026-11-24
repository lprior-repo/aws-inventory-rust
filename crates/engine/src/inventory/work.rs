mod claiming;
pub(super) use claiming::claim;
mod owned_lease;
#[path = "work/transitions.rs"]
pub(in crate::inventory) mod transitions;
pub(super) use super::validation::{validate_active_sweep, validate_page, validate_work};
use super::{BinaryStore, MAX_BINDING_BYTES, MAX_TOKEN_BYTES, PageCommit, StoreError};
use inventory_core::inventory::{ResourceKey, ScopeKey, ScopeOutcome};
use sqlx::{Row, Sqlite, Transaction};
use std::sync::Arc;
use transitions::*;
pub(super) use transitions::{
    create_sweep_manifest, decode_resource, discover_active_sweep, encode_resource,
    ownership_scope_bytes, resource_bytes, resume_sweep, sweep_manifest_account,
};
const MAX_SWEEP_ID: usize = 128;
const MAX_SCOPE_BYTES: usize = 16_384;

pub fn persisted_scope(bytes: &[u8]) -> Result<ScopeKey, StoreError> {
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
    pub(in crate::inventory) token: Option<Vec<u8>>,
    pub(in crate::inventory) bindings: Vec<u8>,
    pub(in crate::inventory) context: Vec<u8>,
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
        // Legacy callers seed empty bindings as Vec::new() for plans with no
        // input; normalize to a valid empty Record (0xA0) so downstream
        // MapRef::from_bytes_bounded does not fail with Missing at offset 0.
        let bindings = if bindings.is_empty() {
            use inventory_core::record::{Record as CoreRecord, Value as CoreValue};
            let empty = CoreRecord::encode(&CoreValue::Map(std::collections::BTreeMap::new()))?;
            empty.into_bytes()
        } else {
            bindings
        };
        Ok(Self {
            sweep,
            scope,
            token,
            bindings,
            context: Vec::new(),
        })
    }

    pub fn with_context(mut self, context: Vec<u8>) -> Result<Self, StoreError> {
        if context.len() > 1_048_576 {
            return Err(StoreError::Invalid("work discovery context exceeds bounds"));
        }
        self.context = context;
        Ok(self)
    }

    pub(super) fn validate_context(&self) -> Result<(), StoreError> {
        if self.context.len() > 1_048_576 {
            return Err(StoreError::Invalid("work discovery context exceeds bounds"));
        }
        let digest = if self.context.is_empty() {
            None
        } else {
            let context = inventory_core::inventory::DiscoveryContext::decode(&self.context)
                .map_err(|_| StoreError::Invalid("invalid work discovery context"))?;
            Some(
                context
                    .discriminator()
                    .map_err(|_| StoreError::Invalid("invalid work discovery context digest"))?,
            )
        };
        if self.scope.context() != digest {
            return Err(StoreError::Invalid(
                "work discovery context digest mismatch",
            ));
        }
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
    pub(crate) pending_work: Option<u64>,
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
    pub fn pending_work(&self) -> Option<u64> {
        self.pending_work
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
    // Compute last_complete from scope_outcomes for any scopes that have completed before.
    sqlx::query("UPDATE work_queue SET last_complete=(SELECT MAX(o.observed_at) FROM scope_outcomes o WHERE o.scope_key=work_queue.scope_key AND o.outcome='complete') WHERE sweep_id=?1 AND last_complete=-1 AND (SELECT COUNT(1) FROM scope_outcomes o WHERE o.scope_key=work_queue.scope_key AND o.outcome='complete')>0")
        .bind(first.sweep.as_str()).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
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
    stats.resources_changed = stats
        .resources_changed
        .checked_add(completed.resources_changed)
        .ok_or(StoreError::Invalid("resource change count overflow"))?;
    stats.edges_changed = stats
        .edges_changed
        .checked_add(completed.edges_changed)
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
    detail: Option<&str>,
) -> Result<ChangeStats, StoreError> {
    super::validation::validate_detail(detail)?;
    if matches!(outcome, ScopeOutcome::Complete) {
        store.registry.permissions.validate_completion(&lease)?;
    }
    let mut tx = store.writer.begin().await?;
    validate_lease(&mut tx, &lease, observed_at_ms).await?;
    let scope = encode_scope(&lease.scope)?;
    sqlx::query("INSERT INTO scope_outcomes(sweep_id,scope_key,outcome,observed_at,detail) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(sweep_id,scope_key) DO UPDATE SET outcome=excluded.outcome,observed_at=excluded.observed_at,detail=excluded.detail")
        .bind(lease.sweep.as_str()).bind(&scope).bind(outcome.as_str()).bind(observed_at_ms).bind(detail)
        .execute(&mut *tx).await?;
    if matches!(outcome, ScopeOutcome::Complete) {
        // Update last_complete for future claim ordering.
        sqlx::query("UPDATE work_queue SET state=2,lease_until=0,last_complete=?4 WHERE sweep_id=?1 AND scope_key=?2 AND generation=?3")
            .bind(lease.sweep.as_str()).bind(&scope).bind(lease.generation).bind(observed_at_ms).execute(&mut *tx).await?;
    } else {
        sqlx::query("UPDATE work_queue SET state=2,lease_until=0 WHERE sweep_id=?1 AND scope_key=?2 AND generation=?3")
            .bind(lease.sweep.as_str()).bind(&scope).bind(lease.generation).execute(&mut *tx).await?;
    }
    let stats =
        super::completion::drain_sweep(store, &mut tx, lease.sweep(), observed_at_ms).await?;
    tx.commit().await?;
    Ok(stats)
}
