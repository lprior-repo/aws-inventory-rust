use crate::inventory::postgres::commit::keys::Keys;
use crate::inventory::{StoreError, TagObservation};
use inventory_core::inventory::{ResourceKey, Tag};
use sqlx::{Postgres, QueryBuilder, Transaction};

type Pair<'a> = (&'a ResourceKey, &'a Tag);

pub(super) async fn replace(
    tx: &mut Transaction<'_, Postgres>,
    tags: &[&TagObservation],
) -> Result<(), StoreError> {
    if tags.is_empty() {
        return Ok(());
    }
    let keys = Keys::new(tags.iter().map(|tags| tags.resource()));
    let mut delete = QueryBuilder::new("DELETE FROM tags t USING ");
    keys.bind(&mut delete);
    delete.push(" u(a,r,k,id) WHERE t.account=u.a AND t.region=u.r AND t.kind=u.k AND t.id=u.id");
    delete.build().execute(&mut **tx).await?;
    let rows = pairs(tags)?;
    if !rows.is_empty() {
        insert(&rows).build().execute(&mut **tx).await?;
    }
    Ok(())
}

fn pairs<'a>(tags: &[&'a TagObservation]) -> Result<Vec<Pair<'a>>, StoreError> {
    let capacity = tags.iter().try_fold(0_usize, |count, tags| {
        count
            .checked_add(tags.tags().len())
            .ok_or(StoreError::Invalid("tag count overflow"))
    })?;
    let mut pairs = Vec::with_capacity(capacity);
    for observation in tags {
        pairs.extend(
            observation
                .tags()
                .iter()
                .map(|tag| (observation.resource(), tag)),
        );
    }
    Ok(pairs)
}

fn insert(rows: &[Pair<'_>]) -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new(
        "INSERT INTO tags(account,region,kind,id,tag_key,tag_value) SELECT * FROM UNNEST(",
    );
    Keys::new(rows.iter().map(|(key, _)| *key)).bind_arrays(&mut query);
    let keys: Vec<_> = rows.iter().map(|(_, tag)| tag.key()).collect();
    let values: Vec<_> = rows.iter().map(|(_, tag)| tag.value()).collect();
    query
        .push(",")
        .push_bind(keys)
        .push("::text[],")
        .push_bind(values)
        .push("::text[])");
    query
}
