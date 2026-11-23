use super::*;

pub(super) fn validate(
    store: &PgStore,
    works: &[WorkSpec],
    sweep: &SweepId,
) -> Result<(), StoreError> {
    for work in works {
        work.validate_context()?;
        if work.sweep() != sweep {
            return Err(StoreError::Invalid("batch work items must share one sweep"));
        }
        if !store.registry.allows(work.scope()) {
            return Err(StoreError::Invalid(
                "work scope is not in immutable catalog",
            ));
        }
    }
    Ok(())
}

pub(super) async fn insert(
    tx: &mut Transaction<'_, Postgres>,
    sweep: &SweepId,
    works: &[WorkSpec],
) -> Result<(), StoreError> {
    let scopes = works
        .iter()
        .map(|work| encode_scope(work.scope()))
        .collect::<Result<Vec<_>, _>>()?;
    let tokens: Vec<_> = works.iter().map(|work| work.token.as_deref()).collect();
    let bindings: Vec<_> = works.iter().map(|work| work.bindings.as_slice()).collect();
    let contexts: Vec<_> = works.iter().map(|work| work.context.as_slice()).collect();
    sqlx::query("INSERT INTO work_queue(sweep_id,scope_key,token,bindings,context,state,generation,lease_until,last_complete) SELECT $1,k,t,b,c,0,0,0,COALESCE((SELECT MAX(observed_at) FROM scope_outcomes WHERE scope_key=k AND outcome='complete'),-1) FROM UNNEST($2::bytea[],$3::bytea[],$4::bytea[],$5::bytea[]) AS u(k,t,b,c) ON CONFLICT DO NOTHING")
        .bind(sweep.as_str()).bind(&scopes).bind(&tokens).bind(&bindings).bind(&contexts)
        .execute(&mut **tx).await?;
    Ok(())
}
