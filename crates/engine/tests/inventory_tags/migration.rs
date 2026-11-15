use super::{BinaryStore, StoreError, TagCoverage, TestResult, coverage, snapshot, state, support, tags};
use sqlx::SqlitePool;

pub(super) async fn reconstruct_schema14(pool: &SqlitePool) -> TestResult {
    // Restore legacy14 before coverage, facets, work context and polymorphic references.
    let mut tx = pool.begin().await?;
    sqlx::query("DROP TABLE tag_observations").execute(&mut *tx).await?;
    sqlx::query("DROP TABLE resource_facets").execute(&mut *tx).await?;
    sqlx::query("ALTER TABLE work_queue DROP COLUMN context").execute(&mut *tx).await?;
    sqlx::query("ALTER TABLE unresolved_refs DROP COLUMN target_kinds").execute(&mut *tx).await?;
    sqlx::query("ALTER TABLE unresolved_refs DROP COLUMN resolution").execute(&mut *tx).await?;
    sqlx::query("PRAGMA user_version=14").execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

#[tokio::test]
async fn tags_upgrade_preserves_legacy_record_hash_timestamps_pairs_and_pending_revision() -> TestResult {
    // Given a valid prior schema with actual typed records/tags written by the public writer.
    let (directory, store, _) = support::fixture().await?;
    state::commit_resources(&store, "legacy", vec![support::observation("one", "legacy-record")?],
        vec![snapshot("one", TagCoverage::Complete, &[("empty", ""), ("owner", "alice")])?], 100).await?;
    let pending = store.drain_projection(100).await?;
    let pool = state::pool(&directory).await?;
    let image = state::resource_image(&pool).await?;
    drop(store);
    reconstruct_schema14(&pool).await?;
    let legacy_version: i64 = sqlx::query_scalar("PRAGMA user_version").fetch_one(&pool).await?;
    assert_eq!(legacy_version, 14);
    let absent: i64 = sqlx::query_scalar("SELECT count(*) FROM sqlite_schema WHERE name='tag_observations'")
        .fetch_one(&pool).await?;
    assert_eq!(absent, 0);
    pool.close().await;

    // When BinaryStore performs the 14→15 tag step and current schema16 additions on open.
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let pool = state::pool(&directory).await?;
    assert_eq!(sqlx::query_scalar::<_, i64>("PRAGMA user_version").fetch_one(&pool).await?, 16);
    // Then old pairs remain readable, but no freshness or complete-empty evidence is invented.
    assert_eq!(state::resource_image(&pool).await?, image);
    assert_eq!(store.drain_projection(100).await?, pending);
    assert_eq!(coverage(&directory).await?, vec![]);
    assert_eq!(tags(&directory).await?, vec![("us-east-1".into(), "one".into(), "empty".into(), "".into()),
        ("us-east-1".into(), "one".into(), "owner".into(), "alice".into())]);
    assert_eq!(state::read_tags(&store, &support::resource("one")?).await?,
        vec![state::pair("empty", ""), state::pair("owner", "alice")]);
    state::commit_tags(&store, "fresh-empty", vec![snapshot("one", TagCoverage::Complete, &[])?], 200).await?;
    assert_eq!(coverage(&directory).await?, vec![("one".into(), "complete".into(), 200, Some(200))]);
    assert_eq!(state::resource_image(&pool).await?, image);
    pool.close().await;
    drop(store);
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(state::read_tags(&store, &support::resource("one")?).await?, vec![]);
    assert_eq!(coverage(&directory).await?, vec![("one".into(), "complete".into(), 200, Some(200))]);
    let pool = state::pool(&directory).await?;
    assert_eq!(state::resource_image(&pool).await?, image);
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn tags_failed_upgrade_leaves_version_schema_and_inventory_unchanged() -> TestResult {
    // Given schema14 plus a facet namespace collision: tag coverage CREATE succeeds first,
    // then the next migration DDL fails inside the same transaction.
    let (directory, store, _) = support::fixture().await?;
    state::commit_resources(&store, "legacy", vec![support::observation("one", "legacy-record")?],
        vec![snapshot("one", TagCoverage::Complete, &[("owner", "alice")])?], 100).await?;
    drop(store);
    let pool = state::pool(&directory).await?;
    reconstruct_schema14(&pool).await?;
    sqlx::query("CREATE VIEW resource_facets AS SELECT 'owned migration fault' AS fault")
        .execute(&pool).await?;
    let before = state::database_state(&pool).await?;
    // When the second CREATE fails, even the already-created tag coverage table must roll back.
    let error = BinaryStore::open(&directory.path().join("inventory.sqlite")).await
        .err().ok_or("migration collision unexpectedly succeeded")?;
    assert!(matches!(&error, StoreError::Sqlite(sqlx::Error::Database(error))
        if error.message() == "view resource_facets already exists"));
    assert_eq!(state::database_state(&pool).await?, before);
    // Then removing only our fault permits the identical legacy inventory to upgrade and read.
    sqlx::query("DROP VIEW resource_facets").execute(&pool).await?;
    pool.close().await;
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(state::read_tags(&store, &support::resource("one")?).await?, vec![state::pair("owner", "alice")]);
    assert_eq!(coverage(&directory).await?, vec![]);
    Ok(())
}

#[tokio::test]
async fn schema15_upgrade_preserves_existing_coverage_instead_of_recreating_it() -> TestResult {
    // Given actual schema15 coverage, including historical complete evidence and latest failure.
    let (directory, store, _) = support::fixture().await?;
    state::commit_resources(&store, "complete", vec![support::observation("one", "same")?],
        vec![snapshot("one", TagCoverage::Complete, &[("owner", "alice")])?], 100).await?;
    state::commit_tags(&store, "degraded", vec![snapshot("one", TagCoverage::Failed, &[])?], 200).await?;
    let pending = store.drain_projection(100).await?;
    drop(store);
    let pool = state::pool(&directory).await?;
    let mut tx = pool.begin().await?;
    sqlx::query("DROP TABLE resource_facets").execute(&mut *tx).await?;
    sqlx::query("ALTER TABLE work_queue DROP COLUMN context").execute(&mut *tx).await?;
    sqlx::query("ALTER TABLE unresolved_refs DROP COLUMN target_kinds").execute(&mut *tx).await?;
    sqlx::query("ALTER TABLE unresolved_refs DROP COLUMN resolution").execute(&mut *tx).await?;
    sqlx::query("PRAGMA user_version=15").execute(&mut *tx).await?;
    tx.commit().await?;
    let image = state::resource_image(&pool).await?;
    pool.close().await;
    // When current migration adds facets, work context and polymorphic reference storage.
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    // Then prior pairs, pending work, and both coverage facts survive.
    assert_eq!(coverage(&directory).await?, vec![("one".into(), "failed".into(), 200, Some(100))]);
    assert_eq!(state::read_tags(&store, &support::resource("one")?).await?, vec![state::pair("owner", "alice")]);
    assert_eq!(store.drain_projection(100).await?, pending);
    let pool = state::pool(&directory).await?;
    assert_eq!(state::resource_image(&pool).await?, image);
    assert_eq!(sqlx::query_scalar::<_, i64>("PRAGMA user_version").fetch_one(&pool).await?, 16);
    let context: Vec<Vec<u8>> = sqlx::query_scalar("SELECT context FROM work_queue ORDER BY sweep_id")
        .fetch_all(&pool).await?;
    assert_eq!(context, vec![vec![], vec![]]);
    pool.close().await;
    Ok(())
}
