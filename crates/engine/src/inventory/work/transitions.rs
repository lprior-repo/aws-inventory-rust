use super::*;
use inventory_core::inventory::AccountId;

pub(super) async fn validate_lease(
    tx: &mut Transaction<'_, Sqlite>,
    lease: &WorkLease,
    now_ms: i64,
) -> Result<(), StoreError> {
    let row = sqlx::query(
        "SELECT state,generation,authority_generation,lease_until FROM work_queue WHERE sweep_id=?1 AND scope_key=?2",
    )
    .bind(lease.sweep.as_str())
    .bind(encode_scope(&lease.scope)?)
    .fetch_optional(&mut **tx)
    .await?;
    let Some(row) = row else {
        return Err(StoreError::StaleLease);
    };
    let state: i64 = row.try_get("state")?;
    let generation: i64 = row.try_get("generation")?;
    let authority_generation: i64 = row.try_get("authority_generation")?;
    let deadline: i64 = row.try_get("lease_until")?;
    if (state != 1 && state != 2)
        || generation != lease.generation
        || authority_generation != generation
        || deadline <= now_ms
        || deadline != lease.lease_until_ms
    {
        return Err(StoreError::StaleLease);
    }
    Ok(())
}

pub(super) async fn record_continuation(
    tx: &mut Transaction<'_, Sqlite>,
    page: &PageCommit,
) -> Result<bool, StoreError> {
    let Some(digest) = page.continuation_digest else {
        return Ok(false);
    };
    let result = sqlx::query(
        "INSERT OR IGNORE INTO work_tokens(sweep_id,scope_key,digest) VALUES(?1,?2,?3)",
    )
    .bind(page.lease.sweep.as_str())
    .bind(encode_scope(&page.lease.scope)?)
    .bind(digest.as_slice())
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected() == 0)
}

pub(super) async fn persist_children(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    page: &PageCommit,
) -> Result<(), StoreError> {
    for child in &page.children {
        validate_work(store, child)?;
        let key = encode_scope(&child.scope)?;
        sqlx::query("INSERT OR IGNORE INTO work_queue(sweep_id,scope_key,token,bindings,state,generation,lease_until,context) VALUES(?1,?2,?3,?4,0,0,0,?5)")
            .bind(child.sweep.as_str()).bind(key).bind(&child.token).bind(&child.bindings).bind(&child.context).execute(&mut **tx).await?;
    }
    Ok(())
}

/// With `keep_until`, a scope that has another page stays leased until then.
pub(super) async fn persist_page_state(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    page: &PageCommit,
    repeated: bool,
    keep_until: Option<i64>,
) -> Result<ChangeStats, StoreError> {
    let scope = encode_scope(&page.lease.scope)?;
    if repeated {
        sqlx::query("INSERT INTO scope_outcomes(sweep_id,scope_key,outcome,observed_at) VALUES(?1,?2,'partial',?3) ON CONFLICT(sweep_id,scope_key) DO UPDATE SET outcome='partial',observed_at=excluded.observed_at,detail=NULL")
            .bind(page.lease.sweep.as_str()).bind(&scope).bind(page.observed_at_ms).execute(&mut **tx).await?;
        sqlx::query("UPDATE work_queue SET state=2,token=NULL,lease_until=0 WHERE sweep_id=?1 AND scope_key=?2 AND generation=?3")
            .bind(page.lease.sweep.as_str()).bind(&scope).bind(page.lease.generation).execute(&mut **tx).await?;
        return super::super::completion::drain_sweep(
            store,
            tx,
            page.lease.sweep(),
            page.observed_at_ms,
        )
        .await;
    }
    if page.done {
        return finalize_page(store, tx, page, &scope).await;
    }
    let (state, until) = keep_until.map_or((0_i64, 0_i64), |until| (1, until));
    sqlx::query("UPDATE work_queue SET state=?5,token=?4,lease_until=?6 WHERE sweep_id=?1 AND scope_key=?2 AND generation=?3")
        .bind(page.lease.sweep.as_str()).bind(&scope).bind(page.lease.generation).bind(&page.next_token).bind(state).bind(until).execute(&mut **tx).await?;
    Ok(ChangeStats::default())
}

