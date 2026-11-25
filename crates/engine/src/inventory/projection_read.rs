mod tags;
mod types;
pub use types::{
    ProjectionEdgeCursor, ProjectionEdgeHeader, ProjectionEdgePage, ProjectionResource,
    ProjectionState, ProjectionTag, ProjectionTagCursor, ProjectionTagPage,
};

use super::{BinaryStore, StoreError};
use inventory_core::inventory::{AccountId, Kind, Region, Relation, ResourceId, ResourceKey};
use sqlx::{Row, Sqlite, Transaction};
use std::sync::Arc;
use tokio::sync::OwnedMutexGuard;

pub struct ProjectionSnapshot {
    tx: Transaction<'static, Sqlite>,
    _turn: Option<OwnedMutexGuard<()>>,
}

impl ProjectionSnapshot {
    /// Reads only persisted tags in this resource's exact account/region scope.
    pub async fn tags_for(
        &mut self,
        key: &ResourceKey,
        after: Option<ProjectionTagCursor>,
        limit: u16,
    ) -> Result<ProjectionTagPage, StoreError> {
        tags::read(&mut self.tx, key, after, limit).await
    }

    pub async fn resources(
        &mut self,
        after: Option<ResourceKey>,
        limit: u16,
    ) -> Result<Vec<ProjectionResource>, StoreError> {
        read_resources(&mut self.tx, after, limit).await
    }
    pub async fn resource(
        &mut self,
        key: &ResourceKey,
    ) -> Result<Option<ProjectionResource>, StoreError> {
        read_resource(&mut self.tx, key).await
    }
    pub async fn edges(
        &mut self,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> Result<ProjectionEdgePage, StoreError> {
        read_edges(&mut self.tx, after, limit).await
    }
    pub async fn edges_for(
        &mut self,
        key: &ResourceKey,
        after: Option<ProjectionEdgeCursor>,
        limit: u16,
    ) -> Result<ProjectionEdgePage, StoreError> {
        read_edges_for(&mut self.tx, key, after, limit).await
    }
    pub async fn finish(self) -> Result<(), StoreError> {
        self.tx.commit().await?;
        Ok(())
    }
}

pub(super) async fn begin_snapshot(store: &BinaryStore) -> Result<ProjectionSnapshot, StoreError> {
    let turn = if store.reader_shares_writer {
        Some(Arc::clone(&store.writer_turn).lock_owned().await)
    } else {
        None
    };
    Ok(ProjectionSnapshot {
        tx: store.reader.begin().await?,
        _turn: turn,
    })
}

pub(super) async fn state(store: &BinaryStore) -> Result<ProjectionState, StoreError> {
    let _turn = store.read_turn().await;
    let mut tx = store.reader.begin().await?;
    let pending: i64 = sqlx::query("SELECT COUNT(*) FROM projection_outbox")
        .fetch_one(&mut *tx)
        .await?
        .try_get(0)?;
    let marker: Vec<u8> = sqlx::query("SELECT value FROM metadata WHERE key='projection_rebuild'")
        .fetch_one(&mut *tx)
        .await?
        .try_get(0)?;
    tx.commit().await?;
    Ok(ProjectionState {
        pending: u64::try_from(pending)
            .map_err(|_| StoreError::Invalid("outbox count is negative"))?,
        rebuild_required: marker.as_slice() == [1],
    })
}

pub(super) async fn resources(
    store: &BinaryStore,
    after: Option<ResourceKey>,
    limit: u16,
) -> Result<Vec<ProjectionResource>, StoreError> {
    let _turn = store.read_turn().await;
    let mut tx = store.reader.begin().await?;
    let rows = read_resources(&mut tx, after, limit).await?;
    tx.commit().await?;
    Ok(rows)
}

pub(super) async fn resource(
    store: &BinaryStore,
    key: &ResourceKey,
) -> Result<Option<ProjectionResource>, StoreError> {
    let _turn = store.read_turn().await;
    let mut tx = store.reader.begin().await?;
    let row = read_resource(&mut tx, key).await?;
    tx.commit().await?;
    Ok(row)
}

pub(super) async fn edges_for(
    store: &BinaryStore,
    key: &ResourceKey,
    after: Option<ProjectionEdgeCursor>,
    limit: u16,
) -> Result<ProjectionEdgePage, StoreError> {
    let _turn = store.read_turn().await;
    let mut tx = store.reader.begin().await?;
    let page = read_edges_for(&mut tx, key, after, limit).await?;
    tx.commit().await?;
    Ok(page)
}

pub(super) async fn edges(
    store: &BinaryStore,
    after: Option<ProjectionEdgeCursor>,
    limit: u16,
) -> Result<ProjectionEdgePage, StoreError> {
    let _turn = store.read_turn().await;
    let mut tx = store.reader.begin().await?;
    let page = read_edges(&mut tx, after, limit).await?;
    tx.commit().await?;
    Ok(page)
}

async fn read_edges(
    tx: &mut Transaction<'_, Sqlite>,
    after: Option<ProjectionEdgeCursor>,
    limit: u16,
) -> Result<ProjectionEdgePage, StoreError> {
    validate_limit(limit)?;
    let rows = match after {
        Some(cursor) => sqlx::query("SELECT from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source FROM edges WHERE (from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source)>(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10) ORDER BY from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source LIMIT ?11")
            .bind(&cursor.from_account).bind(&cursor.from_region).bind(&cursor.from_kind).bind(&cursor.from_id)
            .bind(&cursor.to_account).bind(&cursor.to_region).bind(&cursor.to_kind).bind(&cursor.to_id)
            .bind(&cursor.relation).bind(&cursor.source).bind(fetch_limit(limit)).fetch_all(&mut **tx).await?,
        None => sqlx::query("SELECT from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source FROM edges ORDER BY from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source LIMIT ?1")
            .bind(fetch_limit(limit)).fetch_all(&mut **tx).await?,
    };
    edge_page(rows, limit)
}
async fn read_resources(
    tx: &mut Transaction<'_, Sqlite>,
    after: Option<ResourceKey>,
    limit: u16,
) -> Result<Vec<ProjectionResource>, StoreError> {
    validate_limit(limit)?;
    let rows = match after {
        Some(key) => sqlx::query("SELECT account,region,kind,id,name,content_hash FROM resources WHERE (account,region,kind,id)>(?1,?2,?3,?4) ORDER BY account,region,kind,id LIMIT ?5")
            .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str()).bind(i64::from(limit)).fetch_all(&mut **tx).await?,
        None => sqlx::query("SELECT account,region,kind,id,name,content_hash FROM resources ORDER BY account,region,kind,id LIMIT ?1")
            .bind(i64::from(limit)).fetch_all(&mut **tx).await?,
    };
    rows.into_iter().map(resource_row).collect()
}

