use super::{DatabaseUrl, PgPool, PgStore, TestResult, run, single};
use inventory_engine::inventory::{InventoryStore, ProjectionAck};

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_full_outbox_when_enqueued_then_one_rebuild_marker_replaces_every_key() -> TestResult
{
    run(overflow).await
}

async fn overflow(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    sqlx::query("INSERT INTO inventory.projection_outbox(resource_key,revision,deleted,full_rebuild) SELECT int4send(n),nextval('inventory.projection_revision_seq'),false,false FROM generate_series(1,100000) n")
        .execute(pool).await?;
    assert_eq!(store.projection_state().await?.pending, 100_000);
    single(store, "overflow-first", "thing-1", "first").await?;
    let first = marker(store).await?;
    single(store, "overflow-second", "thing-1", "second").await?;
    let current = marker(store).await?;
    assert!(current > first);
    acknowledge(store, first, 0).await?;
    assert_eq!(marker(store).await?, current);
    acknowledge(store, current, 1).await?;
    assert_eq!(store.projection_state().await?.pending, 0);
    assert!(!store.projection_state().await?.rebuild_required);
    Ok(())
}

async fn marker(store: &PgStore) -> TestResult<i64> {
    let entries = store.drain_projection(1000).await?;
    assert_eq!(entries.len(), 1);
    let marker = entries.into_iter().next().ok_or("missing rebuild marker")?;
    assert!(marker.full_rebuild);
    assert!(!marker.deleted);
    assert_eq!(marker.key, None);
    assert!(store.projection_state().await?.rebuild_required);
    Ok(marker.revision)
}

async fn acknowledge(store: &PgStore, revision: i64, expected: u64) -> TestResult {
    let affected = store
        .ack_projection(ProjectionAck::full_rebuild(revision)?)
        .await?;
    assert_eq!(affected, expected);
    Ok(())
}