pub(super) async fn finalize_page(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    page: &PageCommit,
    scope: &[u8],
) -> Result<ChangeStats, StoreError> {
    sqlx::query("INSERT INTO scope_outcomes(sweep_id,scope_key,outcome,observed_at) VALUES(?1,?2,'complete',?3) ON CONFLICT(sweep_id,scope_key) DO UPDATE SET outcome='complete',observed_at=excluded.observed_at,detail=NULL")
        .bind(page.lease.sweep.as_str()).bind(scope).bind(page.observed_at_ms).execute(&mut **tx).await?;
    sqlx::query("UPDATE work_queue SET state=2,token=NULL,lease_until=0 WHERE sweep_id=?1 AND scope_key=?2 AND generation=?3")
        .bind(page.lease.sweep.as_str()).bind(scope).bind(page.lease.generation).execute(&mut **tx).await?;
    super::super::completion::drain_sweep(store, tx, page.lease.sweep(), page.observed_at_ms).await
}

pub(in super::super) async fn resume_sweep(
    store: &BinaryStore,
    sweep: &SweepId,
    expected_account: &AccountId,
) -> Result<(), StoreError> {
    let mut tx = store.writer.begin().await?;

    // Ensure the sweeps table exists (may not exist in legacy databases).
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sweeps(sweep_id TEXT NOT NULL PRIMARY KEY,account TEXT,started_at INTEGER NOT NULL DEFAULT 0)",
    )
    .execute(&mut *tx)
    .await?;

    // Verify this sweep is active
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM work_queue WHERE sweep_id=?1 AND state IN(0,1)")
            .bind(sweep.as_str())
            .fetch_one(&mut *tx)
            .await?;
    if count == 0 {
        tx.commit().await?;
        return Ok(());
    }

    // Check for other active sweeps
    let other: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT sweep_id) FROM work_queue WHERE sweep_id != ?1 AND state IN(0,1)",
    )
    .bind(sweep.as_str())
    .fetch_one(&mut *tx)
    .await?;
    if other != 0 {
        return Err(StoreError::MultipleActiveSweeps);
    }

    // Verify manifest account matches expected
    let manifest: Option<String> =
        sqlx::query_scalar("SELECT account FROM sweeps WHERE sweep_id=?1")
            .bind(sweep.as_str())
            .fetch_optional(&mut *tx)
            .await?;

    let manifest_account = manifest.as_ref().map(|s| AccountId::parse(s)).transpose()?;
    if manifest_account != Some(expected_account.to_owned()) {
        return Err(StoreError::AccountMismatch(
            expected_account.to_owned(),
            manifest_account,
        ));
    }

    // Check for generation overflow
    let exhausted: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM work_queue WHERE sweep_id=?1 AND state=1 AND generation=9223372036854775807").bind(sweep.as_str()).fetch_one(&mut *tx).await?;
    if exhausted != 0 {
        return Err(StoreError::LeaseGenerationOverflow);
    }

    // Check for live leases
    let now_ms = std::time::UNIX_EPOCH
        .elapsed()
        .map_err(|_| StoreError::Invalid("system clock before epoch"))?
        .as_millis();
    let now_ms =
        i64::try_from(now_ms).map_err(|_| StoreError::Invalid("timestamp exceeds i64 range"))?;
    let live: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM work_queue WHERE sweep_id=?1 AND state=1 AND lease_until>?2",
    )
    .bind(sweep.as_str())
    .bind(now_ms)
    .fetch_one(&mut *tx)
    .await?;
    if live != 0 {
        return Err(StoreError::LiveLease);
    }

    sqlx::query("UPDATE work_queue SET state=0,generation=generation+1,lease_until=0 WHERE sweep_id=?1 AND state=1").bind(sweep.as_str()).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

pub(in crate::inventory) fn encode_scope(scope: &ScopeKey) -> Result<Vec<u8>, StoreError> {
    encode_scope_context(scope, scope.context())
}

pub(in super::super) fn ownership_scope_bytes(
    scope: &ScopeKey,
    bindings: &[u8],
) -> Result<Vec<u8>, StoreError> {
    use sha2::Digest;
    let context = if scope.context().is_some() || scope.parent().is_some() {
        let mut hash = sha2::Sha256::new();
        hash.update(b"aws-inventory/semantic-authority/v1\0");
        hash.update(bindings);
        Some(inventory_core::record::ContentHash::from_bytes(
            hash.finalize().into(),
        ))
    } else {
        None
    };
    encode_scope_context(scope, context)
}

