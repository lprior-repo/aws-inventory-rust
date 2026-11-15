use super::{StoreError, SweepId, TagCoverage, TestResult, acknowledge, snapshot, state, support};
use inventory_core::inventory::{AccountId, Region};
use inventory_engine::inventory::ChangeStats;

#[tokio::test]
async fn tags_invalid_member_rolls_back_entire_page_and_allows_same_lease_retry() -> TestResult {
    // Given a literal baseline and a live lease whose cursor/children have not advanced.
    let (directory, store, _) = support::fixture().await?;
    state::commit_resources(&store, "baseline", vec![support::observation("one", "original")?],
        vec![snapshot("one", TagCoverage::Complete, &[("safe", "original")])?], 100).await?;
    acknowledge(&store).await?;
    store.seed_work(support::work("invalid", "us-east-1")?).await?;
    let lease = store.claim(&SweepId::new("invalid".to_owned())?, 199, 100).await?.ok_or("lease missing")?;
    let pool = state::pool(&directory).await?;
    let before = state::database_state(&pool).await?;

    // When tag validation fails after a valid resource/tag prefix was already persisted.
    let mut invalid = support::page(lease.clone(), vec![support::observation("one", "changed")?],
        false, Some(b"next-page".to_vec()), 200);
    invalid.children = vec![support::child_work("invalid", &support::resource("one")?)?];
    invalid.tags = vec![snapshot("one", TagCoverage::Complete, &[("new", "changed")])?,
        snapshot("missing", TagCoverage::Complete, &[])?];
    assert!(matches!(store.commit_page(invalid).await,
        Err(StoreError::Invalid("tag observation has no persisted resource"))));
    // Then every persisted table, coverage, cursor, generation, lease and metadata is unchanged.
    assert_eq!(state::database_state(&pool).await?, before);

    // Early validation failures are independently tested against otherwise legitimate targets.
    for foreign in [
        { let mut key = support::resource("one")?; key.account = AccountId::parse("000000000000")?; key },
        { let mut key = support::resource("one")?; key.region = Region::parse("global")?; key },
    ] {
        let mut page = support::page(lease.clone(), vec![], true, None, 200);
        page.tags = vec![inventory_core::inventory::TagObservation::new(foreign, TagCoverage::Complete, vec![])?];
        assert!(matches!(store.commit_page(page).await,
            Err(StoreError::Invalid("page contains data outside its leased account and region"))));
        assert_eq!(state::database_state(&pool).await?, before);
    }
    let mut duplicate = support::page(lease.clone(), vec![], true, None, 200);
    duplicate.tags = vec![snapshot("one", TagCoverage::Complete, &[])?, snapshot("one", TagCoverage::Complete, &[])?];
    assert!(matches!(store.commit_page(duplicate).await,
        Err(StoreError::Invalid("page contains duplicate resource tag observations"))));
    assert_eq!(state::database_state(&pool).await?, before);

    // A corrected nonterminal page must still be accepted with precisely that lease.
    let mut corrected = support::page(lease.clone(), vec![support::observation("one", "changed")?],
        false, Some(b"next-page".to_vec()), 200);
    corrected.children = vec![support::child_work("invalid", &support::resource("one")?)?];
    corrected.tags = vec![snapshot("one", TagCoverage::Complete, &[("new", "changed")])?];
    let result = store.commit_page(corrected).await?;
    assert_eq!(result.stats, ChangeStats { resources_changed: 1, ..ChangeStats::default() });
    assert_eq!(result.repeated_token, false);
    assert_eq!(state::read_tags(&store, &support::resource("one")?).await?, vec![state::pair("new", "changed")]);
    let queue: Vec<(Option<Vec<u8>>, i64, i64)> = sqlx::query_as(
        "SELECT token,state,lease_until FROM work_queue WHERE sweep_id='invalid' ORDER BY token IS NOT NULL",
    ).fetch_all(&pool).await?;
    assert_eq!(queue, vec![(None, 0, 0), (Some(b"next-page".to_vec()), 0, 0)]);
    let token_count: i64 = sqlx::query_scalar("SELECT count(*) FROM work_tokens WHERE sweep_id='invalid'")
        .fetch_one(&pool).await?;
    assert_eq!(token_count, 1);
    assert!(matches!(store.commit_page(support::page(lease, vec![], true, None, 200)).await,
        Err(StoreError::StaleLease)));
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn tags_late_checkpoint_failure_rolls_back_children_cursor_coverage_and_outbox() -> TestResult {
    // Given a deterministic SQLite fault after the final lease checkpoint UPDATE.
    let (directory, store, _) = support::fixture().await?;
    state::commit_resources(&store, "baseline", vec![support::observation("one", "original")?],
        vec![snapshot("one", TagCoverage::Complete, &[("safe", "original")])?], 100).await?;
    acknowledge(&store).await?;
    store.seed_work(support::work("checkpoint", "us-east-1")?).await?;
    let lease = store.claim(&SweepId::new("checkpoint".to_owned())?, 199, 100).await?.ok_or("lease missing")?;
    let pool = state::pool(&directory).await?;
    sqlx::query("CREATE TRIGGER reject_checkpoint AFTER UPDATE ON work_queue WHEN OLD.state=1 AND NEW.state=0 BEGIN SELECT RAISE(ABORT,'owned checkpoint fault'); END")
        .execute(&pool).await?;
    let before = state::database_state(&pool).await?;
    let page = || -> TestResult<_> {
        let mut page = support::page(lease.clone(), vec![support::observation("one", "changed")?],
            false, Some(b"next-page".to_vec()), 200);
        page.tags = vec![snapshot("one", TagCoverage::Complete, &[("new", "changed")])?];
        page.children = vec![support::child_work("checkpoint", &support::resource("one")?)?];
        Ok(page)
    };
    // When resources, tags, coverage, outbox, child and token writes precede that fault.
    let error = store.commit_page(page()?).await.err().ok_or("checkpoint fault was not returned")?;
    assert!(matches!(&error, StoreError::Sqlite(sqlx::Error::Database(error))
        if error.message() == "owned checkpoint fault"));
    // Then the entire transaction, including the checkpoint itself, is rolled back.
    assert_eq!(state::database_state(&pool).await?, before);
    sqlx::query("DROP TRIGGER reject_checkpoint").execute(&pool).await?;
    let result = store.commit_page(page()?).await?;
    assert_eq!(result.stats.resources_changed, 1);
    assert_eq!(result.repeated_token, false);
    assert_eq!(state::read_tags(&store, &support::resource("one")?).await?, vec![state::pair("new", "changed")]);
    pool.close().await;
    Ok(())
}
