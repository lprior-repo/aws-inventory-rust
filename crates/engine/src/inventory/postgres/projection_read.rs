mod edges;
mod resources;
mod tags;

use super::PgStore;
use crate::inventory::{
    ProjectionEdgeCursor, ProjectionEdgePage, ProjectionResource, ProjectionSnapshotRead,
    ProjectionTagCursor, ProjectionTagPage, StoreError,
};
use inventory_core::inventory::ResourceKey;
use sqlx::{Postgres, Transaction};

pub struct PgProjectionSnapshot {
    tx: Transaction<'static, Postgres>,
}

pub(super) async fn begin(store: &PgStore) -> Result<PgProjectionSnapshot, StoreError> {
    let mut tx = store.pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await?;
    sqlx::query("SELECT 1 FROM resources LIMIT 1")
        .execute(&mut *tx)
        .await?;
    Ok(PgProjectionSnapshot { tx })
}

impl ProjectionSnapshotRead for PgProjectionSnapshot {
    async fn resources(
        &mut self,
        after: Option<ResourceKey>,
        limit: u16,
    ) -> Result<Vec<ProjectionResource>, StoreError> {
        resources::read(&mut self.tx, after, limit).await
    }
    async fn resource(
        &mut self,
        key: &ResourceKey,
    ) -> Result<Option<ProjectionResource>, StoreError> {
        resources::one(&mut self.tx, key).await
    }
    async fn edges(
        &mut self,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> Result<ProjectionEdgePage, StoreError> {
        edges::read(&mut self.tx, None, after, limit).await
    }
    async fn edges_for(
        &mut self,
        key: &ResourceKey,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> Result<ProjectionEdgePage, StoreError> {
        edges::read(&mut self.tx, Some(key), after, limit).await
    }
    async fn tags_for(
        &mut self,
        key: &ResourceKey,
        after: Option<ProjectionTagCursor>,
        limit: u16,
    ) -> Result<ProjectionTagPage, StoreError> {
        tags::read(&mut self.tx, key, after, limit).await
    }
    async fn finish(self) -> Result<(), StoreError> {
        self.tx.commit().await?;
        Ok(())
    }
}

fn validate_limit(limit: u16) -> Result<(), StoreError> {
    if limit == 0 || limit > 1000 {
        return Err(StoreError::Invalid("projection page size exceeds bounds"));
    }
    Ok(())
}
