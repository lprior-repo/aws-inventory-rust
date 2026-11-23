use super::PgStore;
use crate::inventory::{ProjectionDelivery, StoreError};
use sqlx::{Postgres, Transaction};

pub struct PgDelivery {
    tx: Transaction<'static, Postgres>,
    generation: i64,
}

impl ProjectionDelivery for PgDelivery {
    fn generation(&self) -> i64 {
        self.generation
    }

    async fn release(self) -> Result<(), StoreError> {
        self.tx.commit().await?;
        Ok(())
    }
}

pub(super) async fn acquire(store: &PgStore) -> Result<PgDelivery, StoreError> {
    let mut tx = store.pool.begin().await?;
    let owned: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock(14,3)")
        .fetch_one(&mut *tx)
        .await?;
    if !owned {
        tx.rollback().await?;
        return Err(StoreError::DeliveryBusy);
    }
    let generation = sqlx::query_scalar("SELECT nextval('projection_delivery_generation_seq')")
        .fetch_one(&mut *tx)
        .await?;
    Ok(PgDelivery { tx, generation })
}
