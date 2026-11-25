use super::{
    Postgres, ProjectionEdgeCursor, ProjectionEdgePage, ResourceKey, StoreError, Transaction,
    validate_limit,
};
use crate::inventory::ProjectionEdgeHeader;
use inventory_core::inventory::{AccountId, Kind, Region, Relation, ResourceId};
use sqlx::{
    Row,
    postgres::{PgArguments, PgRow},
    query::Query,
};

const READ: &str = "SELECT from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source FROM edges WHERE ($1::text IS NULL OR (from_account,from_region,from_kind,from_id)=($1,$2,$3,$4) OR (to_account,to_region,to_kind,to_id)=($1,$2,$3,$4)) AND ($5::text IS NULL OR (from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source)>($5,$6,$7,$8,$9,$10,$11,$12,$13,$14)) ORDER BY from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source LIMIT $15";

pub(super) async fn read(
    tx: &mut Transaction<'_, Postgres>,
    key: Option<&ResourceKey>,
    after: Option<ProjectionEdgeCursor>,
    limit: u16,
) -> Result<ProjectionEdgePage, StoreError> {
    validate_limit(limit)?;
    let rows = request(key, after.as_ref(), limit)
        .fetch_all(&mut **tx)
        .await?;
    page(rows, limit)
}

fn request<'a>(
    key: Option<&'a ResourceKey>,
    after: Option<&'a ProjectionEdgeCursor>,
    limit: u16,
) -> Query<'a, Postgres, PgArguments> {
    sqlx::query(READ)
        .bind(key.map(|key| key.account.as_str()))
        .bind(key.map(|key| key.region.as_str()))
        .bind(key.map(|key| key.kind.as_str()))
        .bind(key.map(|key| key.id.as_str()))
        .bind(after.map(|cursor| cursor.from_account.as_str()))
        .bind(after.map(|cursor| cursor.from_region.as_str()))
        .bind(after.map(|cursor| cursor.from_kind.as_str()))
        .bind(after.map(|cursor| cursor.from_id.as_str()))
        .bind(after.map(|cursor| cursor.to_account.as_str()))
        .bind(after.map(|cursor| cursor.to_region.as_str()))
        .bind(after.map(|cursor| cursor.to_kind.as_str()))
        .bind(after.map(|cursor| cursor.to_id.as_str()))
        .bind(after.map(|cursor| cursor.relation.as_str()))
        .bind(after.map(|cursor| cursor.source.as_str()))
        .bind(i64::from(limit).saturating_add(1))
}

fn page(rows: Vec<PgRow>, limit: u16) -> Result<ProjectionEdgePage, StoreError> {
    let more = rows.len() > usize::from(limit);
    let edges = rows
        .into_iter()
        .take(usize::from(limit))
        .map(decode)
        .collect::<Result<Vec<_>, _>>()?;
    let next = if more { edges.last().map(cursor) } else { None };
    Ok(ProjectionEdgePage { edges, next })
}

fn decode(row: PgRow) -> Result<ProjectionEdgeHeader, StoreError> {
    Ok(ProjectionEdgeHeader {
        from: key(&row, true)?,
        to: key(&row, false)?,
        relation: Relation::parse(&row.try_get::<String, _>("relation")?)?,
        source: row.try_get("source")?,
    })
}

fn key(row: &PgRow, from: bool) -> Result<ResourceKey, StoreError> {
    let names = if from {
        ("from_account", "from_region", "from_kind", "from_id")
    } else {
        ("to_account", "to_region", "to_kind", "to_id")
    };
    Ok(ResourceKey {
        account: AccountId::parse(&row.try_get::<String, _>(names.0)?)?,
        region: Region::parse(&row.try_get::<String, _>(names.1)?)?,
        kind: Kind::parse(&row.try_get::<String, _>(names.2)?)?,
        id: ResourceId::parse(&row.try_get::<String, _>(names.3)?)?,
    })
}

fn cursor(edge: &ProjectionEdgeHeader) -> ProjectionEdgeCursor {
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
