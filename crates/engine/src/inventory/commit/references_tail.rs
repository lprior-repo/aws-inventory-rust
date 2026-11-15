use super::*;

pub(super) async fn unresolved_for(
    tx: &mut Transaction<'_, Sqlite>,
    scope: &[u8],
    key: &ResourceKey,
    path: &[u8],
    value: &str,
) -> Result<Vec<sqlx::sqlite::SqliteRow>, StoreError> {
    Ok(sqlx::query("SELECT from_account,from_region,from_kind,from_id,relation,source FROM unresolved_refs WHERE target_scope=?1 AND target_kind=?2 AND target_path=?3 AND target_value=?4")
        .bind(scope).bind(key.kind.as_str()).bind(path).bind(value).fetch_all(&mut **tx).await?)
}

pub(super) struct ReferenceMatch<'a> {
    pub(super) table: &'a str,
    pub(super) column: &'a str,
    pub(super) target: &'a ResourceKey,
    pub(super) value: &'a str,
}

pub(super) async fn retract_ambiguous_edges(
    tx: &mut Transaction<'_, Sqlite>,
    refs: &[sqlx::sqlite::SqliteRow],
    matched: ReferenceMatch<'_>,
) -> Result<(), StoreError> {
    for row in refs {
        let from = source_key(row)?;
        let relation = row.try_get::<String, _>("relation")?;
        let source = row.try_get::<String, _>("source")?;
        let mut query = QueryBuilder::<Sqlite>::new("DELETE FROM edges WHERE from_account=");
        query
            .push_bind(from.account.as_str())
            .push(" AND from_region=")
            .push_bind(from.region.as_str())
            .push(" AND from_kind=")
            .push_bind(from.kind.as_str());
        query
            .push(" AND from_id=")
            .push_bind(from.id.as_str())
            .push(" AND relation=")
            .push_bind(&relation)
            .push(" AND source=")
            .push_bind(&source);
        query
            .push(" AND to_account=")
            .push_bind(matched.target.account.as_str())
            .push(" AND to_region=")
            .push_bind(matched.target.region.as_str());
        query
            .push(" AND to_kind=")
            .push_bind(matched.target.kind.as_str())
            .push(" AND to_id IN(SELECT id FROM ")
            .push(quote(matched.table));
        query
            .push(" WHERE account=")
            .push_bind(matched.target.account.as_str())
            .push(" AND region=")
            .push_bind(matched.target.region.as_str());
        query
            .push(" AND ")
            .push(quote(matched.column))
            .push("=")
            .push_bind(matched.value)
            .push(")");
        if query.build().execute(&mut **tx).await?.rows_affected() > 0 {
            enqueue(tx, &from, false).await?;
        }
    }
    Ok(())
}

pub(in super::super) fn source_key(
    row: &sqlx::sqlite::SqliteRow,
) -> Result<ResourceKey, StoreError> {
    Ok(ResourceKey {
        account: inventory_core::inventory::AccountId::parse(
            &row.try_get::<String, _>("from_account")?,
        )?,
        region: inventory_core::inventory::Region::parse(
            &row.try_get::<String, _>("from_region")?,
        )?,
        kind: inventory_core::inventory::Kind::parse(&row.try_get::<String, _>("from_kind")?)?,
        id: inventory_core::inventory::ResourceId::parse(&row.try_get::<String, _>("from_id")?)?,
    })
}

pub(super) async fn resolve_one(
    tx: &mut Transaction<'_, Sqlite>,
    row: sqlx::sqlite::SqliteRow,
    target: &ResourceKey,
) -> Result<(), StoreError> {
    let from = ResourceKey {
        account: inventory_core::inventory::AccountId::parse(
            &row.try_get::<String, _>("from_account")?,
        )?,
        region: inventory_core::inventory::Region::parse(
            &row.try_get::<String, _>("from_region")?,
        )?,
        kind: inventory_core::inventory::Kind::parse(&row.try_get::<String, _>("from_kind")?)?,
        id: inventory_core::inventory::ResourceId::parse(&row.try_get::<String, _>("from_id")?)?,
    };
    let relation =
        inventory_core::inventory::Relation::parse(&row.try_get::<String, _>("relation")?)?;
    let source = edge_source(&row.try_get::<String, _>("source")?)?;
    if persist_edge(
        tx,
        &Edge {
            from: from.clone(),
            to: target.clone(),
            relation,
            source,
        },
    )
    .await?
    {
        enqueue(tx, &from, false).await?;
        enqueue(tx, target, false).await?;
    }
    Ok(())
}

pub(super) fn edge_source(value: &str) -> Result<EdgeSource, StoreError> {
    match value {
        "model" => Ok(EdgeSource::Model),
        "reviewed-rule" => Ok(EdgeSource::ReviewedRule),
        "inferred-exact" => Ok(EdgeSource::InferredExact),
        _ => Err(StoreError::Invalid("unknown stored edge source")),
    }
}

