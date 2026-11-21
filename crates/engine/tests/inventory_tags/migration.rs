use super::{
    BinaryStore, StoreError, TagCoverage, TestResult, coverage, snapshot, state, support, tags,
};
use sqlx::SqlitePool;

pub(super) async fn reconstruct_schema14(pool: &SqlitePool) -> TestResult {
    // Restore legacy14 before coverage, facets, work context and polymorphic references.
    let mut tx = pool.begin().await?;
    sqlx::query("DROP TABLE tag_observations")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DROP TABLE resource_facets")
        .execute(&mut *tx)
        .await?;
    sqlx::query("ALTER TABLE work_queue DROP COLUMN context")
        .execute(&mut *tx)
        .await?;
    sqlx::query("ALTER TABLE unresolved_refs DROP COLUMN target_kinds")
        .execute(&mut *tx)
        .await?;
    sqlx::query("ALTER TABLE unresolved_refs DROP COLUMN resolution")
        .execute(&mut *tx)
        .await?;
    sqlx::query("PRAGMA user_version=14")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

#[tokio::test]
async fn tags_upgrade_preserves_legacy_record_hash_timestamps_pairs_and_pending_revision()
-> TestResult {
    // Given a valid prior schema with actual typed records/tags written by the public writer.
    let (directory, store, _) = support::fixture().await?;
    state::commit_resources(
        &store,
        "legacy",
        vec![support::observation("one", "legacy-record")?],
        vec![snapshot(
            "one",
            TagCoverage::Complete,
            &[("empty", ""), ("owner", "alice")],
        )?],
        100,
    )
    .await?;
    let pending = store.drain_projection(100).await?;
    let pool = state::pool(&directory).await?;
    let image = state::resource_image(&pool).await?;
    drop(store);
    reconstruct_schema14(&pool).await?;
    let legacy_version: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(&pool)
        .await?;
    assert_eq!(legacy_version, 14);
    let absent: i64 =
        sqlx::query_scalar("SELECT count(*) FROM sqlite_schema WHERE name='tag_observations'")
            .fetch_one(&pool)
            .await?;
    assert_eq!(absent, 0);
    pool.close().await;

    // When BinaryStore performs the 14→17 migration on open.
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let pool = state::pool(&directory).await?;
    assert_eq!(
        sqlx::query_scalar::<_, i64>("PRAGMA user_version")
            .fetch_one(&pool)
            .await?,
        17
    );
    // Then old pairs remain readable, but no freshness or complete-empty evidence is invented.
    assert_eq!(state::resource_image(&pool).await?, image);
    assert_eq!(store.drain_projection(100).await?, pending);
    assert_eq!(coverage(&directory).await?, vec![]);
    assert_eq!(
        tags(&directory).await?,
        vec![
            ("us-east-1".into(), "one".into(), "empty".into(), "".into()),
            (
                "us-east-1".into(),
                "one".into(),
                "owner".into(),
                "alice".into()
            )
        ]
    );
    assert_eq!(
        state::read_tags(&store, &support::resource("one")?).await?,
        vec![state::pair("empty", ""), state::pair("owner", "alice")]
    );
    state::commit_tags(
        &store,
        "fresh-empty",
        vec![snapshot("one", TagCoverage::Complete, &[])?],
        200,
    )
    .await?;
    assert_eq!(
        coverage(&directory).await?,
        vec![("one".into(), "complete".into(), 200, Some(200))]
    );
    assert_eq!(state::resource_image(&pool).await?, image);
    pool.close().await;
    drop(store);
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(
        state::read_tags(&store, &support::resource("one")?).await?,
        vec![]
    );
    assert_eq!(
        coverage(&directory).await?,
        vec![("one".into(), "complete".into(), 200, Some(200))]
    );
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
    state::commit_resources(
        &store,
        "legacy",
        vec![support::observation("one", "legacy-record")?],
        vec![snapshot(
            "one",
            TagCoverage::Complete,
            &[("owner", "alice")],
        )?],
        100,
    )
    .await?;
    drop(store);
    let pool = state::pool(&directory).await?;
    reconstruct_schema14(&pool).await?;
    sqlx::query("CREATE VIEW resource_facets AS SELECT 'owned migration fault' AS fault")
        .execute(&pool)
        .await?;
    let before = state::database_state(&pool).await?;
    // When the second CREATE fails, even the already-created tag coverage table must roll back.
    let error = BinaryStore::open(&directory.path().join("inventory.sqlite"))
        .await
        .err()
        .ok_or("migration collision unexpectedly succeeded")?;
    assert!(
        matches!(&error, StoreError::Sqlite(sqlx::Error::Database(error))
        if error.message() == "view resource_facets already exists")
    );
    assert_eq!(state::database_state(&pool).await?, before);
    // Then removing only our fault permits the identical legacy inventory to upgrade and read.
    sqlx::query("DROP VIEW resource_facets")
        .execute(&pool)
        .await?;
    pool.close().await;
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(
        state::read_tags(&store, &support::resource("one")?).await?,
        vec![state::pair("owner", "alice")]
    );
    assert_eq!(coverage(&directory).await?, vec![]);
    Ok(())
}

#[tokio::test]
async fn schema15_upgrade_preserves_existing_coverage_instead_of_recreating_it() -> TestResult {
    // Given actual schema15 coverage, including historical complete evidence and latest failure.
    let (directory, store, _) = support::fixture().await?;
    state::commit_resources(
        &store,
        "complete",
        vec![support::observation("one", "same")?],
        vec![snapshot(
            "one",
            TagCoverage::Complete,
            &[("owner", "alice")],
        )?],
        100,
    )
    .await?;
    state::commit_tags(
        &store,
        "degraded",
        vec![snapshot("one", TagCoverage::Failed, &[])?],
        200,
    )
    .await?;
    let pending = store.drain_projection(100).await?;
    drop(store);
    let pool = state::pool(&directory).await?;
    let mut tx = pool.begin().await?;
    sqlx::query("DROP TABLE resource_facets")
        .execute(&mut *tx)
        .await?;
    sqlx::query("ALTER TABLE work_queue DROP COLUMN context")
        .execute(&mut *tx)
        .await?;
    sqlx::query("ALTER TABLE unresolved_refs DROP COLUMN target_kinds")
        .execute(&mut *tx)
        .await?;
    sqlx::query("ALTER TABLE unresolved_refs DROP COLUMN resolution")
        .execute(&mut *tx)
        .await?;
    sqlx::query("PRAGMA user_version=15")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    let image = state::resource_image(&pool).await?;
    pool.close().await;
    // When current migration adds facets, work context, polymorphic reference storage, and ES-10 normalization.
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    // Then prior pairs, pending work, and both coverage facts survive.
    assert_eq!(
        coverage(&directory).await?,
        vec![("one".into(), "failed".into(), 200, Some(100))]
    );
    assert_eq!(
        state::read_tags(&store, &support::resource("one")?).await?,
        vec![state::pair("owner", "alice")]
    );
    assert_eq!(store.drain_projection(100).await?, pending);
    let pool = state::pool(&directory).await?;
    assert_eq!(state::resource_image(&pool).await?, image);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("PRAGMA user_version")
            .fetch_one(&pool)
            .await?,
        17
    );
    let context: Vec<Vec<u8>> =
        sqlx::query_scalar("SELECT context FROM work_queue ORDER BY sweep_id")
            .fetch_all(&pool)
            .await?;
    assert_eq!(context, vec![Vec::<u8>::new(), Vec::new()]);
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn failed_upgrade_leaves_pre_provenance_schema14_unmutated_and_retry_completes() -> TestResult
{
    // Given schema14 without reviewed-reference provenance tables, as created before that DDL existed.
    let (directory, store, _) = support::fixture().await?;
    drop(store);
    let pool = state::pool(&directory).await?;
    reconstruct_schema14(&pool).await?;
    let mut tx = pool.begin().await?;
    sqlx::query("DROP TABLE reviewed_reference_evidence")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DROP TABLE reviewed_reference_grants")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    // And a migration fault that fails during the 14→17 transaction's facet DDL.
    sqlx::query("CREATE VIEW resource_facets AS SELECT 'owned migration fault' AS fault")
        .execute(&pool)
        .await?;
    let before = state::database_state(&pool).await?;
    // When open runs the migration, the failure must roll back every phase, provenance included.
    let error = BinaryStore::open(&directory.path().join("inventory.sqlite"))
        .await
        .err()
        .ok_or("migration fault unexpectedly succeeded")?;
    assert!(
        matches!(&error, StoreError::Sqlite(sqlx::Error::Database(error))
        if error.message() == "view resource_facets already exists")
    );
    // Then the old version and schema are unchanged.
    let after = state::database_state(&pool).await?;
    assert_eq!(after, before);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("PRAGMA user_version")
            .fetch_one(&pool)
            .await?,
        14
    );
    // And removing only the fault lets a retry complete the upgrade.
    sqlx::query("DROP VIEW resource_facets")
        .execute(&pool)
        .await?;
    pool.close().await;
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let pool = state::pool(&directory).await?;
    assert_eq!(
        sqlx::query_scalar::<_, i64>("PRAGMA user_version")
            .fetch_one(&pool)
            .await?,
        17
    );
    let provenance: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sqlite_schema WHERE name='reviewed_reference_grants'",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(provenance, 1);
    assert_eq!(
        state::read_tags(&store, &support::resource("one")?).await?,
        vec![]
    );
    pool.close().await;
    Ok(())
}
