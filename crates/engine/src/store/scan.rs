//! Public scan lifecycle and page admission.
use super::*;

impl Store {
    /// Begin a running scan at nonnegative caller time; exact running replay is idempotent.
    /// # Errors
    /// Rejects invalid identities/time, a conflicting start, finalized replay or SQL failures.
    #[tracing::instrument(skip(self))]
    pub async fn begin_scan(&self, scan_id: &str, started_at: i64) -> Result<(), StoreError> {
        validate_identifier(scan_id, "scan id")?;
        if started_at < 0 {
            return Err(StoreError::InvalidInput("negative scan start"));
        }
        let mut tx = self.writer.begin_with("BEGIN IMMEDIATE").await?;
        sqlx::query(
            "INSERT INTO scans(scan_id,started_at,status) VALUES (?1,?2,'running')
                     ON CONFLICT(scan_id) DO NOTHING",
        )
        .bind(scan_id)
        .bind(started_at)
        .execute(&mut *tx)
        .await?;
        let state = scan_state(&mut tx, scan_id).await?;
        if state.started_at() != started_at {
            return Err(StoreError::Conflict(
                "scan id already has a different start time",
            ));
        }
        if !matches!(state, PersistedScan::Running { .. }) {
            return Err(StoreError::FinishedScan(scan_id.to_owned()));
        }
        tx.commit().await?;
        Ok(())
    }

    /// Validate and atomically commit a one-based collection page and its checkpoint.
    /// Exact fingerprint/content replay is idempotent; full-scope identity and page
    /// ordering are enforced before normalized resources and edges are updated.
    /// # Errors
    /// Rejects invalid/oversized pages, closed preparation admission, preparation
    /// task/allocation failures, missing/finalized scans, conflicting replay/order or SQL errors.
    #[tracing::instrument(skip(self, page))]
    pub async fn apply_page(&self, scan_id: &str, page: CollectionPage) -> Result<(), StoreError> {
        validate_identifier(scan_id, "scan id")?;
        let (page, _page_permit) = self.prepare(page).await?;
        let tx = self.writer.begin_with("BEGIN IMMEDIATE").await?;
        persist_page(tx, scan_id, page).await
    }

    #[tracing::instrument(skip(self, page))]
    pub(super) async fn prepare(
        &self,
        page: CollectionPage,
    ) -> Result<(PreparedPage, OwnedSemaphorePermit), StoreError> {
        let permit = Arc::clone(&self.preparation_permits)
            .acquire_owned()
            .await
            .map_err(|_| StoreError::Closed)?;
        let preparation_span = tracing::Span::current();
        let task = self.preparations.spawn_blocking(move || {
            let _entered = preparation_span.enter();
            let prepared = prepare_page(page)?;
            Ok((prepared, permit))
        });
        match task.await {
            Ok(result) => result,
            Err(error) if error.is_panic() => Err(StoreError::PreparationPanicked),
            Err(_) => Err(StoreError::PreparationCancelled),
        }
    }

    /// Persist a diagnostic transactionally; mark its scope failed unless the category is `unsupported_type`.
    /// # Errors
    /// Rejects invalid/oversized diagnostics, missing/finalized scans or SQL failures.
    #[tracing::instrument(skip(self, failure))]
    pub async fn record_failure(
        &self,
        scan_id: &str,
        failure: &CollectionFailure,
    ) -> Result<(), StoreError> {
        validate_failure(scan_id, failure)?;
        let scope_key = failure.scope.key()?;
        let mut tx = self.writer.begin_with("BEGIN IMMEDIATE").await?;
        require_running(&mut tx, scan_id).await?;
        insert_scope(&mut tx, scan_id, &scope_key, &failure.scope).await?;
        persist_failure(&mut tx, scan_id, &scope_key, failure).await?;
        tx.commit().await?;
        Ok(())
    }

