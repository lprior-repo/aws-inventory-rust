use super::*;

pub(super) async fn validate_lease(
    tx: &mut Transaction<'_, Sqlite>,
    lease: &WorkLease,
    now_ms: i64,
) -> Result<(), StoreError> {
    let row = sqlx::query(
        "SELECT state,generation,lease_until FROM work_queue WHERE sweep_id=?1 AND scope_key=?2",
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
    let deadline: i64 = row.try_get("lease_until")?;
    if (state != 1 && state != 2)
        || generation != lease.generation
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
        sqlx::query("INSERT OR IGNORE INTO work_queue(sweep_id,scope_key,token,bindings,state,generation,lease_until) VALUES(?1,?2,?3,?4,0,0,0)")
            .bind(child.sweep.as_str()).bind(key).bind(&child.token).bind(&child.bindings).execute(&mut **tx).await?;
    }
    Ok(())
}

pub(super) async fn persist_page_state(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    page: &PageCommit,
    repeated: bool,
) -> Result<ChangeStats, StoreError> {
    let scope = encode_scope(&page.lease.scope)?;
    if repeated {
        sqlx::query("INSERT INTO scope_outcomes(sweep_id,scope_key,outcome,observed_at) VALUES(?1,?2,'partial',?3) ON CONFLICT(sweep_id,scope_key) DO UPDATE SET outcome='partial',observed_at=excluded.observed_at")
            .bind(page.lease.sweep.as_str()).bind(&scope).bind(page.observed_at_ms).execute(&mut **tx).await?;
        sqlx::query("UPDATE work_queue SET state=2,token=NULL,lease_until=0 WHERE sweep_id=?1 AND scope_key=?2 AND generation=?3")
            .bind(page.lease.sweep.as_str()).bind(&scope).bind(page.lease.generation).execute(&mut **tx).await?;
        return Ok(ChangeStats::default());
    }
    if page.done {
        return finalize_page(store, tx, page, &scope).await;
    }
    sqlx::query("UPDATE work_queue SET state=0,token=?4,lease_until=0 WHERE sweep_id=?1 AND scope_key=?2 AND generation=?3")
        .bind(page.lease.sweep.as_str()).bind(&scope).bind(page.lease.generation).bind(&page.next_token).execute(&mut **tx).await?;
    Ok(ChangeStats::default())
}

pub(super) async fn finalize_page(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    page: &PageCommit,
    scope: &[u8],
) -> Result<ChangeStats, StoreError> {
    sqlx::query("INSERT INTO scope_outcomes(sweep_id,scope_key,outcome,observed_at) VALUES(?1,?2,'complete',?3) ON CONFLICT(sweep_id,scope_key) DO UPDATE SET outcome='complete',observed_at=excluded.observed_at")
        .bind(page.lease.sweep.as_str()).bind(scope).bind(page.observed_at_ms).execute(&mut **tx).await?;
    let completed =
        super::super::completion::complete_scope(store, tx, &page.lease, page.observed_at_ms)
            .await?;
    sqlx::query("UPDATE work_queue SET state=2,token=NULL,lease_until=0 WHERE sweep_id=?1 AND scope_key=?2 AND generation=?3")
        .bind(page.lease.sweep.as_str()).bind(scope).bind(page.lease.generation).execute(&mut **tx).await?;
    Ok(completed)
}

pub(in super::super) async fn resume_sweep(
    store: &BinaryStore,
) -> Result<Option<SweepId>, StoreError> {
    let mut tx = store.writer.begin().await?;
    let rows = sqlx::query(
        "SELECT DISTINCT sweep_id FROM work_queue WHERE state IN(0,1) ORDER BY sweep_id LIMIT 2",
    )
    .fetch_all(&mut *tx)
    .await?;
    if rows.len() > 1 {
        return Err(StoreError::MultipleActiveSweeps);
    }
    let Some(row) = rows.into_iter().next() else {
        tx.commit().await?;
        return Ok(None);
    };
    let sweep = SweepId::new(row.try_get("sweep_id")?)?;
    let exhausted: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM work_queue WHERE sweep_id=?1 AND state=1 AND generation=9223372036854775807").bind(sweep.as_str()).fetch_one(&mut *tx).await?;
    if exhausted != 0 {
        return Err(StoreError::LeaseGenerationOverflow);
    }
    sqlx::query("UPDATE work_queue SET state=0,generation=generation+1,lease_until=0 WHERE sweep_id=?1 AND state=1").bind(sweep.as_str()).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Some(sweep))
}

pub(super) fn encode_scope(scope: &ScopeKey) -> Result<Vec<u8>, StoreError> {
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
    if bytes.len() > MAX_SCOPE_BYTES {
        return Err(StoreError::Invalid("scope key exceeds bounds"));
    }
    Ok(bytes)
}

pub(super) fn decode_scope(bytes: &[u8]) -> Result<ScopeKey, StoreError> {
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
    if !cursor.finished() {
        return Err(StoreError::Invalid("trailing scope bytes"));
    }
    Ok(ScopeKey::new(account, region, service, operation, parent)?)
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
pub(super) fn put_text(out: &mut Vec<u8>, text: &str) -> Result<(), StoreError> {
    let size =
        u32::try_from(text.len()).map_err(|_| StoreError::Invalid("binary key field too large"))?;
    out.extend_from_slice(&size.to_be_bytes());
    out.extend_from_slice(text.as_bytes());
    Ok(())
}

pub(super) struct Cursor<'a> {
    pub(super) bytes: &'a [u8],
    pub(super) offset: usize,
}
impl<'a> Cursor<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
    pub(super) fn byte(&mut self) -> Result<u8, StoreError> {
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
    pub(super) fn text(&mut self) -> Result<&'a str, StoreError> {
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
    pub(super) fn finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

pub(in super::super) fn scope_bytes(scope: &ScopeKey) -> Result<Vec<u8>, StoreError> {
    encode_scope(scope)
}
pub(in super::super) fn resource_bytes(key: &ResourceKey) -> Result<Vec<u8>, StoreError> {
    encode_resource(key)
}
