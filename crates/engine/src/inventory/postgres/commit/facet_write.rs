use super::facets::{Context, Facet};
use super::resources::State;
use crate::inventory::StoreError;
use inventory_core::inventory::ResourceKey;
use sqlx::{Postgres, QueryBuilder, Transaction};
use std::collections::BTreeMap;

type Row<'a> = (&'a ResourceKey, &'a Facet<'a>);

pub(super) async fn persist(
    tx: &mut Transaction<'_, Postgres>,
    states: &BTreeMap<&ResourceKey, State<'_>>,
    context: &Context<'_>,
) -> Result<(), StoreError> {
    let mut rows = Vec::new();
    for state in states.values().filter(|state| state.facet_changed) {
        let facet = state
            .facets
            .iter()
            .find(|facet| facet.identity == context.identity)
            .ok_or(StoreError::Invalid("replacement facet is absent"))?;
        rows.push((state.key, facet));
    }
    if !rows.is_empty() {
        let mut query = statement(&rows);
        query.build().execute(&mut **tx).await?;
    }
    Ok(())
}

fn statement(rows: &[Row<'_>]) -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new(
        "INSERT INTO resource_facets(account,region,kind,id,service,operation,source_key,source_authority,bound_request,precedence,record,owned,exclusions,relations,name,name_owned) SELECT * FROM UNNEST(",
    );
    scope(&mut query, rows);
    resource(&mut query, rows);
    identity(&mut query, rows);
    authority(&mut query, rows);
    records(&mut query, rows);
    presentation(&mut query, rows);
    query.push(") ON CONFLICT(account,region,kind,id,service,operation,source_key) DO UPDATE SET source_authority=EXCLUDED.source_authority,bound_request=EXCLUDED.bound_request,precedence=EXCLUDED.precedence,record=EXCLUDED.record,owned=EXCLUDED.owned,exclusions=EXCLUDED.exclusions,relations=EXCLUDED.relations,name=EXCLUDED.name,name_owned=EXCLUDED.name_owned");
    query
}

fn scope(query: &mut QueryBuilder<Postgres>, rows: &[Row<'_>]) {
    let accounts: Vec<_> = rows.iter().map(|(key, _)| key.account.as_str()).collect();
    let regions: Vec<_> = rows.iter().map(|(key, _)| key.region.as_str()).collect();
    query.push_bind(accounts).push("::text[],");
    query.push_bind(regions).push("::text[],");
}

fn resource(query: &mut QueryBuilder<Postgres>, rows: &[Row<'_>]) {
    let kinds: Vec<_> = rows.iter().map(|(key, _)| key.kind.as_str()).collect();
    let ids: Vec<_> = rows.iter().map(|(key, _)| key.id.as_str()).collect();
    query.push_bind(kinds).push("::text[],");
    query.push_bind(ids).push("::text[],");
}

fn identity(query: &mut QueryBuilder<Postgres>, rows: &[Row<'_>]) {
    let services: Vec<_> = rows
        .iter()
        .map(|(_, f)| f.identity.service.as_ref())
        .collect();
    let operations: Vec<_> = rows
        .iter()
        .map(|(_, f)| f.identity.operation.as_ref())
        .collect();
    let sources: Vec<_> = rows
        .iter()
        .map(|(_, f)| f.identity.source.as_ref())
        .collect();
    query.push_bind(services).push("::text[],");
    query.push_bind(operations).push("::text[],");
    query.push_bind(sources).push("::bytea[],");
}

fn authority(query: &mut QueryBuilder<Postgres>, rows: &[Row<'_>]) {
    let authorities: Vec<_> = rows
        .iter()
        .map(|(_, f)| f.source_authority.as_ref().map(<[u8; 32]>::as_slice))
        .collect();
    let requests: Vec<_> = rows
        .iter()
        .map(|(_, f)| f.bound_request.as_deref())
        .collect();
    let precedence: Vec<_> = rows.iter().map(|(_, f)| f.precedence).collect();
    query.push_bind(authorities).push("::bytea[],");
    query.push_bind(requests).push("::bytea[],");
    query.push_bind(precedence).push("::bigint[],");
}

fn records(query: &mut QueryBuilder<Postgres>, rows: &[Row<'_>]) {
    let records: Vec<_> = rows.iter().map(|(_, f)| f.record.as_bytes()).collect();
    let owned: Vec<_> = rows.iter().map(|(_, f)| f.owned.as_bytes()).collect();
    let exclusions: Vec<_> = rows.iter().map(|(_, f)| f.exclusions.as_bytes()).collect();
    let relations: Vec<_> = rows.iter().map(|(_, f)| f.relations.as_bytes()).collect();
    query.push_bind(records).push("::bytea[],");
    query.push_bind(owned).push("::bytea[],");
    query.push_bind(exclusions).push("::bytea[],");
    query.push_bind(relations).push("::bytea[],");
}

fn presentation(query: &mut QueryBuilder<Postgres>, rows: &[Row<'_>]) {
    let names: Vec<_> = rows.iter().map(|(_, f)| f.name.as_deref()).collect();
    let owned: Vec<_> = rows.iter().map(|(_, f)| i64::from(f.name_owned)).collect();
    query.push_bind(names).push("::text[],");
    query.push_bind(owned).push("::bigint[]");
}
