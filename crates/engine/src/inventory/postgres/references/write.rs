use super::matching::Resolution;
use super::selection::Reference;
use crate::inventory::StoreError;
use crate::inventory::postgres::commit::keys::{self, Keys};
use crate::inventory::postgres::outbox;
use inventory_core::inventory::{ResourceKey, UnresolvedReference};
use sqlx::{Postgres, QueryBuilder, Transaction};
use std::collections::BTreeMap;

type Edge<'a> = (&'a UnresolvedReference, &'a ResourceKey);

const CHANGES: &str = "CREATE TEMP TABLE inventory_reference_changes ON COMMIT DROP AS WITH deleted AS (DELETE FROM edges e USING inventory_reference_sources s WHERE (e.from_account,e.from_region,e.from_kind,e.from_id)=(s.account,s.region,s.kind,s.id) AND NOT EXISTS(SELECT 1 FROM inventory_reference_edges d WHERE (d.from_account,d.from_region,d.from_kind,d.from_id,d.to_account,d.to_region,d.to_kind,d.to_id,d.relation,d.source)=(e.from_account,e.from_region,e.from_kind,e.from_id,e.to_account,e.to_region,e.to_kind,e.to_id,e.relation,e.source)) RETURNING e.*), inserted AS (INSERT INTO edges SELECT * FROM inventory_reference_edges ON CONFLICT DO NOTHING RETURNING *) SELECT *,false AS inserted FROM deleted UNION ALL SELECT *,true FROM inserted";
const ENDPOINTS: &str = "SELECT * FROM inventory_reference_endpoints WHERE $1::text IS NULL OR (account,region,kind,id)>($1,$2,$3,$4) ORDER BY account,region,kind,id LIMIT 1000";

pub(super) async fn batch(
    tx: &mut Transaction<'_, Postgres>,
    references: &[Reference],
    resolutions: &[Resolution],
) -> Result<(), StoreError> {
    statuses(tx, references, resolutions).await?;
    let mut edges = Vec::with_capacity(1000);
    for (reference, resolution) in references.iter().zip(resolutions) {
        for target in &resolution.targets {
            edges.push((&reference.value, target));
            if edges.len() == 1000 {
                persist(tx, &edges).await?;
                edges.clear();
            }
        }
    }
    if !edges.is_empty() {
        persist(tx, &edges).await?;
    }
    Ok(())
}

async fn persist(tx: &mut Transaction<'_, Postgres>, edges: &[Edge<'_>]) -> Result<(), StoreError> {
    let mut query =
        QueryBuilder::new("INSERT INTO inventory_reference_edges SELECT * FROM UNNEST(");
    Keys::new(edges.iter().map(|(reference, _)| &reference.from)).bind_arrays(&mut query);
    query.push(",");
    Keys::new(edges.iter().map(|(_, target)| *target)).bind_arrays(&mut query);
    let relations: Vec<_> = edges
        .iter()
        .map(|(reference, _)| reference.relation.as_str())
        .collect();
    let sources: Vec<_> = edges
        .iter()
        .map(|(reference, _)| reference.source.as_str())
        .collect();
    query
        .push(",")
        .push_bind(relations)
        .push("::text[],")
        .push_bind(sources)
        .push("::text[]) ON CONFLICT DO NOTHING");
    query.build().execute(&mut **tx).await?;
    Ok(())
}

async fn statuses(
    tx: &mut Transaction<'_, Postgres>,
    references: &[Reference],
    resolutions: &[Resolution],
) -> Result<(), StoreError> {
    let mut query =
        QueryBuilder::new("UPDATE unresolved_refs u SET resolution=i.status FROM UNNEST(");
    Keys::new(references.iter().map(|row| &row.value.from)).bind_arrays(&mut query);
    identity(&mut query, references);
    let statuses: Vec<_> = resolutions
        .iter()
        .map(|resolution| {
            if !resolution.targets.is_empty() {
                "resolved"
            } else if resolution.ambiguous {
                "ambiguous"
            } else {
                "unresolved"
            }
        })
        .collect();
    query.push(",").push_bind(statuses).push("::text[]) i(a,r,k,id,scope,kind,path,value,relation,source,status) WHERE (u.from_account,u.from_region,u.from_kind,u.from_id,u.target_scope,u.target_kind,u.target_path,u.target_value,u.relation,u.source)=(i.a,i.r,i.k,i.id,i.scope,i.kind,i.path,i.value,i.relation,i.source) AND u.resolution IS DISTINCT FROM i.status");
    query.build().execute(&mut **tx).await?;
    Ok(())
}

