//! Bounded stored-record streams and export visitors.
use super::*;

impl Store {
    /// Preferred resources in ID order, capped at 100,000 rows and 64 MiB of stored JSON.
    /// # Errors
    /// Fails for invalid/missing scans, SQL/decoding/validation errors or exceeded bounds.
    pub async fn resources(&self, scan_id: &str) -> Result<Vec<Resource>, StoreError> {
        collect_export(self.resource_stream(scan_id), MAX_EXPORT_RESOURCES).await
    }

    /// Preferred edges in endpoint/relation order, capped at 250,000 rows and 64 MiB.
    /// # Errors
    /// Fails for invalid/missing scans, SQL/decoding/validation errors or exceeded bounds.
    pub async fn edges(&self, scan_id: &str) -> Result<Vec<Edge>, StoreError> {
        collect_export(self.edge_stream(scan_id), MAX_EXPORT_EDGES).await
    }

    /// Deterministically ordered, one buffered row at a time; no complete snapshot allocation.
    /// At most 10,000,000 records; an existing empty scan yields no records.
    /// # Errors
    /// Yields invalid/missing-scan, SQL, decode, validation or record-count errors.
    pub fn resource_stream<'a>(&'a self, scan_id: &'a str) -> ResourceStream<'a> {
        self.json_stream(
            scan_id,
            "SELECT resource_json FROM scan_resources WHERE scan_id=?1 ORDER BY id",
        )
    }

    /// Preferred edges in `(from_id, to_id, relation)` order, one buffered row at a time.
    /// An existing empty scan yields no records; the stream caps at 10,000,000.
    /// # Errors
    /// Yields invalid/missing-scan, SQL, decode, validation or record-count errors.
    pub fn edge_stream<'a>(&'a self, scan_id: &'a str) -> EdgeStream<'a> {
        self.json_stream(
            scan_id,
            "SELECT edge_json FROM scan_edges WHERE scan_id=?1 ORDER BY from_id,to_id,relation",
        )
    }

    /// Invoke a synchronous visitor in resource-ID order without collecting the scan.
    /// # Errors
    /// Propagates the first resource-stream error or visitor failure and stops visiting.
    pub async fn visit_resources(
        &self,
        scan_id: &str,
        mut visitor: impl FnMut(Resource) -> Result<(), StoreError>,
    ) -> Result<(), StoreError> {
        self.resource_stream(scan_id)
            .try_for_each(|item| future::ready(visitor(item)))
            .await
    }

    pub(super) fn json_stream<'a, T>(
        &'a self,
        scan_id: &'a str,
        sql: &'static str,
    ) -> Pin<Box<dyn Stream<Item = Result<T, StoreError>> + Send + 'a>>
    where
        T: serde::de::DeserializeOwned + Send + 'a,
    {
        let records = stream::once(async move {
            self.scan_status(scan_id).await?;
            Ok::<_, StoreError>(())
        })
        .map_ok(move |_| {
            sqlx::query(sql)
                .bind(scan_id)
                .fetch(&self.reader)
                .take(MAX_STREAM_RECORDS.saturating_add(1))
                .enumerate()
                .then(|(index, row)| async move {
                    if index >= MAX_STREAM_RECORDS {
                        return Err(StoreError::Limit("stream record count"));
                    }
                    let row = row?;
                    tokio::task::coop::consume_budget().await;
                    decode_record(row)
                })
        })
        .try_flatten();
        Box::pin(records)
    }
}