async fn read_resource(
    tx: &mut Transaction<'_, Sqlite>,
    key: &ResourceKey,
) -> Result<Option<ProjectionResource>, StoreError> {
    let row = sqlx::query("SELECT account,region,kind,id,name,content_hash FROM resources WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str()).fetch_optional(&mut **tx).await?;
    row.map(resource_row).transpose()
}

async fn read_edges_for(
    tx: &mut Transaction<'_, Sqlite>,
    key: &ResourceKey,
    after: Option<ProjectionEdgeCursor>,
    limit: u16,
) -> Result<ProjectionEdgePage, StoreError> {
    validate_limit(limit)?;
    let rows = match after {
        Some(cursor) => sqlx::query("SELECT from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source FROM edges WHERE ((from_account=?1 AND from_region=?2 AND from_kind=?3 AND from_id=?4) OR (to_account=?1 AND to_region=?2 AND to_kind=?3 AND to_id=?4)) AND (from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source)>(?5,?6,?7,?8,?9,?10,?11,?12,?13,?14) ORDER BY from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source LIMIT ?15")
            .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
            .bind(&cursor.from_account).bind(&cursor.from_region).bind(&cursor.from_kind).bind(&cursor.from_id)
            .bind(&cursor.to_account).bind(&cursor.to_region).bind(&cursor.to_kind).bind(&cursor.to_id)
            .bind(&cursor.relation).bind(&cursor.source).bind(fetch_limit(limit)).fetch_all(&mut **tx).await?,
        None => sqlx::query("SELECT from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source FROM edges WHERE (from_account=?1 AND from_region=?2 AND from_kind=?3 AND from_id=?4) OR (to_account=?1 AND to_region=?2 AND to_kind=?3 AND to_id=?4) ORDER BY from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source LIMIT ?5")
            .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str()).bind(fetch_limit(limit)).fetch_all(&mut **tx).await?,
    };
    edge_page(rows, limit)
}

