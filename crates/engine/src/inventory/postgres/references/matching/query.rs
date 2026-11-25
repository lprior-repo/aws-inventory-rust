use super::{Request, bound, terms};
use crate::inventory::StoreError;
use inventory_core::catalog::runtime::RelationTargetMode;
use inventory_core::inventory::{KindSchema, ReferenceTarget, ScalarType};
use sqlx::{Postgres, QueryBuilder};

type Input<'a, 'b> = [(usize, &'a Request<'b>)];

pub(super) fn build(
    schema: &KindSchema,
    requests: &[Request<'_>],
    indexes: &[usize],
) -> Result<Option<QueryBuilder<Postgres>>, StoreError> {
    let input = input(requests, indexes)?;
    let first = input
        .first()
        .map(|(_, request)| &request.reference.value.target)
        .ok_or(StoreError::Invalid("reference matching group is empty"))?;
    if !available(schema, first) {
        return Ok(None);
    }
    let mut query = QueryBuilder::new("WITH input AS (SELECT * FROM UNNEST(");
    arguments(&mut query, &input, first)?;
    header(&mut query, schema, first);
    predicates(&mut query, schema, first)?;
    query.push(") p");
    Ok(Some(query))
}

fn input<'a, 'b>(
    requests: &'a [Request<'b>],
    indexes: &[usize],
) -> Result<Vec<(usize, &'a Request<'b>)>, StoreError> {
    indexes
        .iter()
        .map(|index| {
            requests
                .get(*index)
                .map(|request| (*index, request))
                .ok_or(StoreError::Invalid("reference index exceeds bounds"))
        })
        .collect()
}

fn available(schema: &KindSchema, target: &ReferenceTarget) -> bool {
    terms(target).all(|(mode, path, _)| {
        !matches!(
            mode,
            RelationTargetMode::SelectedFields | RelationTargetMode::SelectedRootText
        ) || schema
            .columns()
            .iter()
            .any(|column| column.path() == path && column.scalar() == ScalarType::Text)
    })
}

fn arguments(
    query: &mut QueryBuilder<Postgres>,
    input: &Input<'_, '_>,
    first: &ReferenceTarget,
) -> Result<(), StoreError> {
    let indexes = indexes(input)?;
    let accounts: Vec<_> = input
        .iter()
        .map(|(_, request)| request.reference.value.target.account().as_str())
        .collect();
    let regions: Vec<_> = input
        .iter()
        .map(|(_, request)| request.reference.value.target.region().as_str())
        .collect();
    let expected = expected(input);
    query.push_bind(indexes).push("::bigint[],");
    query.push_bind(accounts).push("::text[],");
    query.push_bind(regions).push("::text[],");
    query.push_bind(expected).push("::text[]");
    values(query, input, terms(first).count());
    Ok(())
}

fn indexes(input: &Input<'_, '_>) -> Result<Vec<i64>, StoreError> {
    input
        .iter()
        .map(|(index, _)| {
            i64::try_from(*index).map_err(|_| StoreError::Invalid("reference index exceeds bounds"))
        })
        .collect()
}

fn expected<'a>(input: &Input<'_, 'a>) -> Vec<Option<&'a str>> {
    input
        .iter()
        .map(|(_, request)| request.expected.map(|id| id.as_str()))
        .collect()
}

fn values(query: &mut QueryBuilder<Postgres>, input: &Input<'_, '_>, count: usize) {
    let mut columns: Vec<Vec<&str>> = std::iter::repeat_with(|| Vec::with_capacity(input.len()))
        .take(count)
        .collect();
    for (_, request) in input {
        for (column, (_, _, value)) in columns
            .iter_mut()
            .zip(terms(&request.reference.value.target))
        {
            column.push(value.as_str());
        }
    }
    for column in columns {
        query.push(",").push_bind(column).push("::text[]");
    }
}

fn header(query: &mut QueryBuilder<Postgres>, schema: &KindSchema, target: &ReferenceTarget) {
    query.push(") u(reference_index,account,region,expected");
    for (index, _) in terms(target).enumerate() {
        query.push(",v").push(index);
    }
    query.push(")) SELECT i.reference_index,p.id,p.bound_request FROM input i CROSS JOIN LATERAL (SELECT p.id,");
    if terms(target).any(|(mode, _, _)| bound(mode)) {
        query.push("f.bound_request FROM \"").push(schema.table().as_str()).push("\" p JOIN resource_facets f ON f.account=p.account AND f.region=p.region AND f.id=p.id AND f.kind=")
            .push_bind(schema.kind().as_str())
            .push(" AND f.source_authority IS NOT NULL AND f.bound_request IS NOT NULL");
    } else {
        query
            .push("NULL::bytea AS bound_request FROM \"")
            .push(schema.table().as_str())
            .push("\" p");
    }
    query.push(" WHERE p.account=i.account AND p.region=i.region AND (i.expected IS NULL OR p.id=i.expected)");
}

fn predicates(
    query: &mut QueryBuilder<Postgres>,
    schema: &KindSchema,
    target: &ReferenceTarget,
) -> Result<(), StoreError> {
    for (index, (mode, path, _)) in terms(target).enumerate() {
        match mode {
            RelationTargetMode::CanonicalId => {
                query.push(" AND p.id=i.v").push(index);
            }
            RelationTargetMode::SelectedFields | RelationTargetMode::SelectedRootText => {
                let column = column(schema, path)?;
                query
                    .push(" AND p.\"")
                    .push(column.name())
                    .push("\"=i.v")
                    .push(index);
            }
            RelationTargetMode::BoundContextFields | RelationTargetMode::BoundContextRootText => {}
        }
    }
    if !terms(target).any(|(mode, _, _)| bound(mode)) {
        query.push(" ORDER BY p.id LIMIT 2");
    }
    Ok(())
}

fn column<'a>(
    schema: &'a KindSchema,
    path: &[inventory_core::record::FieldId],
) -> Result<&'a inventory_core::inventory::Column, StoreError> {
    schema
        .columns()
        .iter()
        .find(|column| column.path() == path && column.scalar() == ScalarType::Text)
        .ok_or(StoreError::Invalid("reference target column is absent"))
}
