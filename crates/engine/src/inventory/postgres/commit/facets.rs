mod aggregate;

use crate::inventory::facets::{exclusions, merge, relations};
use crate::inventory::relation_permissions::SourceAuthority;
use crate::inventory::{PageCommit, SchemaRegistry, StoreError};
use inventory_core::inventory::{Observation, ResourceId, ResourceKey, UnresolvedReference};
use inventory_core::record::Record;
use sqlx::{Row, postgres::PgRow};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;

pub(super) use aggregate::build;

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Identity {
    pub service: Arc<str>,
    pub operation: Arc<str>,
    pub source: Arc<[u8]>,
}

type Reference<'a> = (&'a UnresolvedReference, Option<&'a ResourceId>);
type Index<'a> = BTreeMap<&'a ResourceKey, Vec<Reference<'a>>>;

pub(super) struct Context<'a> {
    pub registry: &'a SchemaRegistry,
    pub page: &'a PageCommit,
    references: Index<'a>,
    pub identity: Arc<Identity>,
    pub bindings: Arc<[u8]>,
}

pub(super) struct Facet<'a> {
    pub identity: Arc<Identity>,
    pub source_authority: Option<[u8; 32]>,
    pub bound_request: Option<Arc<[u8]>>,
    pub precedence: i64,
    pub record: Cow<'a, Record>,
    pub owned: Record,
    pub exclusions: Record,
    pub relations: Record,
    pub name: Option<Arc<str>>,
    pub name_owned: bool,
}

impl<'a> Context<'a> {
    pub(super) fn new(
        registry: &'a SchemaRegistry,
        page: &'a PageCommit,
        direct: &'a [(UnresolvedReference, ResourceId)],
    ) -> Result<Self, StoreError> {
        let scope = page.lease.scope();
        Ok(Self {
            registry,
            page,
            references: reference_index(page, direct),
            identity: Arc::new(Identity {
                service: Arc::from(scope.service().as_str()),
                operation: Arc::from(scope.operation().as_str()),
                source: crate::inventory::work::ownership_scope_bytes(
                    scope,
                    page.lease.bindings(),
                )?
                .into(),
            }),
            bindings: page.lease.bindings().into(),
        })
    }

    pub(super) fn references(&self, key: &ResourceKey) -> impl Iterator<Item = Reference<'a>> + '_ {
        self.references.get(key).into_iter().flatten().copied()
    }
}

pub(super) fn decode<'a>(row: &PgRow, context: &Context<'_>) -> Result<Facet<'a>, StoreError> {
    Ok(Facet {
        identity: decode_identity(row, &context.identity)?,
        source_authority: row
            .try_get::<Option<&[u8]>, _>("source_authority")?
            .map(<[u8; 32]>::try_from)
            .transpose()
            .map_err(|_| StoreError::Invalid("facet source authority is not 32 bytes"))?,
        bound_request: decode_request(row, &context.bindings)?,
        precedence: row.try_get("precedence")?,
        record: Cow::Owned(Record::from_bytes(row.try_get("record")?)?),
        owned: Record::from_bytes(row.try_get("owned")?)?,
        exclusions: Record::from_bytes(row.try_get("exclusions")?)?,
        relations: Record::from_bytes(row.try_get("relations")?)?,
        name: row.try_get::<Option<&str>, _>("name")?.map(Arc::from),
        name_owned: row.try_get::<i64, _>("name_owned")? != 0,
    })
}

pub(super) fn replace<'a>(
    facets: &mut Vec<Facet<'a>>,
    context: &Context<'_>,
    observation: &'a Observation,
    authority: Option<&SourceAuthority<'_, '_>>,
) -> Result<bool, StoreError> {
    let index = facets
        .iter()
        .position(|facet| facet.identity == context.identity);
    let previous = index.and_then(|index| facets.get(index));
    let next = replacement(context, observation, authority, previous)?;
    let changed = !previous.is_some_and(|old| equal(old, &next));
    let room = facets.len() < 4096;
    match index.and_then(|index| facets.get_mut(index)) {
        Some(old) => *old = next,
        None if room => facets.push(next),
        None => return Err(StoreError::Invalid("resource facet count exceeds bounds")),
    }
    Ok(changed)
}

fn replacement<'a>(
    context: &Context<'_>,
    observation: &'a Observation,
    authority: Option<&SourceAuthority<'_, '_>>,
    previous: Option<&Facet<'_>>,
) -> Result<Facet<'a>, StoreError> {
    let fields = merge::fields(observation.record())?;
    Ok(Facet {
        identity: Arc::clone(&context.identity),
        source_authority: authority.map(|value| *value.source_authority()),
        bound_request: authority.map(|_| Arc::clone(&context.bindings)),
        precedence: i64::from(observation.facet_precedence()),
        record: Cow::Borrowed(observation.record()),
        owned: merge::owned(previous.map(|facet| &facet.owned), &fields)?,
        exclusions: exclusions::encode(observation.hash_exclusions())?,
        relations: relation_record(context, observation, authority)?,
        name: observation.name().map(Arc::from),
        name_owned: observation.name().is_some() || previous.is_some_and(|facet| facet.name_owned),
    })
}

fn equal(left: &Facet<'_>, right: &Facet<'_>) -> bool {
    left.source_authority == right.source_authority
        && left.bound_request == right.bound_request
        && left.precedence == right.precedence
        && left.record.as_bytes() == right.record.as_bytes()
        && left.owned.as_bytes() == right.owned.as_bytes()
        && left.exclusions.as_bytes() == right.exclusions.as_bytes()
        && left.relations.as_bytes() == right.relations.as_bytes()
        && left.name == right.name
        && left.name_owned == right.name_owned
}

fn relation_record(
    context: &Context<'_>,
    observation: &Observation,
    authority: Option<&SourceAuthority<'_, '_>>,
) -> Result<Record, StoreError> {
    relations::encode(
        context.registry,
        &context.page.lease,
        observation,
        context.references(observation.key()),
        authority,
    )
}

fn reference_index<'a>(
    page: &'a PageCommit,
    direct: &'a [(UnresolvedReference, ResourceId)],
) -> Index<'a> {
    let references = page
        .unresolved
        .iter()
        .map(|reference| (reference, None))
        .chain(
            direct
                .iter()
                .map(|(reference, target)| (reference, Some(target))),
        );
    let mut index = BTreeMap::<_, Vec<_>>::new();
    for (reference, target) in references {
        index
            .entry(&reference.from)
            .or_default()
            .push((reference, target));
    }
    index
}

fn decode_identity(row: &PgRow, current: &Arc<Identity>) -> Result<Arc<Identity>, StoreError> {
    let service: &str = row.try_get("service")?;
    let operation: &str = row.try_get("operation")?;
    let source: &[u8] = row.try_get("source_key")?;
    if service == current.service.as_ref()
        && operation == current.operation.as_ref()
        && source == current.source.as_ref()
    {
        return Ok(Arc::clone(current));
    }
    Ok(Arc::new(Identity {
        service: Arc::from(service),
        operation: Arc::from(operation),
        source: Arc::from(source),
    }))
}

fn decode_request(row: &PgRow, current: &Arc<[u8]>) -> Result<Option<Arc<[u8]>>, StoreError> {
    let request: Option<&[u8]> = row.try_get("bound_request")?;
    Ok(request.map(|value| {
        if value == current.as_ref() {
            Arc::clone(current)
        } else {
            Arc::from(value)
        }
    }))
}