fn fetch_limit(limit: u16) -> i64 {
    i64::from(limit).saturating_add(1)
}

fn edge_page(
    rows: Vec<sqlx::sqlite::SqliteRow>,
    limit: u16,
) -> Result<ProjectionEdgePage, StoreError> {
    let more = rows.len() > usize::from(limit);
    let edges = rows
        .into_iter()
        .take(usize::from(limit))
        .map(edge_row)
        .collect::<Result<Vec<_>, _>>()?;
    let next = if more {
        edges.last().map(edge_cursor)
    } else {
        None
    };
    Ok(ProjectionEdgePage { edges, next })
}

fn validate_limit(limit: u16) -> Result<(), StoreError> {
    if limit == 0 || limit > 1000 {
        return Err(StoreError::Invalid("projection page size exceeds bounds"));
    }
    Ok(())
}

fn resource_row(row: sqlx::sqlite::SqliteRow) -> Result<ProjectionResource, StoreError> {
    let key = ResourceKey {
        account: AccountId::parse(&row.try_get::<String, _>("account")?)?,
        region: Region::parse(&row.try_get::<String, _>("region")?)?,
        kind: Kind::parse(&row.try_get::<String, _>("kind")?)?,
        id: ResourceId::parse(&row.try_get::<String, _>("id")?)?,
    };
    let hash: Vec<u8> = row.try_get("content_hash")?;
    let content_hash: [u8; 32] = hash
        .try_into()
        .map_err(|_| StoreError::Invalid("resource content hash length"))?;
    Ok(ProjectionResource {
        key,
        name: row.try_get("name")?,
        content_hash,
    })
}

fn edge_row(row: sqlx::sqlite::SqliteRow) -> Result<ProjectionEdgeHeader, StoreError> {
    let from = resource_fields(&row, "from_")?;
    let to = resource_fields(&row, "to_")?;
    Ok(ProjectionEdgeHeader {
        from,
        to,
        relation: Relation::parse(&row.try_get::<String, _>("relation")?)?,
        source: row.try_get("source")?,
    })
}

fn resource_fields(row: &sqlx::sqlite::SqliteRow, prefix: &str) -> Result<ResourceKey, StoreError> {
    let names = match prefix {
        "from_" => ("from_account", "from_region", "from_kind", "from_id"),
        "to_" => ("to_account", "to_region", "to_kind", "to_id"),
        _ => return Err(StoreError::Invalid("unknown edge key prefix")),
    };
    let account: String = row.try_get(names.0)?;
    let region: String = row.try_get(names.1)?;
    let kind: String = row.try_get(names.2)?;
    let id: String = row.try_get(names.3)?;
    Ok(ResourceKey {
        account: AccountId::parse(&account)?,
        region: Region::parse(&region)?,
        kind: Kind::parse(&kind)?,
        id: ResourceId::parse(&id)?,
    })
}

fn edge_cursor(edge: &ProjectionEdgeHeader) -> ProjectionEdgeCursor {
    ProjectionEdgeCursor {
        from_account: edge.from.account.as_str().to_owned(),
        from_region: edge.from.region.as_str().to_owned(),
        from_kind: edge.from.kind.as_str().to_owned(),
        from_id: edge.from.id.as_str().to_owned(),
        to_account: edge.to.account.as_str().to_owned(),
        to_region: edge.to.region.as_str().to_owned(),
        to_kind: edge.to.kind.as_str().to_owned(),
        to_id: edge.to.id.as_str().to_owned(),
        relation: edge.relation.as_str().to_owned(),
        source: edge.source.clone(),
    }
}