fn identity(query: &mut QueryBuilder<Postgres>, references: &[Reference]) {
    let scopes: Vec<_> = references.iter().map(|row| row.scope.as_slice()).collect();
    let kinds: Vec<_> = references
        .iter()
        .map(|row| row.value.target.kind().as_str())
        .collect();
    let paths: Vec<_> = references.iter().map(|row| row.path.as_slice()).collect();
    let values: Vec<_> = references
        .iter()
        .map(|row| row.value.target.value().as_str())
        .collect();
    target_identity(query, scopes, kinds, paths, values);
    source_identity(query, references);
}

fn target_identity(
    query: &mut QueryBuilder<Postgres>,
    scopes: Vec<&[u8]>,
    kinds: Vec<&str>,
    paths: Vec<&[u8]>,
    values: Vec<&str>,
) {
    query
        .push(",")
        .push_bind(scopes)
        .push("::bytea[],")
        .push_bind(kinds)
        .push("::text[],");
    query
        .push_bind(paths)
        .push("::bytea[],")
        .push_bind(values)
        .push("::text[],");
}

fn source_identity(query: &mut QueryBuilder<Postgres>, references: &[Reference]) {
    let relations: Vec<_> = references
        .iter()
        .map(|row| row.value.relation.as_str())
        .collect();
    let sources: Vec<_> = references
        .iter()
        .map(|row| row.value.source.as_str())
        .collect();
    query
        .push_bind(relations)
        .push("::text[],")
        .push_bind(sources)
        .push("::text[]");
}

pub(super) async fn finish(tx: &mut Transaction<'_, Postgres>) -> Result<u64, StoreError> {
    sqlx::query(CHANGES).execute(&mut **tx).await?;
    let inserted: i64 =
        sqlx::query_scalar("SELECT count(*) FROM inventory_reference_changes WHERE inserted")
            .fetch_one(&mut **tx)
            .await?;
    sqlx::query("CREATE TEMP TABLE inventory_reference_endpoints ON COMMIT DROP AS SELECT from_account AS account,from_region AS region,from_kind AS kind,from_id AS id FROM inventory_reference_changes UNION SELECT to_account,to_region,to_kind,to_id FROM inventory_reference_changes")
        .execute(&mut **tx).await?;
    sqlx::query(
        "ALTER TABLE inventory_reference_endpoints ADD PRIMARY KEY(account,region,kind,id)",
    )
    .execute(&mut **tx)
    .await?;
    enqueue(tx).await?;
    sqlx::query("DROP TABLE inventory_reference_sources,inventory_reference_edges,inventory_reference_changes,inventory_reference_endpoints")
        .execute(&mut **tx).await?;
    u64::try_from(inserted).map_err(|_| StoreError::Invalid("reference edge count exceeds bounds"))
}

async fn enqueue(tx: &mut Transaction<'_, Postgres>) -> Result<(), StoreError> {
    let mut after: Option<ResourceKey> = None;
    loop {
        let rows = sqlx::query(ENDPOINTS)
            .bind(after.as_ref().map(|key| key.account.as_str()))
            .bind(after.as_ref().map(|key| key.region.as_str()))
            .bind(after.as_ref().map(|key| key.kind.as_str()))
            .bind(after.as_ref().map(|key| key.id.as_str()))
            .fetch_all(&mut **tx)
            .await?;
        if rows.is_empty() {
            break;
        }
        let keys = rows
            .iter()
            .map(keys::decode)
            .collect::<Result<Vec<_>, _>>()?;
        let entries: BTreeMap<_, _> = keys.iter().map(|key| (key, false)).collect();
        outbox::enqueue(tx, &entries).await?;
        after = keys.into_iter().last();
    }
    Ok(())
}
