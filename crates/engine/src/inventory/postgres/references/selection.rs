use crate::inventory::StoreError;
use crate::inventory::commit::{edge_source, reference_scope};
use crate::inventory::postgres::commit::keys::Keys;
use crate::inventory::reference_candidates::{decode, decode_target};
use crate::inventory::work::transitions::Cursor;
use inventory_core::inventory::{
    AccountId, Kind, Region, Relation, ResourceId, ResourceKey, UnresolvedReference,
};
use sqlx::{Postgres, QueryBuilder, Row as SqlRow, Transaction, postgres::PgRow};
use std::collections::BTreeSet;

const SELECT: &str = "SELECT u.* FROM unresolved_refs u JOIN inventory_reference_sources s ON (u.from_account,u.from_region,u.from_kind,u.from_id)=(s.account,s.region,s.kind,s.id) WHERE $1::text IS NULL OR (u.from_account,u.from_region,u.from_kind,u.from_id,u.target_scope,u.target_kind,u.target_path,u.target_value,u.relation,u.source)>($1,$2,$3,$4,$5::bytea,$6,$7::bytea,$8,$9,$10) ORDER BY u.from_account,u.from_region,u.from_kind,u.from_id,u.target_scope,u.target_kind,u.target_path,u.target_value,u.relation,u.source LIMIT 256";

pub(super) struct Reference {
    pub value: UnresolvedReference,
    pub scope: Vec<u8>,
    pub path: Vec<u8>,
    pub grants: BTreeSet<Option<ResourceId>>,
}

pub(super) async fn begin(
    tx: &mut Transaction<'_, Postgres>,
    keys: &[&ResourceKey],
) -> Result<(), StoreError> {
    let mut query = QueryBuilder::new(
        "CREATE TEMP TABLE inventory_reference_sources ON COMMIT DROP AS WITH input AS (SELECT * FROM ",
    );
    Keys::new(keys.iter().copied()).bind(&mut query);
    query.push(" u(account,region,kind,id)), targets AS (SELECT * FROM UNNEST(");
    targets(&mut query, keys)?;
    query.push(") u(scope,kind)) SELECT * FROM input UNION SELECT c.from_account,c.from_region,c.from_kind,c.from_id FROM targets t JOIN unresolved_ref_candidates c ON c.target_scope=t.scope AND c.candidate_kind=t.kind");
    query.build().execute(&mut **tx).await?;
    sqlx::query("ALTER TABLE inventory_reference_sources ADD PRIMARY KEY(account,region,kind,id)")
        .execute(&mut **tx)
        .await?;
    sqlx::query("CREATE TEMP TABLE inventory_reference_edges(LIKE inventory.edges INCLUDING DEFAULTS INCLUDING CONSTRAINTS INCLUDING INDEXES) ON COMMIT DROP")
        .execute(&mut **tx).await?;
    Ok(())
}

fn targets(query: &mut QueryBuilder<Postgres>, keys: &[&ResourceKey]) -> Result<(), StoreError> {
    let groups: BTreeSet<_> = keys
        .iter()
        .map(|key| (&key.account, &key.region, &key.kind))
        .collect();
    let mut scopes = Vec::with_capacity(groups.len());
    let mut kinds = Vec::with_capacity(groups.len());
    for (account, region, kind) in groups {
        scopes.push(reference_scope(account.as_str(), region.as_str())?);
        kinds.push(kind.as_str());
    }
    query
        .push_bind(scopes)
        .push("::bytea[],")
        .push_bind(kinds)
        .push("::text[]");
    Ok(())
}

pub(super) async fn next(
    tx: &mut Transaction<'_, Postgres>,
    after: Option<&Reference>,
) -> Result<Vec<Reference>, StoreError> {
    let value = after.map(|row| &row.value);
    let rows = sqlx::query(SELECT)
        .bind(value.map(|value| value.from.account.as_str()))
        .bind(value.map(|value| value.from.region.as_str()))
        .bind(value.map(|value| value.from.kind.as_str()))
        .bind(value.map(|value| value.from.id.as_str()))
        .bind(after.map(|row| row.scope.as_slice()))
        .bind(value.map(|value| value.target.kind().as_str()))
        .bind(after.map(|row| row.path.as_slice()))
        .bind(value.map(|value| value.target.value().as_str()))
        .bind(value.map(|value| value.relation.as_str()))
        .bind(value.map(|value| value.source.as_str()))
        .fetch_all(&mut **tx)
        .await?;
    rows.iter().map(reference).collect()
}

fn reference(row: &PgRow) -> Result<Reference, StoreError> {
    let scope: Vec<u8> = row.try_get("target_scope")?;
    let path: Vec<u8> = row.try_get("target_path")?;
    let target = target(row, &scope, &path)?;
    let value = UnresolvedReference {
        from: source(row)?,
        relation: Relation::parse(row.try_get("relation")?)?,
        source: edge_source(row.try_get("source")?)?,
        target,
    };
    Ok(Reference {
        value,
        scope,
        path,
        grants: BTreeSet::new(),
    })
}

fn target(
    row: &PgRow,
    scope: &[u8],
    path: &[u8],
) -> Result<inventory_core::inventory::ReferenceTarget, StoreError> {
    let (account, region) = target_scope(scope)?;
    let kind = Kind::parse(row.try_get("target_kind")?)?;
    let kinds = decode(row.try_get("target_kinds")?, &kind)?;
    decode_target(
        account,
        region,
        kind,
        path,
        ResourceId::parse(row.try_get("target_value")?)?,
    )?
    .with_candidate_kinds(kinds)
    .map_err(StoreError::from)
}

fn source(row: &PgRow) -> Result<ResourceKey, StoreError> {
    Ok(ResourceKey {
        account: AccountId::parse(row.try_get("from_account")?)?,
        region: Region::parse(row.try_get("from_region")?)?,
        kind: Kind::parse(row.try_get("from_kind")?)?,
        id: ResourceId::parse(row.try_get("from_id")?)?,
    })
}

fn target_scope(bytes: &[u8]) -> Result<(AccountId, Region), StoreError> {
    let mut cursor = Cursor::new(bytes);
    let account = AccountId::parse(cursor.text()?)?;
    let region = Region::parse(cursor.text()?)?;
    if !cursor.finished() {
        return Err(StoreError::Invalid("reference scope has trailing bytes"));
    }
    Ok((account, region))
}
