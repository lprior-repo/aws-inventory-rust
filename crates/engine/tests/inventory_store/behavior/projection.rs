use super::*;

#[tokio::test]
pub(super) async fn given_a_final_page_when_committed_then_complete_outcome_is_atomic_and_idempotent()
-> TestResult {
    let (_directory, store, _) = fixture().await?;
    store.seed_work(work("finalize", "us-east-1")?).await?;
    let sweep = SweepId::new("finalize".to_owned())?;
    let lease = store.claim(&sweep, 100, 30).await?.ok_or("missing lease")?;
    let result = store
        .commit_page(page(
            lease,
            vec![observation("thing-1", "alpha")?],
            true,
            None,
            101,
        ))
        .await?;

    assert_eq!(result.stats.resources_inserted, 1);
    assert_eq!(store.resume_sweep().await?, None);
    assert_eq!(
        store
            .projection_resource(&resource("thing-1")?)
            .await?
            .map(|item| item.name),
        Some(None)
    );
    Ok(())
}

#[tokio::test]
pub(super) async fn given_a_partial_scope_and_two_complete_misses_then_only_the_second_complete_miss_deletes()
-> TestResult {
    let (directory, store, _) = fixture().await?;
    store.seed_work(work("seen", "us-east-1")?).await?;
    let seen = store
        .claim(&SweepId::new("seen".to_owned())?, 100, 20)
        .await?
        .ok_or("missing lease")?;
    store
        .commit_page(page(
            seen,
            vec![observation("thing-1", "alpha")?],
            true,
            None,
            101,
        ))
        .await?;

    store.seed_work(work("partial", "us-east-1")?).await?;
    let partial = store
        .claim(&SweepId::new("partial".to_owned())?, 200, 20)
        .await?
        .ok_or("missing partial lease")?;
    store
        .finish_scope(partial, ScopeOutcome::Inaccessible, 201)
        .await?;
    assert!(
        store
            .projection_resource(&resource("thing-1")?)
            .await?
            .is_some()
    );

    for sweep in ["complete-one", "complete-two"] {
        store.seed_work(work(sweep, "us-east-1")?).await?;
        let lease = store
            .claim(&SweepId::new(sweep.to_owned())?, 300, 20)
            .await?
            .ok_or("missing complete lease")?;
        store
            .commit_page(page(lease, Vec::new(), true, None, 301))
            .await?;
        let remains = store
            .projection_resource(&resource("thing-1")?)
            .await?
            .is_some();
        assert_eq!(remains, sweep == "complete-one");
    }
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    let typed_rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM thing_table")
        .fetch_one(&pool)
        .await?;
    pool.close().await;
    assert_eq!(typed_rows, 0);
    Ok(())
}
