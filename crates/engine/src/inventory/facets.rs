mod merge;
mod relations;
mod exclusions;

use super::{PageCommit, SchemaRegistry, StoreError};
use inventory_core::inventory::{Observation, ResourceId, ResourceKey, UnresolvedReference};
use inventory_core::record::{Exclusions, Record};
use sqlx::{Row, Sqlite, Transaction};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub(super) const CREATE_SQL: &str = "CREATE TABLE resource_facets(account TEXT NOT NULL,region TEXT NOT NULL,kind TEXT NOT NULL,id TEXT NOT NULL,service TEXT NOT NULL,operation TEXT NOT NULL,source_key BLOB NOT NULL,source_authority BLOB CHECK(source_authority IS NULL OR length(source_authority)=32),bound_request BLOB,precedence INTEGER NOT NULL CHECK(precedence BETWEEN 0 AND 65535),record BLOB NOT NULL,owned BLOB NOT NULL,exclusions BLOB NOT NULL,relations BLOB NOT NULL,name TEXT,name_owned INTEGER NOT NULL CHECK(name_owned IN(0,1)),PRIMARY KEY(account,region,kind,id,service,operation,source_key),FOREIGN KEY(account,region,kind,id) REFERENCES resources(account,region,kind,id) ON DELETE CASCADE) WITHOUT ROWID";

pub(super) struct Aggregate {
    pub(super) observation: Observation,
    pub(super) references: Vec<(UnresolvedReference, Option<ResourceId>, super::relation_permissions::ReferenceProvenance)>,
    pub(super) relations_changed: bool,
    pub(super) record_changed: bool,
    pub(super) bound_context_changed: bool,
}

pub(super) async fn apply(
    registry: &SchemaRegistry, tx: &mut Transaction<'_, Sqlite>, page: &PageCommit, observation: &Observation,
    direct_references: &[(UnresolvedReference, ResourceId)],
    authority: Option<&super::relation_permissions::SourceAuthority<'_, '_>>,
) -> Result<Aggregate, StoreError> {
    let (relations_changed, record_changed, bound_context_changed) = replace(registry, tx, page, observation, direct_references, authority).await?;
    let mut aggregate = load(registry, tx, observation.key()).await?;
    aggregate.relations_changed = relations_changed;
    aggregate.record_changed = record_changed;
    aggregate.bound_context_changed = bound_context_changed;
    Ok(aggregate)
}

pub(super) async fn load(registry: &SchemaRegistry, tx: &mut Transaction<'_, Sqlite>, key: &ResourceKey) -> Result<Aggregate, StoreError> {
    let rows = sqlx::query("SELECT record,owned,exclusions,relations,name,name_owned,source_key,source_authority FROM resource_facets WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 ORDER BY precedence,service,operation,source_key LIMIT 4097")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .fetch_all(&mut **tx).await?;
    if rows.len() > 4096 { return Err(StoreError::Invalid("resource facet count exceeds bounds")); }
    aggregate(registry, key, rows)
}

