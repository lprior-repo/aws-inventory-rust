use super::arrays::Array;
use super::resources::{Prepared, State};
use crate::inventory::{SchemaRegistry, StoreError};
use inventory_core::inventory::{Kind, KindSchema, ResourceKey, ScalarType};
use sqlx::{Postgres, QueryBuilder, Transaction};
use std::collections::BTreeMap;

pub(in crate::inventory::postgres) async fn persist(
    tx: &mut Transaction<'_, Postgres>,
    registry: &SchemaRegistry,
    states: &BTreeMap<&ResourceKey, State<'_>>,
) -> Result<(), StoreError> {
    let mut groups = groups(states)?;
    for schema in registry.schemas() {
        if let Some(rows) = groups.remove(schema.kind()) {
            let mut query = statement(schema, &rows)?;
            query.build().execute(&mut **tx).await?;
        }
    }
    if !groups.is_empty() {
        return Err(StoreError::Invalid("observation kind has no schema"));
    }
    Ok(())
}

fn groups<'a>(
    states: &'a BTreeMap<&ResourceKey, State<'_>>,
) -> Result<BTreeMap<&'a Kind, Vec<&'a Prepared>>, StoreError> {
    let mut groups = BTreeMap::<_, Vec<_>>::new();
    for state in states.values().filter(|state| state.facet_changed) {
        let row = state
            .prepared
            .as_ref()
            .ok_or(StoreError::Invalid("resource was not prepared"))?;
        groups.entry(&state.key.kind).or_default().push(row);
    }
    Ok(groups)
}

fn statement(
    schema: &KindSchema,
    rows: &[&Prepared],
) -> Result<QueryBuilder<Postgres>, StoreError> {
    let mut query = header(schema);
    projection(&mut query, schema);
    query.push(" FROM UNNEST(");
    identity(&mut query, rows);
    payload(&mut query, rows);
    for column in schema.columns() {
        query.push(",");
        Array::new(rows, column)?.bind(&mut query);
    }
    query.push(") u(account,region,id,content_hash,first_seen,last_changed,record");
    for (index, _) in schema.columns().iter().enumerate() {
        query.push(",c").push(index);
    }
    query.push(")");
    conflict(&mut query, schema);
    Ok(query)
}

fn header(schema: &KindSchema) -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new("INSERT INTO inventory.\"");
    query
        .push(schema.table().as_str())
        .push("\"(account,region,id,content_hash,first_seen,last_changed,record");
    for column in schema.columns() {
        query.push(",\"").push(column.name()).push("\"");
    }
    query.push(
        ") SELECT u.account,u.region,u.id,u.content_hash,u.first_seen,u.last_changed,u.record",
    );
    query
}

fn projection(query: &mut QueryBuilder<Postgres>, schema: &KindSchema) {
    for (index, column) in schema.columns().iter().enumerate() {
        query.push(",");
        if column.scalar() == ScalarType::Timestamp {
            query.push("timestamptz 'epoch' + u.c").push(index);
        } else {
            query.push("u.c").push(index);
        }
    }
}

fn identity(query: &mut QueryBuilder<Postgres>, rows: &[&Prepared]) {
    let accounts: Vec<_> = rows
        .iter()
        .map(|row| row.aggregate.observation.key().account.as_str())
        .collect();
    let regions: Vec<_> = rows
        .iter()
        .map(|row| row.aggregate.observation.key().region.as_str())
        .collect();
    let ids: Vec<_> = rows
        .iter()
        .map(|row| row.aggregate.observation.key().id.as_str())
        .collect();
    query.push_bind(accounts).push("::text[],");
    query.push_bind(regions).push("::text[],");
    query.push_bind(ids).push("::text[],");
}

fn payload(query: &mut QueryBuilder<Postgres>, rows: &[&Prepared]) {
    let hashes: Vec<_> = rows.iter().map(|row| row.hash.as_slice()).collect();
    let first: Vec<_> = rows.iter().map(|row| row.first_seen).collect();
    let last: Vec<_> = rows.iter().map(|row| row.last_changed).collect();
    let records: Vec<_> = rows
        .iter()
        .map(|row| row.aggregate.observation.record().as_bytes())
        .collect();
    query.push_bind(hashes).push("::bytea[],");
    query.push_bind(first).push("::bigint[],");
    query.push_bind(last).push("::bigint[],");
    query.push_bind(records).push("::bytea[]");
}

fn conflict(query: &mut QueryBuilder<Postgres>, schema: &KindSchema) {
    query.push(" ON CONFLICT(account,region,id) DO UPDATE SET content_hash=EXCLUDED.content_hash,last_changed=EXCLUDED.last_changed,record=EXCLUDED.record");
    for column in schema.columns() {
        query
            .push(",\"")
            .push(column.name())
            .push("\"=EXCLUDED.\"")
            .push(column.name())
            .push("\"");
    }
    query
        .push(" WHERE \"")
        .push(schema.table().as_str())
        .push("\".content_hash IS DISTINCT FROM EXCLUDED.content_hash OR \"");
    query
        .push(schema.table().as_str())
        .push("\".record IS DISTINCT FROM EXCLUDED.record");
}
