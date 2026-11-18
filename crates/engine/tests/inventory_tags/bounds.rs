use super::{StoreError, SweepId, Tag, TagCoverage, TagObservation, TestResult, state, support};
use inventory_engine::inventory::ChangeStats;

fn numbered_snapshot(index: usize, pairs: usize) -> TestResult<TagObservation> {
    let tags = (0..pairs)
        .map(|number| Tag::new(format!("key-{number:03}"), "value"))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TagObservation::new(
        support::resource(&format!("target-{index:05}"))?,
        TagCoverage::Complete,
        tags,
    )?)
}

#[tokio::test]
async fn tags_page_snapshot_count_accepts_50000_and_rejects_50001_valid_empty_targets() -> TestResult
{
    // Given 50,001 distinct targets legitimately persisted by a public resource page.
    // Its acceptance also distinguishes the 100,000 resource bound from the 50,000 tag bound.
    let (directory, store, _) = support::fixture().await?;
    let observations = (0..50_001)
        .map(|index| support::observation(&format!("target-{index:05}"), "same"))
        .collect::<TestResult<Vec<_>>>()?;
    let created = state::commit_resources(&store, "targets", observations, vec![], 100).await?;
    assert_eq!(
        created.stats,
        ChangeStats {
            resources_inserted: 50_001,
            ..ChangeStats::default()
        }
    );
    let target = support::resource("target-00000")?;
    let lease = state::scoped_work(&store, support::child_work("boundary", &target)?, 200).await?;
    let mut page = support::page(lease, vec![], true, None, 200);
    page.tags = (0..50_000)
        .map(|index| numbered_snapshot(index, 0))
        .collect::<TestResult<Vec<_>>>()?;
    // When exactly 50,000 complete-empty snapshots (zero pairs) are committed.
    assert_eq!(store.commit_page(page).await?.stats, ChangeStats::default());
    let pool = state::pool(&directory).await?;
    let coverage: Vec<(String, String, i64, Option<i64>)> = sqlx::query_as(
        "SELECT id,coverage,observed_at,last_complete_at FROM tag_observations ORDER BY id",
    )
    .fetch_all(&pool)
    .await?;
    let expected = (0..50_000)
        .map(|index| {
            (
                format!("target-{index:05}"),
                "complete".into(),
                200,
                Some(200),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(coverage, expected);
    let counts: (i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM resources),(SELECT count(*) FROM thing_table),(SELECT count(*) FROM tags)",
    ).fetch_one(&pool).await?;
    assert_eq!(counts, (50_001, 50_001, 0));
    let before_image = state::resource_image(&pool).await?;
    let before_outbox = store.drain_projection(1000).await?;

    // Then 50,001 otherwise-valid empty snapshots fail this bound, not pair/target validation.
    let lease =
        state::scoped_work(&store, support::child_work("over-boundary", &target)?, 300).await?;
    let mut page = support::page(lease.clone(), vec![], true, None, 300);
    page.tags = (0..50_001)
        .map(|index| numbered_snapshot(index, 0))
        .collect::<TestResult<Vec<_>>>()?;
    assert!(matches!(
        store.commit_page(page).await,
        Err(StoreError::Invalid("page exceeds storage bounds"))
    ));
    assert_eq!(state::resource_image(&pool).await?, before_image);
    assert_eq!(store.drain_projection(1000).await?, before_outbox);
    let after: Vec<(String, String, i64, Option<i64>)> = sqlx::query_as(
        "SELECT id,coverage,observed_at,last_complete_at FROM tag_observations ORDER BY id",
    )
    .fetch_all(&pool)
    .await?;
    assert_eq!(after, expected);
    // The same unconsumed lease accepts the single previously unobserved legitimate target.
    let mut corrected = support::page(lease, vec![], true, None, 300);
    corrected.tags = vec![numbered_snapshot(50_000, 0)?];
    assert_eq!(
        store.commit_page(corrected).await?.stats,
        ChangeStats::default()
    );
    let last: (String, i64, Option<i64>) = sqlx::query_as(
        "SELECT coverage,observed_at,last_complete_at FROM tag_observations WHERE id='target-50000'",
    ).fetch_one(&pool).await?;
    assert_eq!(last, ("complete".into(), 300, Some(300)));
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn tags_page_pair_count_accepts_50000_and_rejects_50001_without_snapshot_count_mask()
-> TestResult {
    // Given 501 real targets, each snapshot individually within the 100-pair constructor bound.
    let (directory, store, _) = support::fixture().await?;
    let observations = (0..501)
        .map(|index| support::observation(&format!("target-{index:05}"), "same"))
        .collect::<TestResult<Vec<_>>>()?;
    state::commit_resources(&store, "targets", observations, vec![], 100).await?;
    let accepted = (0..500)
        .map(|index| numbered_snapshot(index, 100))
        .collect::<TestResult<Vec<_>>>()?;
    assert_eq!(
        state::commit_tags(&store, "exact-pairs", accepted, 200)
            .await?
            .stats,
        ChangeStats::default()
    );
    let pool = state::pool(&directory).await?;
    let counts: (i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM tags),(SELECT count(*) FROM tag_observations)",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(counts, (50_000, 500));
    let expected = (0..100)
        .map(|index| state::pair(&format!("key-{index:03}"), "value"))
        .collect::<Vec<_>>();
    assert_eq!(
        state::read_tags(&store, &support::resource("target-00000")?).await?,
        expected
    );
    assert_eq!(
        state::read_tags(&store, &support::resource("target-00499")?).await?,
        expected
    );
    let lease = state::scoped_work(
        &store,
        support::child_work("excess-pairs", &support::resource("target-00000")?)?,
        300,
    )
    .await?;
    let before = state::database_state(&pool).await?;
    let mut excessive = support::page(lease.clone(), vec![], true, None, 300);
    excessive.tags = (0..500)
        .map(|index| numbered_snapshot(index, 100))
        .chain(std::iter::once(numbered_snapshot(500, 1)))
        .collect::<TestResult<Vec<_>>>()?;
    // When only the aggregate pair count is exceeded, return its exact error atomically.
    assert!(matches!(
        store.commit_page(excessive).await,
        Err(StoreError::Invalid("page exceeds tag pair storage bound"))
    ));
    assert_eq!(state::database_state(&pool).await?, before);
    let mut corrected = support::page(lease, vec![], true, None, 300);
    corrected.tags = vec![numbered_snapshot(500, 1)?];
    assert_eq!(
        store.commit_page(corrected).await?.stats,
        ChangeStats::default()
    );
    assert_eq!(
        state::read_tags(&store, &support::resource("target-00500")?).await?,
        vec![state::pair("key-000", "value")]
    );
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn resource_observation_count_still_rejects_100001_independently_of_tags() -> TestResult {
    let (directory, store, _) = support::fixture().await?;
    store
        .seed_work(support::work("too-many-resources", "us-east-1")?)
        .await?;
    let lease = store
        .claim(&SweepId::new("too-many-resources".into())?, 99, 100)
        .await?
        .ok_or("lease missing")?;
    let pool = state::pool(&directory).await?;
    let before = state::database_state(&pool).await?;
    let observations = (0..100_001)
        .map(|_| support::observation("one", "value"))
        .collect::<Result<Vec<_>, _>>()?;
    assert!(matches!(
        store
            .commit_page(support::page(lease.clone(), observations, true, None, 100))
            .await,
        Err(StoreError::Invalid("page exceeds storage bounds"))
    ));
    assert_eq!(state::database_state(&pool).await?, before);
    assert_eq!(
        store
            .commit_page(support::page(
                lease,
                vec![support::observation("one", "value")?],
                true,
                None,
                100
            ))
            .await?
            .stats
            .resources_inserted,
        1
    );
    pool.close().await;
    Ok(())
}
