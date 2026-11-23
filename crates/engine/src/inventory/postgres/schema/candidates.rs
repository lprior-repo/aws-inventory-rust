use crate::inventory::StoreError;
use sqlx::{Postgres, Transaction};

pub(super) async fn create(tx: &mut Transaction<'_, Postgres>) -> Result<(), StoreError> {
    sqlx::query("CREATE TABLE inventory.unresolved_ref_candidates(from_account text NOT NULL,from_region text NOT NULL,from_kind text NOT NULL,from_id text NOT NULL,target_scope bytea NOT NULL,target_kind text NOT NULL,target_path bytea NOT NULL,target_value text NOT NULL,relation text NOT NULL,source text NOT NULL,candidate_kind text NOT NULL,PRIMARY KEY(from_account,from_region,from_kind,from_id,target_scope,target_kind,target_path,target_value,relation,source,candidate_kind),FOREIGN KEY(from_account,from_region,from_kind,from_id,target_scope,target_kind,target_path,target_value,relation,source) REFERENCES inventory.unresolved_refs(from_account,from_region,from_kind,from_id,target_scope,target_kind,target_path,target_value,relation,source) ON DELETE CASCADE)")
        .execute(&mut **tx).await?;
    sqlx::query("CREATE INDEX unresolved_ref_candidates_target ON inventory.unresolved_ref_candidates(target_scope,candidate_kind)")
        .execute(&mut **tx).await?;
    Ok(())
}
