use super::{
    BinaryStore, ProjectionAck, TagCoverage, TestResult, acknowledge, coverage, snapshot, state,
    support,
};
use inventory_engine::inventory::{ChangeStats, PendingProjection};

#[tokio::test]
async fn tags_only_changes_survive_reopen_and_stale_ack_while_newer_revision_is_pending()
-> TestResult {
    let (directory, store, _) = support::fixture().await?;
    state::commit_resources(
        &store,
        "baseline",
        vec![support::observation("one", "same")?],
        vec![snapshot(
            "one",
            TagCoverage::Complete,
            &[("owner", "initial")],
        )?],
        100,
    )
    .await?;
    acknowledge(&store).await?;
    let pool = state::pool(&directory).await?;
    let image = state::resource_image(&pool).await?;
    assert_eq!(
        state::commit_tags(
            &store,
            "revision-r",
            vec![snapshot(
                "one",
                TagCoverage::Complete,
                &[("owner", "replacement"), ("team", "alice")]
            )?],
            200
        )
        .await?
        .stats,
        ChangeStats::default()
    );
    let r = store.drain_projection(100).await?;
    assert_eq!(r.len(), 1);
    let r = r.first().ok_or("revision R missing")?.clone();
    let key = support::resource("one")?;
    assert_eq!(r.key, Some(key.clone()));
    let mut retained = store.begin_projection_snapshot().await?;
    assert_eq!(
        retained.tags_for(&key, None, 1000).await?.tags,
        vec![
            state::pair("owner", "replacement"),
            state::pair("team", "alice")
        ]
    );

    assert_eq!(
        state::commit_tags(
            &store,
            "revision-s",
            vec![snapshot("one", TagCoverage::Complete, &[])?],
            300
        )
        .await?
        .stats,
        ChangeStats::default()
    );
    let s = store.drain_projection(100).await?;
    assert_eq!(s.len(), 1);
    let s = s.first().ok_or("revision S missing")?.clone();
    assert!(s.revision > r.revision);
    assert_eq!(
        s,
        PendingProjection {
            key: Some(key.clone()),
            revision: s.revision,
            deleted: false,
            full_rebuild: false
        }
    );
    assert_eq!(
        retained.tags_for(&key, None, 1000).await?.tags,
        vec![
            state::pair("owner", "replacement"),
            state::pair("team", "alice")
        ]
    );
    retained.finish().await?;
    assert_eq!(state::resource_image(&pool).await?, image);
    pool.close().await;
    drop(store);

    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let stale = ProjectionAck::new(key.clone(), r.revision)?;
    store.ack_projection(stale.clone()).await?;
    assert_eq!(store.drain_projection(100).await?, vec![s.clone()]);
    assert_eq!(state::read_tags(&store, &key).await?, vec![]);
    assert_eq!(
        coverage(&directory).await?,
        vec![("one".into(), "complete".into(), 300, Some(300))]
    );
    store
        .ack_projection(ProjectionAck::new(key.clone(), s.revision)?)
        .await?;
    store.ack_projection(stale).await?;
    assert_eq!(store.drain_projection(100).await?, vec![]);

    let result = state::commit_resources(
        &store,
        "same-record-new-tags",
        vec![support::observation("one", "same")?],
        vec![snapshot(
            "one",
            TagCoverage::Complete,
            &[("owner", "final")],
        )?],
        400,
    )
    .await?;
    assert_eq!(result.stats, ChangeStats::default());
    let pool = state::pool(&directory).await?;
    assert_eq!(state::resource_image(&pool).await?, image);
    assert_eq!(
        state::read_tags(&store, &key).await?,
        vec![state::pair("owner", "final")]
    );
    let final_pending = store.drain_projection(100).await?;
    assert_eq!(final_pending.len(), 1);
    assert!(
        final_pending
            .first()
            .ok_or("final revision missing")?
            .revision
            > s.revision
    );
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn absent_and_every_degraded_observation_retain_last_complete_and_known_pairs() -> TestResult
{
    let (directory, store, _) = support::fixture().await?;
    state::commit_resources(
        &store,
        "complete",
        vec![support::observation("one", "original")?],
        vec![snapshot(
            "one",
            TagCoverage::Complete,
            &[("empty", ""), ("owner", "alice")],
        )?],
        100,
    )
    .await?;
    let key = support::resource("one")?;
    let expected = vec![state::pair("empty", ""), state::pair("owner", "alice")];
    for (name, status, at) in [
        ("partial", TagCoverage::Partial, 200),
        ("failed", TagCoverage::Failed, 300),
        ("unobserved", TagCoverage::Unobserved, 400),
    ] {
        state::commit_resources(
            &store,
            name,
            vec![support::observation("one", name)?],
            vec![snapshot("one", status, &[("ignored", "not-complete")])?],
            at,
        )
        .await?;
        assert_eq!(
            coverage(&directory).await?,
            vec![("one".into(), name.into(), at, Some(100))]
        );
        assert_eq!(state::read_tags(&store, &key).await?, expected);
    }
    let mut retained = store.begin_projection_snapshot().await?;
    assert_eq!(retained.tags_for(&key, None, 1000).await?.tags, expected);
    state::commit_resources(
        &store,
        "absent",
        vec![support::observation("one", "metadata-only")?],
        vec![],
        500,
    )
    .await?;
    assert_eq!(
        coverage(&directory).await?,
        vec![("one".into(), "unobserved".into(), 400, Some(100))]
    );
    assert_eq!(state::read_tags(&store, &key).await?, expected);
    assert_eq!(retained.tags_for(&key, None, 1000).await?.tags, expected);
    retained.finish().await?;
    let pool = state::pool(&directory).await?;
    let record: Vec<u8> = sqlx::query_scalar("SELECT record FROM thing_table WHERE id='one'")
        .fetch_one(&pool)
        .await?;
    assert_eq!(
        record.as_slice(),
        support::observation("one", "metadata-only")?
            .record()
            .as_bytes()
    );
    pool.close().await;
    drop(store);
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(
        coverage(&directory).await?,
        vec![("one".into(), "unobserved".into(), 400, Some(100))]
    );
    assert_eq!(state::read_tags(&store, &key).await?, expected);
    state::commit_tags(
        &store,
        "complete-empty",
        vec![snapshot("one", TagCoverage::Complete, &[])?],
        600,
    )
    .await?;
    assert_eq!(
        coverage(&directory).await?,
        vec![("one".into(), "complete".into(), 600, Some(600))]
    );
    assert_eq!(state::read_tags(&store, &key).await?, vec![]);
    Ok(())
}
