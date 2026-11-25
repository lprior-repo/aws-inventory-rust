use super::PgStore;
use crate::inventory::{ProjectionIdentity, ProjectionState, StoreError};
use sqlx::Row;

pub(super) async fn state(store: &PgStore) -> Result<ProjectionState, StoreError> {
    let row = sqlx::query("SELECT count(*) AS pending,COALESCE(bool_or(full_rebuild),false) OR EXISTS(SELECT 1 FROM metadata WHERE key='projection_rebuild' AND value=$1) AS rebuilding FROM projection_outbox")
        .bind(b"\x01".as_slice()).fetch_one(&store.pool).await?;
    Ok(ProjectionState {
        pending: u64::try_from(row.try_get::<i64, _>("pending")?)
            .map_err(|_| StoreError::Invalid("negative outbox count"))?,
        rebuild_required: row.try_get("rebuilding")?,
    })
}

pub(super) async fn identity(store: &PgStore) -> Result<Option<ProjectionIdentity>, StoreError> {
    let value: Option<Option<Vec<u8>>> = sqlx::query_scalar("SELECT CASE WHEN octet_length(value)<=4225 THEN value ELSE NULL END FROM metadata WHERE key='projection_identity'")
        .fetch_optional(&store.pool).await?;
    match value {
        None => Ok(None),
        Some(None) => Err(StoreError::Invalid("projection identity exceeds bounds")),
        Some(Some(value)) if value.is_empty() => Ok(None),
        Some(Some(value)) => ProjectionIdentity::decode(&value).map(Some),
    }
}

pub(super) async fn write_identity(
    store: &PgStore,
    identity: &ProjectionIdentity,
) -> Result<(), StoreError> {
    sqlx::query("INSERT INTO metadata(key,value) VALUES('projection_identity',$1) ON CONFLICT(key) DO UPDATE SET value=EXCLUDED.value")
        .bind(identity.encode()?).execute(&store.pool).await?;
    Ok(())
}

pub(super) async fn mark_rebuild(store: &PgStore) -> Result<(), StoreError> {
    sqlx::query("INSERT INTO metadata(key,value) VALUES('projection_rebuild',$1) ON CONFLICT(key) DO UPDATE SET value=EXCLUDED.value")
        .bind(b"\x01".as_slice()).execute(&store.pool).await?;
    Ok(())
}
