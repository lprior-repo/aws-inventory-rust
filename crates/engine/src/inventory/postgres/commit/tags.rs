mod storage;

use super::keys::Keys;
use crate::inventory::{PageCommit, StoreError, TagCoverage, TagObservation};
use futures_util::TryStreamExt;
use inventory_core::inventory::ResourceKey;
use sqlx::{Postgres, Row, Transaction, postgres::PgRow};
use std::collections::BTreeSet;

const LOAD: &str = "WITH input AS (SELECT * FROM UNNEST($1::text[],$2::text[],$3::text[],$4::text[],$5::text[]) WITH ORDINALITY u(a,r,k,id,c,ordinal)), coverage AS (INSERT INTO tag_observations(account,region,kind,id,coverage,observed_at,last_complete_at) SELECT i.a,i.r,i.k,i.id,i.c,$6,CASE WHEN i.c='complete' THEN $6::bigint END FROM input i JOIN resources x ON x.account=i.a AND x.region=i.r AND x.kind=i.k AND x.id=i.id ON CONFLICT(account,region,kind,id) DO UPDATE SET coverage=EXCLUDED.coverage,observed_at=EXCLUDED.observed_at,last_complete_at=COALESCE(EXCLUDED.last_complete_at,tag_observations.last_complete_at)) SELECT i.ordinal,x.id IS NOT NULL AS present,t.tag_key,t.tag_value FROM input i LEFT JOIN resources x ON x.account=i.a AND x.region=i.r AND x.kind=i.k AND x.id=i.id LEFT JOIN LATERAL(SELECT tag_key,tag_value FROM tags WHERE account=i.a AND region=i.r AND kind=i.k AND id=i.id AND i.c='complete' ORDER BY tag_key COLLATE \"C\" LIMIT 101) t ON true ORDER BY i.ordinal,t.tag_key COLLATE \"C\"";

#[derive(Clone, Default)]
struct Comparison {
    count: usize,
    changed: bool,
}

pub(super) async fn persist<'a>(
    tx: &mut Transaction<'_, Postgres>,
    page: &'a PageCommit,
) -> Result<BTreeSet<&'a ResourceKey>, StoreError> {
    if page.tags.is_empty() {
        return Ok(BTreeSet::new());
    }
    let mut compared = vec![Comparison::default(); page.tags.len()];
    load(tx, page, &mut compared).await?;
    let selected = selected(&page.tags, &compared);
    storage::replace(tx, &selected).await?;
    Ok(selected.into_iter().map(TagObservation::resource).collect())
}

async fn load(
    tx: &mut Transaction<'_, Postgres>,
    page: &PageCommit,
    compared: &mut [Comparison],
) -> Result<(), StoreError> {
    let keys = Keys::new(page.tags.iter().map(TagObservation::resource));
    let coverage: Vec<_> = page
        .tags
        .iter()
        .map(|tags| tags.coverage().as_str())
        .collect();
    let mut rows = sqlx::query(LOAD)
        .bind(&keys.accounts)
        .bind(&keys.regions)
        .bind(&keys.kinds)
        .bind(&keys.ids)
        .bind(coverage)
        .bind(page.observed_at_ms)
        .fetch(&mut **tx);
    while let Some(row) = rows.try_next().await? {
        compare(&row, &page.tags, compared)?;
    }
    Ok(())
}

fn compare(
    row: &PgRow,
    tags: &[TagObservation],
    compared: &mut [Comparison],
) -> Result<(), StoreError> {
    if !row.try_get::<bool, _>("present")? {
        return Err(StoreError::Invalid(
            "tag observation has no persisted resource",
        ));
    }
    let index = super::resources::ordinal(row)?;
    let observation = tags
        .get(index)
        .ok_or(StoreError::Invalid("tag index exceeds bounds"))?;
    if observation.coverage() == TagCoverage::Complete {
        let state = compared
            .get_mut(index)
            .ok_or(StoreError::Invalid("tag index exceeds bounds"))?;
        compare_tag(row, observation, state)?;
    }
    Ok(())
}

fn compare_tag(
    row: &PgRow,
    observation: &TagObservation,
    state: &mut Comparison,
) -> Result<(), StoreError> {
    let key: Option<&str> = row.try_get("tag_key")?;
    let value: Option<&str> = row.try_get("tag_value")?;
    if let Some(pair) = key.zip(value) {
        state.changed |= observation
            .tags()
            .get(state.count)
            .map(|tag| (tag.key(), tag.value()))
            != Some(pair);
        state.count = state
            .count
            .checked_add(1)
            .ok_or(StoreError::Invalid("tag count overflow"))?;
    }
    Ok(())
}

fn selected<'a>(tags: &'a [TagObservation], compared: &[Comparison]) -> Vec<&'a TagObservation> {
    tags.iter()
        .zip(compared)
        .filter(|(tags, state)| {
            tags.coverage() == TagCoverage::Complete
                && (state.changed || state.count != tags.tags().len())
        })
        .map(|(tags, _)| tags)
        .collect()
}