fn encode_scope_context(
    scope: &ScopeKey,
    context: Option<inventory_core::record::ContentHash>,
) -> Result<Vec<u8>, StoreError> {
    let mut bytes = Vec::new();
    put_text(&mut bytes, scope.account().as_str())?;
    put_text(&mut bytes, scope.region().as_str())?;
    put_text(&mut bytes, scope.service().as_str())?;
    put_text(&mut bytes, scope.operation().as_str())?;
    match scope.parent() {
        Some(parent) => {
            bytes.push(1);
            put_resource(&mut bytes, parent)?;
        }
        None => bytes.push(0),
    }
    if scope.branch().is_some() || context.is_some() {
        bytes.push(1);
        put_text(&mut bytes, scope.branch().map_or("", |branch| branch))?;
        match context {
            Some(context) => {
                bytes.push(1);
                bytes.extend_from_slice(context.as_bytes());
            }
            None => bytes.push(0),
        }
    }
    if bytes.len() > MAX_SCOPE_BYTES {
        return Err(StoreError::Invalid("scope key exceeds bounds"));
    }
    Ok(bytes)
}

pub(in crate::inventory) fn decode_scope(bytes: &[u8]) -> Result<ScopeKey, StoreError> {
    let mut cursor = Cursor::new(bytes);
    let account = inventory_core::inventory::AccountId::parse(cursor.text()?)?;
    let region = inventory_core::inventory::Region::parse(cursor.text()?)?;
    let service = inventory_core::inventory::ServiceId::parse(cursor.text()?)?;
    let operation = inventory_core::inventory::OperationId::parse(cursor.text()?)?;
    let parent = match cursor.byte()? {
        0 => None,
        1 => Some(decode_resource_from(&mut cursor)?),
        _ => return Err(StoreError::Invalid("scope parent marker")),
    };
    let mut scope = ScopeKey::new(account, region, service, operation, parent)?;
    if !cursor.finished() {
        if cursor.byte()? != 1 {
            return Err(StoreError::Invalid("scope extension version"));
        }
        let branch = cursor.text()?;
        if !branch.is_empty() {
            scope = scope.with_branch(branch.to_owned())?;
        }
        match cursor.byte()? {
            0 => {}
            1 => {
                let end = cursor
                    .offset
                    .checked_add(32)
                    .ok_or(StoreError::Invalid("scope context offset overflow"))?;
                let digest = cursor
                    .bytes
                    .get(cursor.offset..end)
                    .ok_or(StoreError::Invalid("truncated scope context"))?
                    .try_into()
                    .map_err(|_| StoreError::Invalid("invalid scope context digest"))?;
                cursor.offset = end;
                scope = scope.with_context(inventory_core::record::ContentHash::from_bytes(digest));
            }
            _ => return Err(StoreError::Invalid("scope context marker")),
        }
        if scope.branch().is_none() && scope.context().is_none() {
            return Err(StoreError::Invalid("empty scope extension"));
        }
    }
    if !cursor.finished() {
        return Err(StoreError::Invalid("trailing scope bytes"));
    }
    Ok(scope)
}

