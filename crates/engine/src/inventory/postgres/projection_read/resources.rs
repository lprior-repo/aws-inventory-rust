use super::{Postgres, ProjectionResource, ResourceKey, StoreError, Transaction, validate_limit};
use inventory_core::inventory::{AccountId, Kind, Region, ResourceId};
use sqlx::{Row, postgres::PgRow};

pub(super) async fn read(
    tx: &mut Transaction<'_, Postgres>,
    after: Option<ResourceKey>,
    limit: u16,
) -> Result<Vec<ProjectionResource>, StoreError> {
    validate_limit(limit)?;
    let rows = match after {
        Some(key) => sqlx::query("SELECT account,region,kind,id,name,content_hash FROM resources WHERE (account,region,kind,id)>($1,$2,$3,$4) ORDER BY account,region,kind,id LIMIT $5")
            .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str()).bind(i64::from(limit)).fetch_all(&mut **tx).await?,
        None => sqlx::query("SELECT account,region,kind,id,name,content_hash FROM resources ORDER BY account,region,kind,id LIMIT $1")
            .bind(i64::from(limit)).fetch_all(&mut **tx).await?,
    };
    rows.into_iter().map(decode).collect()
}

pub(super) async fn one(
    tx: &mut Transaction<'_, Postgres>,
    key: &ResourceKey,
) -> Result<Option<ProjectionResource>, StoreError> {
    let row = sqlx::query("SELECT account,region,kind,id,name,content_hash FROM resources WHERE account=$1 AND region=$2 AND kind=$3 AND id=$4")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str()).fetch_optional(&mut **tx).await?;
    row.map(decode).transpose()
}

fn decode(row: PgRow) -> Result<ProjectionResource, StoreError> {
    let hash: Vec<u8> = row.try_get("content_hash")?;
    Ok(ProjectionResource {
        key: ResourceKey {
            account: AccountId::parse(&row.try_get::<String, _>("account")?)?,
            region: Region::parse(&row.try_get::<String, _>("region")?)?,
            kind: Kind::parse(&row.try_get::<String, _>("kind")?)?,
            id: ResourceId::parse(&row.try_get::<String, _>("id")?)?,
        },
        name: row.try_get("name")?,
        content_hash: hash
            .try_into()
            .map_err(|_| StoreError::Invalid("resource content hash length"))?,
    })
}
