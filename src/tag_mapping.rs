use crate::cli::TagMappingCommand;
use anyhow::{Context, Result};
use inventory_core::inventory::{TagMapping, TagRole};
use inventory_engine::inventory::BinaryStore;
use serde_json::{Value, json};
use std::path::Path;

pub async fn execute(command: TagMappingCommand) -> Result<Value> {
    match command {
        TagMappingCommand::Apply { db, file } => apply(&db, &file).await,
    }
}

async fn apply(db: &Path, file: &Path) -> Result<Value> {
    let text = std::fs::read_to_string(file)
        .with_context(|| format!("read tag mapping {}", file.display()))?;
    let mapping = TagMapping::parse(&text)?;
    let store = BinaryStore::open(db).await?;
    store.replace_tag_mapping(&mapping).await?;
    let mut roles = serde_json::Map::new();
    for role in TagRole::ALL {
        roles.insert(role.as_str().to_owned(), json!(mapping.keys(role)));
    }
    Ok(json!({"database": db.display().to_string(), "mapping": roles}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use inventory_core::catalog::runtime::RuntimeCatalog;
    use inventory_engine::inventory::SchemaRegistry;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};

    const CATALOG: &[u8] = include_bytes!("../catalog/aws-api.catalog");

    async fn pool(db: &Path) -> Result<SqlitePool> {
        Ok(SqlitePool::connect_with(SqliteConnectOptions::new().filename(db)).await?)
    }

    async fn seeded_database(dir: &Path) -> Result<std::path::PathBuf> {
        let db = dir.join("inventory.sqlite");
        let runtime = RuntimeCatalog::from_bytes(CATALOG)?;
        let registry = SchemaRegistry::new(CATALOG, runtime.schemas().to_vec())?;
        drop(BinaryStore::create(&db, registry).await?);
        let pool = pool(&db).await?;
        sqlx::query("INSERT INTO resources(account,region,kind,id,name,content_hash,first_seen,last_changed) VALUES('123456789012','us-east-1','AWS::Example::Thing','i-1',NULL,x'00',0,0)")
            .execute(&pool)
            .await?;
        for (key, value) in [("Owner", "alice"), ("Team", "blue")] {
            sqlx::query("INSERT INTO tags(account,region,kind,id,tag_key,tag_value) VALUES('123456789012','us-east-1','AWS::Example::Thing','i-1',?1,?2)")
                .bind(key)
                .bind(value)
                .execute(&pool)
                .await?;
        }
        pool.close().await;
        Ok(db)
    }

    async fn owner_value(db: &Path) -> Result<Option<String>> {
        let pool = pool(db).await?;
        let (value,): (Option<String>,) =
            sqlx::query_as("SELECT value FROM resource_roles WHERE id='i-1' AND role='owner'")
                .fetch_one(&pool)
                .await?;
        pool.close().await;
        Ok(value)
    }

    #[tokio::test]
    async fn apply_replaces_mapping_so_roles_change_without_any_aws_client() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let db = seeded_database(dir.path()).await?;
        let first = dir.path().join("first.toml");
        std::fs::write(&first, "owner = [\"Owner\"]\nteam = [\"Team\"]\n")?;
        apply(&db, &first).await?;
        assert_eq!(owner_value(&db).await?, Some("alice".to_owned()));

        let second = dir.path().join("second.toml");
        std::fs::write(&second, "owner = [\"Team\"]\n")?;
        apply(&db, &second).await?;
        assert_eq!(owner_value(&db).await?, Some("blue".to_owned()));
        Ok(())
    }
}
