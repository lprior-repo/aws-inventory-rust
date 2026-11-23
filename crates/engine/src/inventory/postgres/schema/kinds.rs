use crate::inventory::{StoreError, schema::encode_schema};
use futures_util::{TryStreamExt, stream};
use inventory_core::inventory::{KindSchema, ScalarType};
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;

pub(super) async fn create_batch(
    pool: &PgPool,
    schemas: &[Arc<KindSchema>],
) -> Result<(), StoreError> {
    let tx = pool.begin().await?;
    let tx = stream::iter(schemas.iter().map(Ok))
        .try_fold(tx, |mut tx, schema| async move {
            create(&mut tx, schema).await?;
            Ok::<_, StoreError>(tx)
        })
        .await?;
    tx.commit().await?;
    Ok(())
}

async fn create(tx: &mut Transaction<'_, Postgres>, schema: &KindSchema) -> Result<(), StoreError> {
    definition(schema)?.build().execute(&mut **tx).await?;
    sqlx::query("INSERT INTO inventory.kinds(kind,table_name) VALUES($1,$2)")
        .bind(schema.kind().as_str())
        .bind(schema.table().as_str())
        .execute(&mut **tx)
        .await?;
    sqlx::query("INSERT INTO inventory.kind_schemas(kind,table_name,definition) VALUES($1,$2,$3)")
        .bind(schema.kind().as_str())
        .bind(schema.table().as_str())
        .bind(encode_schema(schema)?)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

fn definition(schema: &KindSchema) -> Result<sqlx::QueryBuilder<Postgres>, StoreError> {
    identifier(schema.table().as_str())?;
    let mut query = sqlx::QueryBuilder::new("CREATE TABLE inventory.\"");
    query.push(schema.table().as_str()).push("\"(account text NOT NULL,region text NOT NULL,id text NOT NULL,content_hash bytea NOT NULL,first_seen bigint NOT NULL,last_changed bigint NOT NULL,record bytea NOT NULL,nested jsonb");
    schema.columns().iter().try_for_each(|column| {
        identifier(column.name())?;
        query.push(",\"").push(column.name()).push("\" ");
        query.push(scalar_type(column.scalar()));
        Ok::<_, StoreError>(())
    })?;
    query.push(",PRIMARY KEY(account,region,id))");
    Ok(query)
}

fn identifier(name: &str) -> Result<(), StoreError> {
    if name.is_empty()
        || name.len() > 63
        || name
            .bytes()
            .any(|byte| !(byte.is_ascii_alphanumeric() || byte == b'_'))
    {
        return Err(StoreError::Invalid("PostgreSQL identifier exceeds bounds"));
    }
    Ok(())
}

fn scalar_type(scalar: ScalarType) -> &'static str {
    match scalar {
        ScalarType::Bool => "boolean",
        ScalarType::Integer => "bigint",
        ScalarType::Real => "double precision",
        ScalarType::Text => "text",
        ScalarType::Blob => "bytea",
        ScalarType::Timestamp => "timestamptz",
    }
}