    /// Finalize a scan at caller time no earlier than its start.
    /// Requested success becomes partial for zero, incomplete or failed scopes.
    /// Repeating the same effective terminal status/time is idempotent.
    /// # Errors
    /// Rejects running terminal requests, invalid identity/time, conflicting replay,
    /// missing scans or SQL/persisted-state decoding failures.
    #[tracing::instrument(skip(self))]
    pub async fn finish_scan(
        &self,
        scan_id: &str,
        status: ScanStatus,
        finished_at: i64,
    ) -> Result<ScanSummary, StoreError> {
        validate_identifier(scan_id, "scan id")?;
        let status = FinalScanStatus::try_from(status)?;
        let mut tx = self.writer.begin_with("BEGIN IMMEDIATE").await?;
        finalize_scan(&mut tx, scan_id, status, finished_at).await?;
        let summary = scan_summary(&mut tx, scan_id).await?;
        tx.commit().await?;
        Ok(summary)
    }

    /// Read normalized resource/edge totals and persisted scope completion counts.
    /// # Errors
    /// Rejects invalid/missing scans, invalid persisted state/counts or SQL failures.
    pub async fn summary(&self, scan_id: &str) -> Result<ScanSummary, StoreError> {
        let mut connection = self.reader.acquire().await?;
        reset_progress(&mut connection).await?;
        scan_summary(&mut connection, scan_id).await
    }

    /// Cloud Control drain accounting for one scan's persisted list scopes.
    /// Unsupported-type diagnostics remain visible without marking their scopes failed.
    /// An unknown valid scan identity returns zero counts.
    /// # Errors
    /// Rejects invalid scan identities, failed count conversion or SQL/row-decoding failures.
    pub async fn universal_outcomes(&self, scan_id: &str) -> Result<UniversalOutcomes, StoreError> {
        validate_identifier(scan_id, "scan id")?;
        let mut connection = self.reader.acquire().await?;
        reset_progress(&mut connection).await?;
        let row = sqlx::query(
            "SELECT
                (SELECT count(*) FROM scope_checkpoints
                    WHERE scan_id=?1 AND collector='cloud_control_list' AND source='cloud_control') AS attempted,
                (SELECT count(*) FROM scope_checkpoints AS scopes
                    WHERE scopes.scan_id=?1 AND scopes.collector='cloud_control_list'
                    AND scopes.source='cloud_control' AND scopes.complete=1 AND scopes.failed=0
                    AND NOT EXISTS (SELECT 1 FROM collection_failures AS failures
                        WHERE failures.scan_id=scopes.scan_id AND failures.scope_key=scopes.scope_key
                        AND failures.category='unsupported_type')) AS succeeded,
                (SELECT count(*) FROM collection_failures AS failures
                    JOIN scope_checkpoints AS scopes
                        ON scopes.scan_id=failures.scan_id
                        AND scopes.scope_key=failures.scope_key
                    WHERE failures.scan_id=?1 AND failures.category='unsupported_type'
                    AND scopes.collector='cloud_control_list' AND scopes.source='cloud_control')
                    AS unsupported,
                (SELECT count(*) FROM scope_checkpoints
                    WHERE scan_id=?1 AND collector='cloud_control_list' AND source='cloud_control'
                    AND failed=1) AS failed",
        )
        .bind(scan_id)
        .fetch_one(&mut *connection)
        .await?;
        Ok(UniversalOutcomes {
            attempted: row_u64(&row, "attempted")?,
            succeeded: row_u64(&row, "succeeded")?,
            unsupported: row_u64(&row, "unsupported")?,
            failed: row_u64(&row, "failed")?,
        })
    }

    /// Read the stored lifecycle state; this does not infer success from resource presence.
    /// # Errors
    /// Rejects invalid/missing scan identities, invalid persisted states or SQL failures.
    pub async fn scan_status(&self, scan_id: &str) -> Result<ScanStatus, StoreError> {
        let mut connection = self.reader.acquire().await?;
        reset_progress(&mut connection).await?;
        Ok(scan_state(&mut connection, scan_id).await?.status())
    }

    /// Load continuation evidence for the exact scan and full collection scope.
    /// `None` means no committed checkpoint, including for an otherwise unknown scan.
    /// # Errors
    /// Rejects invalid identities/scopes, SQL failures or malformed stored checkpoint values.
    pub async fn checkpoint(
        &self,
        scan_id: &str,
        scope: &CollectionScope,
    ) -> Result<Option<Checkpoint>, StoreError> {
        validate_identifier(scan_id, "scan id")?;
        validate_scope(scope)?;
        let mut connection = self.reader.acquire().await?;
        reset_progress(&mut connection).await?;
        read_checkpoint(&mut connection, scan_id, &scope.key()?).await
    }
}
