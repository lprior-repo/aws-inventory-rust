use super::{BinaryStore, StoreError};
use inventory_core::inventory::{TagMapping, TagRole};

const RESOURCE_ROLES_SQL: &str = include_str!("../../sql/inventory_resource_roles.sql");

pub(super) async fn create(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<(), StoreError> {
    sqlx::query("CREATE TABLE IF NOT EXISTS tag_mapping(role TEXT NOT NULL CHECK(role IN('owner','team','application')),position INTEGER NOT NULL CHECK(position>=0),tag_key TEXT NOT NULL,PRIMARY KEY(role,position),UNIQUE(role,tag_key)) WITHOUT ROWID")
        .execute(&mut **tx)
        .await?;
    sqlx::query("DROP VIEW IF EXISTS resource_roles")
        .execute(&mut **tx)
        .await?;
    sqlx::query(RESOURCE_ROLES_SQL).execute(&mut **tx).await?;
    Ok(())
}

impl BinaryStore {
    pub async fn replace_tag_mapping(&self, mapping: &TagMapping) -> Result<(), StoreError> {
        let (_turn, mut tx) = self.begin_write().await?;
        sqlx::query("DELETE FROM tag_mapping")
            .execute(&mut *tx)
            .await?;
        for role in TagRole::ALL {
            for (position, key) in mapping.keys(role).iter().enumerate() {
                let position = i64::try_from(position)
                    .map_err(|_| StoreError::Invalid("tag mapping position is out of range"))?;
                sqlx::query("INSERT INTO tag_mapping(role,position,tag_key) VALUES(?1,?2,?3)")
                    .bind(role.as_str())
                    .bind(position)
                    .bind(key)
                    .execute(&mut *tx)
                    .await?;
            }
        }
        tx.commit().await?;
        Ok(())
    }
}
