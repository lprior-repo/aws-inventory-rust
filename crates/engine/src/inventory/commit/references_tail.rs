use super::*;

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

pub(in super::super) fn edge_source(value: &str) -> Result<EdgeSource, StoreError> {
    match value {
        "model" => Ok(EdgeSource::Model),
        "reviewed-rule" => Ok(EdgeSource::ReviewedRule),
        "inferred-exact" => Ok(EdgeSource::InferredExact),
        _ => Err(StoreError::Invalid("unknown stored edge source")),
    }
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

pub(super) async fn persist_tags(
    tx: &mut Transaction<'_, Sqlite>,
    observation: &super::super::TagObservation,
    refresh: bool,
    observed_at: i64,
) -> Result<(), StoreError> {
    let resource = observation.resource();
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM resources WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4)")
        .bind(resource.account.as_str()).bind(resource.region.as_str())
        .bind(resource.kind.as_str()).bind(resource.id.as_str()).fetch_one(&mut **tx).await?;
    if !exists {
        return Err(StoreError::Invalid(
            "tag observation has no persisted resource",
        ));
    }
    persist_tag_coverage(tx, observation, observed_at).await?;
    if observation.coverage() != super::super::TagCoverage::Complete {
        return Ok(());
    }
    let existing = sqlx::query("SELECT tag_key,tag_value FROM tags WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 ORDER BY tag_key LIMIT 101")
        .bind(resource.account.as_str()).bind(resource.region.as_str())
        .bind(resource.kind.as_str()).bind(resource.id.as_str()).fetch_all(&mut **tx).await?
        .into_iter().map(|row| Ok((row.try_get::<String, _>("tag_key")?, row.try_get::<String, _>("tag_value")?)))
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    if existing
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .eq(observation
            .tags()
            .iter()
            .map(|tag| (tag.key(), tag.value())))
    {
        return Ok(());
    }
    sqlx::query("DELETE FROM tags WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4")
        .bind(resource.account.as_str())
        .bind(resource.region.as_str())
        .bind(resource.kind.as_str())
        .bind(resource.id.as_str())
        .execute(&mut **tx)
        .await?;
    if !observation.tags().is_empty() {
        let mut insert = QueryBuilder::<Sqlite>::new(
            "INSERT INTO tags(account,region,kind,id,tag_key,tag_value) ",
        );
        insert.push_values(observation.tags(), |mut row, tag| {
            row.push_bind(resource.account.as_str())
                .push_bind(resource.region.as_str())
                .push_bind(resource.kind.as_str())
                .push_bind(resource.id.as_str())
                .push_bind(tag.key())
                .push_bind(tag.value());
        });
        insert.build().execute(&mut **tx).await?;
    }
    if refresh {
        enqueue(tx, resource, false).await?;
    }
    Ok(())
}

async fn persist_tag_coverage(
    tx: &mut Transaction<'_, Sqlite>,
    observation: &super::super::TagObservation,
    observed_at: i64,
) -> Result<(), StoreError> {
    let key = observation.resource();
    let complete_at =
        (observation.coverage() == super::super::TagCoverage::Complete).then_some(observed_at);
    sqlx::query("INSERT INTO tag_observations(account,region,kind,id,coverage,observed_at,last_complete_at) VALUES(?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(account,region,kind,id) DO UPDATE SET coverage=excluded.coverage,observed_at=excluded.observed_at,last_complete_at=COALESCE(excluded.last_complete_at,tag_observations.last_complete_at)")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str()).bind(key.id.as_str())
        .bind(observation.coverage().as_str()).bind(observed_at).bind(complete_at)
        .execute(&mut **tx).await?;
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
