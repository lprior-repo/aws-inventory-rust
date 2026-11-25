use crate::inventory::work::ownership_scope_bytes;
use crate::inventory::{PageCommit, StoreError};
use sqlx::{Postgres, Transaction};

pub(super) async fn persist(
    tx: &mut Transaction<'_, Postgres>,
    page: &PageCommit,
) -> Result<(), StoreError> {
    let Some(first) = page
        .children
        .iter()
        .position(|child| child.scope_member.is_some())
    else {
        return Ok(());
    };
    let source = page.lease.scope();
    let source_key = ownership_scope_bytes(source, page.lease.bindings())?;

    for child in page.children.iter().skip(first) {
        let Some(member) = &child.scope_member else {
            continue;
        };
        let child_key = ownership_scope_bytes(child.scope(), &child.bindings)?;
        sqlx::query(
            "INSERT INTO inventory.scope_members(account,region,scope_key,child_scope,member_id,missing_complete,observed_sweep) \
             VALUES($1,$2,$3,$4,$5,0,$6) \
             ON CONFLICT(account,region,scope_key,child_scope,member_id) DO UPDATE SET \
             missing_complete=0,observed_sweep=excluded.observed_sweep",
        )
        .bind(source.account().as_str())
        .bind(source.region().as_str())
        .bind(&source_key)
        .bind(&child_key)
        .bind(member.as_str())
        .bind(page.lease.sweep().as_str())
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}
