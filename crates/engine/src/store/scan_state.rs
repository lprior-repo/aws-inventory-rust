use super::*;

pub(super) async fn scan_state(
    connection: &mut SqliteConnection,
    scan_id: &str,
) -> Result<PersistedScan, StoreError> {
    validate_identifier(scan_id, "scan id")?;
    let (started_at, status, finished_at): (i64, String, Option<i64>) =
        sqlx::query_as("SELECT started_at,status,finished_at FROM scans WHERE scan_id=?1")
            .bind(scan_id)
            .fetch_optional(connection)
            .await?
            .ok_or_else(|| StoreError::MissingScan(scan_id.to_owned()))?;
    PersistedScan::parse(started_at, &status, finished_at)
}

pub(super) async fn require_running(
    connection: &mut SqliteConnection,
    scan_id: &str,
) -> Result<(), StoreError> {
    if !matches!(
        scan_state(connection, scan_id).await?,
        PersistedScan::Running { .. }
    ) {
        return Err(StoreError::FinishedScan(scan_id.to_owned()));
    }
    Ok(())
}

pub(super) async fn insert_scope(
    connection: &mut SqliteConnection,
    scan_id: &str,
    key: &str,
    scope: &CollectionScope,
) -> Result<(), StoreError> {
    sqlx::query("INSERT INTO scope_checkpoints(scan_id,scope_key,account_id,region,collector,parent_id,source)
        VALUES (?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(scan_id,scope_key) DO NOTHING")
        .bind(scan_id).bind(key).bind(&scope.account_id).bind(&scope.region).bind(&scope.collector)
        .bind(&scope.parent_id).bind(scope.source.as_str()).execute(connection).await?;
    Ok(())
}

pub(super) async fn read_checkpoint(
    connection: &mut SqliteConnection,
    scan_id: &str,
    key: &str,
) -> Result<Option<Checkpoint>, StoreError> {
    let row = sqlx::query("SELECT last_page,next_token,complete,failed FROM scope_checkpoints WHERE scan_id=?1 AND scope_key=?2")
        .bind(scan_id).bind(key).fetch_optional(connection).await?;
    row.as_ref()
        .map(|row| {
            Ok(Checkpoint {
                last_page: u32::try_from(row.try_get::<i64, _>("last_page")?)
                    .map_err(|_| StoreError::Conflict("invalid persisted page number"))?,
                next_token: row.try_get("next_token")?,
                complete: row.try_get("complete")?,
                failed: row.try_get("failed")?,
            })
        })
        .transpose()
}

pub(super) async fn page_already_committed(
    connection: &mut SqliteConnection,
    scan_id: &str,
    page: &PreparedPage,
) -> Result<bool, StoreError> {
    let previous: Option<Vec<u8>> = sqlx::query_scalar("SELECT fingerprint FROM collection_pages WHERE scan_id=?1 AND scope_key=?2 AND page_number=?3")
        .bind(scan_id).bind(&page.scope_key).bind(i64::from(page.page_number)).fetch_optional(connection).await?;
    match previous {
        Some(bytes) if bytes == page.fingerprint => Ok(true),
        Some(_) => Err(StoreError::Conflict(
            "page identity reused with different content",
        )),
        None => Ok(false),
    }
}

pub(super) async fn validate_page_sequence(
    connection: &mut SqliteConnection,
    scan_id: &str,
    page: &PreparedPage,
) -> Result<(), StoreError> {
    let previous = read_checkpoint(connection, scan_id, &page.scope_key)
        .await?
        .ok_or(StoreError::Conflict("scope checkpoint missing"))?;
    let expected = previous
        .last_page
        .checked_add(1)
        .ok_or(StoreError::Limit("page sequence overflow"))?;
    if previous.complete || page.page_number != expected {
        return Err(StoreError::Conflict(
            "page is out of order or scope is complete",
        ));
    }
    if let Some(token) = &page.next_token {
        let repeated: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM collection_pages WHERE scan_id=?1 AND scope_key=?2 AND next_token=?3)")
            .bind(scan_id).bind(&page.scope_key).bind(token).fetch_one(connection).await?;
        if repeated {
            return Err(StoreError::Conflict("continuation token cycle"));
        }
    }
    Ok(())
}

pub(super) fn validate_failure(
    scan_id: &str,
    failure: &CollectionFailure,
) -> Result<(), StoreError> {
    validate_identifier(scan_id, "scan id")?;
    validate_scope(&failure.scope)?;
    validate_identifier(&failure.category, "failure category")?;
    if failure.message.len() > MAX_FAILURE_BYTES {
        return Err(StoreError::Limit("failure message bytes"));
    }
    Ok(())
}

fn accounted_category(category: &str) -> bool {
    matches!(category, "unsupported_type")
}

pub(super) async fn persist_failure(
    connection: &mut SqliteConnection,
    scan_id: &str,
    key: &str,
    failure: &CollectionFailure,
) -> Result<(), StoreError> {
    sqlx::query("INSERT INTO collection_failures(scan_id,scope_key,category,message) VALUES (?1,?2,?3,?4) ON CONFLICT DO NOTHING")
        .bind(scan_id).bind(key).bind(&failure.category).bind(&failure.message).execute(&mut *connection).await?;
    if !accounted_category(&failure.category) {
        sqlx::query("UPDATE scope_checkpoints SET failed=1 WHERE scan_id=?1 AND scope_key=?2")
            .bind(scan_id)
            .bind(key)
            .execute(connection)
            .await?;
    }
    Ok(())
}

pub(super) async fn finalize_scan(
    connection: &mut SqliteConnection,
    scan_id: &str,
    status: FinalScanStatus,
    finished_at: i64,
) -> Result<(), StoreError> {
    let state = scan_state(connection, scan_id).await?;
    if finished_at < state.started_at() {
        return Err(StoreError::InvalidInput("scan finish precedes start"));
    }
    let effective = effective_status(connection, scan_id, status).await?;
    if let PersistedScan::Finalized {
        status,
        finished_at: previous_finish,
        ..
    } = state
    {
        if status != effective || previous_finish != finished_at {
            return Err(StoreError::Conflict(
                "scan was already finalized differently",
            ));
        }
        return Ok(());
    }
    sqlx::query("UPDATE scans SET status=?2,finished_at=?3 WHERE scan_id=?1")
        .bind(scan_id)
        .bind(effective.as_str())
        .bind(finished_at)
        .execute(connection)
        .await?;
    Ok(())
}

pub(super) async fn effective_status(
    connection: &mut SqliteConnection,
    scan_id: &str,
    requested: FinalScanStatus,
) -> Result<FinalScanStatus, StoreError> {
    if requested != FinalScanStatus::Complete {
        return Ok(requested);
    }
    let (total, incomplete): (i64, i64) = sqlx::query_as(
        "SELECT count(*),coalesce(sum(CASE WHEN complete=0 OR failed=1 THEN 1 ELSE 0 END),0)
        FROM scope_checkpoints WHERE scan_id=?1",
    )
    .bind(scan_id)
    .fetch_one(connection)
    .await?;
    if total == 0 || incomplete > 0 {
        Ok(FinalScanStatus::Partial)
    } else {
        Ok(FinalScanStatus::Complete)
    }
}

pub(super) async fn scan_summary(
    connection: &mut SqliteConnection,
    scan_id: &str,
) -> Result<ScanSummary, StoreError> {
    scan_state(connection, scan_id).await?;
    let row = sqlx::query("SELECT
        (SELECT count(*) FROM scan_resources WHERE scan_id=?1) AS resources,
        (SELECT count(*) FROM scan_edges WHERE scan_id=?1) AS edges,
        (SELECT count(*) FROM scope_checkpoints WHERE scan_id=?1 AND complete=1 AND failed=0) AS completed,
        (SELECT count(*) FROM scope_checkpoints WHERE scan_id=?1 AND failed=1) AS failed")
        .bind(scan_id).fetch_one(connection).await?;
    Ok(ScanSummary {
        scan_id: scan_id.to_owned(),
        resources: row_u64(&row, "resources")?,
        edges: row_u64(&row, "edges")?,
        completed_scopes: row_u64(&row, "completed")?,
        failed_scopes: row_u64(&row, "failed")?,
    })
}
