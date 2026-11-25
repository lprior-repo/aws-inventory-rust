mod binary;
#[cfg(test)]
mod tests;

use crate::inventory::StoreError;
pub(super) use binary::{Field, field, row};
use futures_util::{TryStreamExt, stream};
use inventory_core::inventory::{MAX_SQL_NAME_BYTES, SqlName};
use sha2::{Digest, Sha224};
use sqlx::{PgConnection, Postgres, SqlSafeStr, Transaction, postgres::PgCopyIn};

const FLUSH_BYTES: usize = 65_536;

pub(super) fn kind_stage(table: &SqlName) -> Result<String, StoreError> {
    let table = table.as_str();
    if table.len() <= MAX_SQL_NAME_BYTES.saturating_sub("stg_".len()) {
        Ok(format!("stg_{table}"))
    } else {
        let mut hex = [0; 56];
        let suffix = base16ct::lower::encode_str(&Sha224::digest(table.as_bytes()), &mut hex)
            .map_err(|_| StoreError::Invalid("PostgreSQL staging name exceeds length limit"))?;
        Ok(format!("stg:{suffix}"))
    }
}

pub(super) async fn load<T>(
    tx: &mut Transaction<'_, Postgres>,
    create: impl SqlSafeStr,
    statement: &str,
    rows: &[T],
    encode: impl Fn(&mut Vec<u8>, &T) -> Result<(), StoreError>,
) -> Result<u64, StoreError> {
    sqlx::query(create)
        .persistent(false)
        .execute(&mut **tx)
        .await?;
    load_direct(tx, statement, rows, encode).await
}

pub(super) async fn load_direct<T>(
    tx: &mut Transaction<'_, Postgres>,
    statement: &str,
    rows: &[T],
    encode: impl Fn(&mut Vec<u8>, &T) -> Result<(), StoreError>,
) -> Result<u64, StoreError> {
    let mut copy = tx.copy_in_raw(statement).await?;
    match send(&mut copy, rows, encode).await {
        Ok(()) => Ok(copy.finish().await?),
        Err(error) => {
            if let Err(cleanup) = copy.abort("inventory COPY failed").await {
                tracing::warn!(error = %cleanup, "could not abort PostgreSQL COPY");
            }
            Err(error)
        }
    }
}

async fn send<T>(
    copy: &mut PgCopyIn<&mut PgConnection>,
    rows: &[T],
    encode: impl Fn(&mut Vec<u8>, &T) -> Result<(), StoreError>,
) -> Result<(), StoreError> {
    let mut buffer = Vec::new();
    binary::reserve(&mut buffer, FLUSH_BYTES)?;
    buffer.extend_from_slice(binary::HEADER);
    let (copy, mut buffer) = stream::iter(rows.iter().map(Ok))
        .try_fold((copy, buffer), |(copy, mut buffer), row| {
            let encoded = encode(&mut buffer, row);
            async move {
                encoded?;
                if buffer.len() >= FLUSH_BYTES {
                    copy.send(buffer.as_slice()).await?;
                    buffer.clear();
                }
                Ok::<_, StoreError>((copy, buffer))
            }
        })
        .await?;
    binary::reserve(&mut buffer, binary::TRAILER.len())?;
    buffer.extend_from_slice(&binary::TRAILER);
    copy.send(buffer.as_slice()).await?;
    Ok(())
}
