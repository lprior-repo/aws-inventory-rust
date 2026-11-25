use super::{BinaryStore, StoreError, WorkLease, encode_scope};

impl BinaryStore {
    pub async fn release_lease(&self, lease: WorkLease) -> Result<(), StoreError> {
        let generation = lease
            .generation
            .checked_add(1)
            .ok_or(StoreError::LeaseGenerationOverflow)?;
        let _turn = self.writer_turn.lock().await;
        let result = sqlx::query(
            "UPDATE work_queue SET state=0,generation=?3,lease_until=0 WHERE sweep_id=?1 AND scope_key=?2 AND state=1 AND generation=?4 AND lease_until=?5",
        )
        .bind(lease.sweep.as_str())
        .bind(encode_scope(&lease.scope)?)
        .bind(generation)
        .bind(lease.generation)
        .bind(lease.lease_until_ms)
        .execute(&self.writer)
        .await?;
        if result.rows_affected() != 1 {
            return Err(StoreError::StaleLease);
        }
        Ok(())
    }
}
