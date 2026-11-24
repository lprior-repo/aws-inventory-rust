use crate::inventory::work::transitions::encode_scope;
use crate::inventory::{PageCommit, StoreError};
use sqlx::{Postgres, Transaction};

const ADVANCE: &str = "WITH inserted AS (INSERT INTO work_tokens(sweep_id,scope_key,digest) SELECT $1,$2,$3 WHERE $3::bytea IS NOT NULL ON CONFLICT DO NOTHING RETURNING 1), decision AS MATERIALIZED(SELECT $3::bytea IS NOT NULL AND NOT EXISTS(SELECT 1 FROM inserted) AS repeated), outcome AS (INSERT INTO scope_outcomes(sweep_id,scope_key,outcome,observed_at) SELECT $1,$2,CASE WHEN repeated THEN 'partial' ELSE 'complete' END,$6 FROM decision WHERE repeated OR $5 ON CONFLICT(sweep_id,scope_key) DO UPDATE SET outcome=EXCLUDED.outcome,observed_at=EXCLUDED.observed_at,detail=NULL), updated AS (UPDATE work_queue SET state=CASE WHEN repeated OR $5 THEN 2 WHEN $8::bigint IS NULL THEN 0 ELSE 1 END,token=CASE WHEN repeated OR $5 THEN NULL ELSE $4::bytea END,lease_until=CASE WHEN repeated OR $5 THEN 0 ELSE COALESCE($8::bigint,0) END FROM decision WHERE sweep_id=$1 AND scope_key=$2 AND generation=$7 RETURNING decision.repeated) SELECT repeated FROM updated";

/// With `keep_until`, a scope that has another page stays leased until then.
pub(super) async fn advance(
    tx: &mut Transaction<'_, Postgres>,
    page: &PageCommit,
    keep_until: Option<i64>,
) -> Result<bool, StoreError> {
    Ok(sqlx::query_scalar(ADVANCE)
        .bind(page.lease.sweep().as_str())
        .bind(encode_scope(page.lease.scope())?)
        .bind(
            page.continuation_digest
                .as_ref()
                .map(|digest| digest.as_slice()),
        )
        .bind(page.next_token.as_deref())
        .bind(page.done)
        .bind(page.observed_at_ms)
        .bind(page.lease.generation())
        .bind(keep_until)
        .fetch_one(&mut **tx)
        .await?)
}

pub(super) async fn children(
    tx: &mut Transaction<'_, Postgres>,
    page: &PageCommit,
) -> Result<(), StoreError> {
    if page.children.is_empty() {
        return Ok(());
    }
    let scopes = page
        .children
        .iter()
        .map(|child| encode_scope(child.scope()))
        .collect::<Result<Vec<_>, _>>()?;
    insert(tx, page, scopes).await
}

async fn insert(
    tx: &mut Transaction<'_, Postgres>,
    page: &PageCommit,
    scopes: Vec<Vec<u8>>,
) -> Result<(), StoreError> {
    let tokens: Vec<_> = page.children.iter().map(|c| c.token.as_deref()).collect();
    let bindings: Vec<_> = page
        .children
        .iter()
        .map(|c| c.bindings.as_slice())
        .collect();
    let contexts: Vec<_> = page.children.iter().map(|c| c.context.as_slice()).collect();
    sqlx::query("INSERT INTO work_queue(sweep_id,scope_key,token,bindings,context,state,generation,lease_until) SELECT $1,s,t,b,c,0,0,0 FROM UNNEST($2::bytea[],$3::bytea[],$4::bytea[],$5::bytea[]) u(s,t,b,c) ON CONFLICT DO NOTHING")
        .bind(page.lease.sweep().as_str())
        .bind(scopes)
        .bind(tokens)
        .bind(bindings)
        .bind(contexts)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
