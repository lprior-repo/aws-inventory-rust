//! Public scan lifecycle and page admission.
use super::*;

impl Store {
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

    pub async fn summary(&self, scan_id: &str) -> Result<ScanSummary, StoreError> {
        let mut connection = self.reader.acquire().await?;
        reset_progress(&mut connection).await?;
        scan_summary(&mut connection, scan_id).await
    }

    pub async fn scan_status(&self, scan_id: &str) -> Result<ScanStatus, StoreError> {
        let mut connection = self.reader.acquire().await?;
        reset_progress(&mut connection).await?;
        Ok(scan_state(&mut connection, scan_id).await?.status())
    }

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
