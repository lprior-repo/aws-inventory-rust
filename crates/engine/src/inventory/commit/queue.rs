use super::*;

pub(super) async fn resolve_references(
    store: &BinaryStore,
    tx: &mut Transaction<'_, Sqlite>,
    target_key: &ResourceKey,
    observation: &Observation,
) -> Result<(), StoreError> {
    let schema = store
        .registry
        .schemas()
        .iter()
        .find(|schema| schema.kind() == &target_key.kind)
        .ok_or(StoreError::Invalid(
            "target kind is absent from schema registry",
        ))?;
    for column in schema
        .columns()
        .iter()
        .filter(|column| column.scalar() == ScalarType::Text)
    {
        let Some(value) = text_field(observation, column.path())? else {
            continue;
        };
        let scope = reference_scope(target_key.account.as_str(), target_key.region.as_str())?;
        let path = reference_path(column.path());
        let refs = unresolved_for(tx, &scope, target_key, &path, value).await?;
        let mut query = QueryBuilder::<Sqlite>::new("SELECT id FROM ");
        query
            .push(quote(schema.table().as_str()))
            .push(" WHERE account=")
            .push_bind(target_key.account.as_str())
            .push(" AND region=")
            .push_bind(target_key.region.as_str())
            .push(" AND ")
            .push(quote(column.name()))
            .push("=")
            .push_bind(value)
            .push(" LIMIT 2");
        let candidates = query.build().fetch_all(&mut **tx).await?;
        if candidates.len() > 1 {
            retract_ambiguous_edges(
                tx,
                &refs,
                ReferenceMatch {
                    table: schema.table().as_str(),
                    column: column.name(),
                    target: target_key,
                    value,
                },
            )
            .await?;
            continue;
        }
        let Some(candidate) = candidates.first() else {
            continue;
        };
        if candidate.try_get::<String, _>("id")? != target_key.id.as_str() {
            continue;
        }
        for reference in refs {
            resolve_one(tx, reference, target_key).await?;
        }
    }
    Ok(())
}