pub(in super::super) fn encode_resource(key: &ResourceKey) -> Result<Vec<u8>, StoreError> {
    let mut out = Vec::new();
    put_resource(&mut out, key)?;
    Ok(out)
}
pub(super) fn put_resource(out: &mut Vec<u8>, key: &ResourceKey) -> Result<(), StoreError> {
    put_text(out, key.account.as_str())?;
    put_text(out, key.region.as_str())?;
    put_text(out, key.kind.as_str())?;
    put_text(out, key.id.as_str())?;
    Ok(())
}
pub(in super::super) fn decode_resource(bytes: &[u8]) -> Result<ResourceKey, StoreError> {
    let mut cursor = Cursor::new(bytes);
    let key = decode_resource_from(&mut cursor)?;
    if !cursor.finished() {
        return Err(StoreError::Invalid("trailing resource bytes"));
    }
    Ok(key)
}
pub(super) fn decode_resource_from(cursor: &mut Cursor<'_>) -> Result<ResourceKey, StoreError> {
    Ok(ResourceKey {
        account: inventory_core::inventory::AccountId::parse(cursor.text()?)?,
        region: inventory_core::inventory::Region::parse(cursor.text()?)?,
        kind: inventory_core::inventory::Kind::parse(cursor.text()?)?,
        id: inventory_core::inventory::ResourceId::parse(cursor.text()?)?,
    })
}
pub(in crate::inventory) fn put_text(out: &mut Vec<u8>, text: &str) -> Result<(), StoreError> {
    let size =
        u32::try_from(text.len()).map_err(|_| StoreError::Invalid("binary key field too large"))?;
    out.extend_from_slice(&size.to_be_bytes());
    out.extend_from_slice(text.as_bytes());
    Ok(())
}

pub(in crate::inventory) struct Cursor<'a> {
    pub(in crate::inventory) bytes: &'a [u8],
    pub(in crate::inventory) offset: usize,
}
impl<'a> Cursor<'a> {
    pub(in crate::inventory) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
    pub(in crate::inventory) fn byte(&mut self) -> Result<u8, StoreError> {
        let byte = self
            .bytes
            .get(self.offset)
            .copied()
            .ok_or(StoreError::Invalid("truncated binary key"))?;
        self.offset = self
            .offset
            .checked_add(1)
            .ok_or(StoreError::Invalid("binary offset overflow"))?;
        Ok(byte)
    }
    pub(in crate::inventory) fn text(&mut self) -> Result<&'a str, StoreError> {
        let length = u32::from_be_bytes([self.byte()?, self.byte()?, self.byte()?, self.byte()?]);
        let size =
            usize::try_from(length).map_err(|_| StoreError::Invalid("binary length overflow"))?;
        let end = self
            .offset
            .checked_add(size)
            .ok_or(StoreError::Invalid("binary offset overflow"))?;
        let value = std::str::from_utf8(
            self.bytes
                .get(self.offset..end)
                .ok_or(StoreError::Invalid("truncated binary key"))?,
        )
        .map_err(|_| StoreError::Invalid("invalid UTF-8 in binary key"))?;
        self.offset = end;
        Ok(value)
    }
    pub(in crate::inventory) fn finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

pub(in super::super) fn resource_bytes(key: &ResourceKey) -> Result<Vec<u8>, StoreError> {
    encode_resource(key)
}

pub(in super::super) async fn create_sweep_manifest(
    store: &BinaryStore,
    sweep: &SweepId,
    account: &str,
) -> Result<(), StoreError> {
    let now_ms = std::time::UNIX_EPOCH
        .elapsed()
        .map_err(|_| StoreError::Invalid("system clock before epoch"))?
        .as_millis();
    let now_ms =
        i64::try_from(now_ms).map_err(|_| StoreError::Invalid("timestamp exceeds i64 range"))?;
    let mut tx = store.writer.begin().await?;
    sqlx::query(
        "INSERT INTO sweeps(sweep_id,account,started_at) VALUES(?1,?2,?3) ON CONFLICT(sweep_id) DO UPDATE SET sweep_id=sweep_id",
    )
    .bind(sweep.as_str())
    .bind(account)
    .bind(now_ms)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub(in super::super) async fn sweep_manifest_account(
    store: &BinaryStore,
    sweep: &SweepId,
) -> Result<Option<String>, StoreError> {
    let row = sqlx::query_scalar("SELECT account FROM sweeps WHERE sweep_id=?1")
        .bind(sweep.as_str())
        .fetch_optional(&store.reader)
        .await?;
    Ok(row)
}

pub(in super::super) async fn discover_active_sweep(
    store: &BinaryStore,
) -> Result<Option<SweepId>, StoreError> {
    let row = sqlx::query_scalar(
        "SELECT DISTINCT sweep_id FROM work_queue WHERE state IN(0,1) ORDER BY sweep_id LIMIT 1",
    )
    .fetch_optional(&store.reader)
    .await?;
    if let Some(sweep_str) = row {
        return SweepId::new(sweep_str).map(Some);
    }
    Ok(None)
}
