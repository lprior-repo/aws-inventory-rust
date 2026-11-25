use super::{BinaryStore, StoreError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionIdentity {
    pub endpoint: String,
    pub graph: String,
}

impl ProjectionIdentity {
    pub fn differs(&self, endpoint: &str, graph: &str) -> bool {
        self.endpoint != endpoint || self.graph != graph
    }

    pub(crate) fn encode(&self) -> Result<Vec<u8>, StoreError> {
        validate(&self.endpoint, &self.graph)?;
        Ok(format!("{}\n{}", self.endpoint, self.graph).into_bytes())
    }

    pub(crate) fn decode(value: &[u8]) -> Result<Self, StoreError> {
        if value.len() > 4225 {
            return Err(StoreError::Invalid("projection identity exceeds bounds"));
        }
        let text = std::str::from_utf8(value)
            .map_err(|_| StoreError::Invalid("projection identity is not UTF-8"))?;
        let (endpoint, graph) = text
            .split_once('\n')
            .ok_or(StoreError::Invalid("projection identity missing graph"))?;
        validate(endpoint, graph)?;
        Ok(Self {
            endpoint: endpoint.to_owned(),
            graph: graph.to_owned(),
        })
    }
}

fn validate(endpoint: &str, graph: &str) -> Result<(), StoreError> {
    if endpoint.len() > 4096 || graph.len() > 128 || endpoint.contains('\n') || graph.contains('\n')
    {
        return Err(StoreError::Invalid("projection identity exceeds bounds"));
    }
    Ok(())
}

pub(super) async fn read(store: &BinaryStore) -> Result<Option<ProjectionIdentity>, StoreError> {
    let _turn = store.read_turn().await;
    let value: Option<Vec<u8>> =
        sqlx::query_scalar("SELECT value FROM metadata WHERE key='projection_identity'")
            .fetch_optional(&store.reader)
            .await?;
    value
        .filter(|value| !value.is_empty())
        .map(|value| ProjectionIdentity::decode(&value))
        .transpose()
}

pub(super) async fn write(
    store: &BinaryStore,
    identity: &ProjectionIdentity,
) -> Result<(), StoreError> {
    let _turn = store.writer_turn.lock().await;
    sqlx::query("INSERT INTO metadata(key,value) VALUES('projection_identity',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value")
        .bind(identity.encode()?).execute(&store.writer).await?;
    Ok(())
}

pub(super) async fn mark_rebuild(store: &BinaryStore) -> Result<(), StoreError> {
    let _turn = store.writer_turn.lock().await;
    sqlx::query("UPDATE metadata SET value=X'01' WHERE key='projection_rebuild'")
        .execute(&store.writer)
        .await?;
    Ok(())
}
