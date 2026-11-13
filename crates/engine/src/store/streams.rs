//! Bounded stored-record streams and export visitors.
use super::*;

impl Store {
    pub async fn resources(&self, scan_id: &str) -> Result<Vec<Resource>, StoreError> {
        collect_export(self.resource_stream(scan_id), MAX_EXPORT_RESOURCES).await
    }

    pub async fn edges(&self, scan_id: &str) -> Result<Vec<Edge>, StoreError> {
        collect_export(self.edge_stream(scan_id), MAX_EXPORT_EDGES).await
    }

    /// Deterministically ordered, one buffered row at a time; no complete snapshot allocation.
    pub fn resource_stream<'a>(&'a self, scan_id: &'a str) -> ResourceStream<'a> {
        self.json_stream(
            scan_id,
            "SELECT resource_json FROM scan_resources WHERE scan_id=?1 ORDER BY id",
        )
    }

    pub fn edge_stream<'a>(&'a self, scan_id: &'a str) -> EdgeStream<'a> {
        self.json_stream(
            scan_id,
            "SELECT edge_json FROM scan_edges WHERE scan_id=?1 ORDER BY from_id,to_id,relation",
        )
    }

    pub async fn visit_resources(
        &self,
        scan_id: &str,
        mut visitor: impl FnMut(Resource) -> Result<(), StoreError>,
    ) -> Result<(), StoreError> {
        self.resource_stream(scan_id)
            .try_for_each(|item| future::ready(visitor(item)))
            .await
    }

    pub async fn visit_edges(
        &self,
        scan_id: &str,
        mut visitor: impl FnMut(Edge) -> Result<(), StoreError>,
    ) -> Result<(), StoreError> {
        self.edge_stream(scan_id)
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
