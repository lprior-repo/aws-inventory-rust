use super::StoreError;
use crate::inventory::BinaryStore;
use sqlx::Row;

/// The target identity for this inventory's projection.
/// Changing endpoint or graph name without a rebuild leaves
/// the graph stale relative to the outbox acknowledgements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionIdentity {
    pub endpoint: String,
    pub graph: String,
}

impl ProjectionIdentity {
    /// True if either endpoint or graph differ.
    pub fn differs(&self, endpoint: &str, graph: &str) -> bool {
        self.endpoint != endpoint || self.graph != graph
    }
}

pub(super) async fn read_identity(
    store: &BinaryStore,
) -> Result<Option<ProjectionIdentity>, StoreError> {
    let mut tx = store.reader.begin().await?;
    let row = sqlx::query("SELECT value FROM metadata WHERE key='projection_identity'")
        .fetch_optional(&mut *tx)
        .await?;
    tx.commit().await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let value: Vec<u8> = row.try_get(0)?;
    if value.is_empty() {
        return Ok(None);
    }
    // Stored as "<endpoint>\n<graph>"
    let text = std::str::from_utf8(&value)
        .map_err(|_| StoreError::Invalid("projection_identity is not UTF-8"))?;
    let mut parts = text.split('\n');
    let endpoint = parts
        .next()
        .ok_or(StoreError::Invalid("projection_identity missing endpoint"))?;
    let graph = parts
        .next()
        .ok_or(StoreError::Invalid("projection_identity missing graph"))?;
    if parts.next().is_some() {
        return Err(StoreError::Invalid(
            "projection_identity has too many fields",
        ));
    }
    Ok(Some(ProjectionIdentity {
        endpoint: endpoint.to_owned(),
        graph: graph.to_owned(),
    }))
}

pub(super) async fn write_identity(
    store: &BinaryStore,
    identity: &ProjectionIdentity,
) -> Result<(), StoreError> {
    let mut tx = store.writer.begin().await?;
    let value = format!("{}\n{}", identity.endpoint, identity.graph);
    sqlx::query("INSERT INTO metadata(key,value) VALUES('projection_identity',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value")
        .bind(value.as_bytes())
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub(super) async fn mark_rebuild_required(store: &BinaryStore) -> Result<(), StoreError> {
    let mut tx = store.writer.begin().await?;
    sqlx::query("UPDATE metadata SET value=X'01' WHERE key='projection_rebuild'")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}