async fn replace(
    registry: &SchemaRegistry, tx: &mut Transaction<'_, Sqlite>, page: &PageCommit, observation: &Observation,
    direct_references: &[(UnresolvedReference, ResourceId)],
    authority: Option<&super::relation_permissions::SourceAuthority<'_, '_>>,
) -> Result<(bool, bool, bool), StoreError> {
    let key = observation.key();
    let scope = page.lease.scope();
    let source = match authority {
        Some(authority) => std::borrow::Cow::Borrowed(authority.source_facet()),
        None => std::borrow::Cow::Owned(super::work::ownership_scope_bytes(scope, page.lease.bindings())?),
    };
    let bound_request = authority.map(|_| page.lease.bindings());
    let previous = sqlx::query("SELECT owned,name_owned,relations,record,bound_request FROM resource_facets WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 AND service=?5 AND operation=?6 AND source_key=?7")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .bind(scope.service().as_str()).bind(scope.operation().as_str()).bind(source.as_ref()).fetch_optional(&mut **tx).await?;
    let old_owned = previous.as_ref().map(|row| -> Result<Record, StoreError> {
        Ok(Record::from_bytes(row.try_get("owned")?)?)
    }).transpose()?;
    let name_owned = observation.name().is_some() || previous.as_ref().map(|row| row.try_get::<bool, _>("name_owned")).transpose()?.is_some_and(|owned| owned);
    let owned = merge::owned(old_owned.as_ref(), &merge::fields(observation.record())?)?;
    let exclusions = exclusions::encode(observation.hash_exclusions())?;
    let relations = relations::encode(registry, page, observation, direct_references, authority)?;
    let old_relations = previous.as_ref().map(|row| row.try_get::<Vec<u8>, _>("relations")).transpose()?;
    let relations_changed = old_relations.as_deref() != Some(relations.as_bytes());
    let old_record = previous.as_ref().map(|row| row.try_get::<&[u8], _>("record")).transpose()?;
    let record_changed = old_record != Some(observation.record().as_bytes());
    let old_request = previous.as_ref().map(|row| row.try_get::<Option<&[u8]>, _>("bound_request")).transpose()?.flatten();
    let bound_context_changed = old_request != bound_request;
    sqlx::query("INSERT INTO resource_facets(account,region,kind,id,service,operation,precedence,record,owned,exclusions,relations,name,name_owned,source_key,source_authority,bound_request) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16) ON CONFLICT(account,region,kind,id,service,operation,source_key) DO UPDATE SET precedence=excluded.precedence,record=excluded.record,owned=excluded.owned,exclusions=excluded.exclusions,relations=excluded.relations,name=excluded.name,name_owned=excluded.name_owned,source_authority=excluded.source_authority,bound_request=excluded.bound_request")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .bind(scope.service().as_str()).bind(scope.operation().as_str()).bind(i64::from(observation.facet_precedence()))
        .bind(observation.record().as_bytes()).bind(owned.as_bytes()).bind(exclusions.as_bytes())
        .bind(relations.as_bytes()).bind(observation.name()).bind(name_owned).bind(source.as_ref())
        .bind(authority.map(|authority| authority.source_authority().as_slice()))
        .bind(bound_request).execute(&mut **tx).await?;
    Ok((relations_changed, record_changed, bound_context_changed))
}

fn aggregate(registry: &SchemaRegistry, key: &ResourceKey, rows: Vec<sqlx::sqlite::SqliteRow>) -> Result<Aggregate, StoreError> {
    let mut merged = BTreeMap::new();
    let mut owners = BTreeMap::new();
    let mut exclusions = Vec::new();
    let mut map_exclusions = BTreeSet::new();
    let mut relation_records = Vec::new();
    let mut name = None;
    rows.into_iter().enumerate().try_for_each(|(facet, row)| -> Result<(), StoreError> {
        let record = Record::from_bytes(row.try_get("record")?)?;
        let owned = Record::from_bytes(row.try_get("owned")?)?;
        let excluded = Record::from_bytes(row.try_get("exclusions")?)?;
        merge::overlay(&mut merged, &mut owners, &owned, merge::fields(&record)?, facet)?;
        exclusions::append(&excluded, &mut exclusions, &mut map_exclusions)?;
        let source: Vec<u8> = row.try_get("source_key")?;
        let source_authority = row.try_get::<Option<&[u8]>, _>("source_authority")?
            .map(<[u8; 32]>::try_from).transpose()
            .map_err(|_| StoreError::Invalid("facet source authority is not 32 bytes"))?;
        relation_records.push((Record::from_bytes(row.try_get("relations")?)?, Arc::<[u8]>::from(source), source_authority));
        if row.try_get::<bool, _>("name_owned")? { name = row.try_get::<Option<String>, _>("name")?.map(Arc::from); }
        Ok(())
    })?;
    let masks = Exclusions::new(exclusions)?.with_map_keys(map_exclusions.into_iter().collect())?;
    let observation = Observation::new(key.clone(), merge::record(merged)?, &masks, name)?;
    let references = relation_records.iter().enumerate().try_fold(Vec::new(), |mut references, (facet, (record, source, source_authority))| {
        references.extend(relations::decode(registry, record, observation.key(), facet, &owners, source, source_authority.as_ref())?);
        Ok::<_, StoreError>(references)
    })?;
    Ok(Aggregate { observation, references, relations_changed: false, record_changed: false, bound_context_changed: false })
}

pub(super) async fn retire_source(
    registry: &SchemaRegistry, tx: &mut Transaction<'_, Sqlite>, key: &ResourceKey, source: &[u8],
) -> Result<Option<Aggregate>, StoreError> {
    sqlx::query("DELETE FROM resource_parents WHERE child_account=?1 AND child_region=?2 AND child_kind=?3 AND child_id=?4 AND scope_key=?5")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .bind(source).execute(&mut **tx).await?;
    let removed = sqlx::query("DELETE FROM resource_facets WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 AND source_key=?5")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .bind(source).execute(&mut **tx).await?;
    if removed.rows_affected() == 0 { return Ok(None); }
    Ok(Some(load(registry, tx, key).await?))
}