pub(in super::super) fn text_field<'a>(
    observation: &'a Observation,
    path: &[FieldId],
) -> Result<Option<&'a str>, StoreError> {
    path.iter()
        .try_fold(
            Some(ValueRef::Map(observation.record().root())),
            |current, field| match current {
                Some(ValueRef::Map(map)) => map.get_field(*field).map_err(StoreError::from),
                _ => Ok(None),
            },
        )
        .map(|value| match value {
            Some(ValueRef::Text(text)) => Some(text),
            _ => None,
        })
}

pub(in super::super) fn reference_scope(
    account: &str,
    region: &str,
) -> Result<Vec<u8>, StoreError> {
    let mut bytes = Vec::new();
    put_component(&mut bytes, account)?;
    put_component(&mut bytes, region)?;
    Ok(bytes)
}

pub(super) fn put_component(out: &mut Vec<u8>, text: &str) -> Result<(), StoreError> {
    let len = u32::try_from(text.len())
        .map_err(|_| StoreError::Invalid("reference scope component too long"))?;
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(text.as_bytes());
    Ok(())
}

pub(in super::super) fn reference_path(path: &[FieldId]) -> Vec<u8> {
    path.iter()
        .flat_map(|field| field.value().to_be_bytes())
        .collect()
}

pub(super) async fn persist_tag(
    tx: &mut Transaction<'_, Sqlite>,
    tag: &super::super::Tag,
) -> Result<(), StoreError> {
    sqlx::query("INSERT INTO tags(account,region,kind,id,tag_key,tag_value) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(account,region,kind,id,tag_key) DO UPDATE SET tag_value=excluded.tag_value")
        .bind(tag.resource.account.as_str()).bind(tag.resource.region.as_str()).bind(tag.resource.kind.as_str()).bind(tag.resource.id.as_str())
        .bind(&tag.key).bind(&tag.value).execute(&mut **tx).await?;
    Ok(())
}

pub(in super::super) async fn enqueue(
    tx: &mut Transaction<'_, Sqlite>,
    key: &ResourceKey,
    deleted: bool,
) -> Result<(), StoreError> {
    let encoded = super::super::work::resource_bytes(key)?;
    let revision: i64 = sqlx::query_scalar("UPDATE metadata SET value=CAST(value AS INTEGER)+1 WHERE key='projection_revision' RETURNING value").fetch_one(&mut **tx).await?;
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM projection_outbox WHERE resource_key=?1)")
            .bind(&encoded)
            .fetch_one(&mut **tx)
            .await?;
    let pending: i64 = sqlx::query_scalar(
        "SELECT CAST(value AS INTEGER) FROM metadata WHERE key='projection_pending'",
    )
    .fetch_one(&mut **tx)
    .await?;
    if !exists && pending >= super::super::MAX_PROJECTION_ROWS {
        enqueue_rebuild(tx, revision).await?;
    } else {
        enqueue_resource(tx, &encoded, revision, deleted).await?;
        if !exists {
            sqlx::query(
                "UPDATE metadata SET value=CAST(value AS INTEGER)+1 WHERE key='projection_pending'",
            )
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(())
}

pub(super) async fn enqueue_rebuild(
    tx: &mut Transaction<'_, Sqlite>,
    revision: i64,
) -> Result<(), StoreError> {
    sqlx::query("DELETE FROM projection_outbox WHERE full_rebuild=0")
        .execute(&mut **tx)
        .await?;
    sqlx::query("UPDATE metadata SET value=1 WHERE key='projection_pending'")
        .execute(&mut **tx)
        .await?;
    sqlx::query("INSERT INTO projection_outbox(resource_key,revision,deleted,full_rebuild) VALUES(X'00',?1,0,1) ON CONFLICT(resource_key) DO UPDATE SET revision=excluded.revision,full_rebuild=1")
        .bind(revision).execute(&mut **tx).await?;
    sqlx::query("UPDATE metadata SET value=X'01' WHERE key='projection_rebuild'")
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub(super) async fn enqueue_resource(
    tx: &mut Transaction<'_, Sqlite>,
    encoded: &[u8],
    revision: i64,
    deleted: bool,
) -> Result<(), StoreError> {
    sqlx::query("INSERT INTO projection_outbox(resource_key,revision,deleted,full_rebuild) VALUES(?1,?2,?3,0) ON CONFLICT(resource_key) DO UPDATE SET revision=excluded.revision,deleted=excluded.deleted")
        .bind(encoded).bind(revision).bind(if deleted { 1_i64 } else { 0_i64 }).execute(&mut **tx).await?;
    Ok(())
}
