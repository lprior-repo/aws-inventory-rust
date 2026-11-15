#[path = "behavior/lifecycle.rs"]
mod lifecycle;
#[path = "behavior/projection.rs"]
mod projection;
use inventory_core::inventory::{
    AccountId, Edge, EdgeSource, OperationId, Region, Relation, ResourceId, ResourceKey, ScopeKey,
    ScopeOutcome, ServiceId,
};
use inventory_engine::inventory::{BinaryStore, StoreError, SweepId, WorkSpec};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use super::support::*;

#[tokio::test]
async fn given_a_late_unique_reference_when_target_becomes_ambiguous_then_edges_are_retracted()
-> TestResult {
    let (directory, store, _) = fixture().await?;
    store.seed_work(work("source", "us-east-1")?).await?;
    let source_lease = store
        .claim(&SweepId::new("source".to_owned())?, 100, 20)
        .await?
        .ok_or("missing source lease")?;
    let mut source_page = page(
        source_lease,
        vec![observation("source", "source-name")?],
        true,
        None,
        101,
    );
    source_page
        .unresolved
        .push(unresolved("source", "target-name")?);
    store.commit_page(source_page).await?;

    store.seed_work(work("target-one", "us-east-1")?).await?;
    let one = store
        .claim(&SweepId::new("target-one".to_owned())?, 200, 20)
        .await?
        .ok_or("missing first target lease")?;
    store
        .commit_page(page(
            one,
            vec![observation("target-one", "target-name")?],
            true,
            None,
            201,
        ))
        .await?;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    let unique_edges: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM edges WHERE from_id='source'")
        .fetch_one(&pool)
        .await?;
    assert_eq!(unique_edges, 1);

    store.seed_work(work("target-two", "us-east-1")?).await?;
    let two = store
        .claim(&SweepId::new("target-two".to_owned())?, 300, 20)
        .await?
        .ok_or("missing second target lease")?;
    store
        .commit_page(page(
            two,
            vec![observation("target-two", "target-name")?],
            true,
            None,
            301,
        ))
        .await?;
    let ambiguous_edges: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM edges WHERE from_id='source'")
            .fetch_one(&pool)
            .await?;
    pool.close().await;
    assert_eq!(ambiguous_edges, 0);
    Ok(())
}

#[tokio::test]
async fn given_two_regions_when_work_is_seeded_then_scope_identity_is_preserved() -> TestResult {
    let (_directory, store, _) = fixture().await?;
    store.seed_work(work("sweep-a", "us-east-1")?).await?;
    store.seed_work(work("sweep-a", "us-west-2")?).await?;
    let denied_scope = ScopeKey::new(
        AccountId::parse("123456789012")?,
        Region::parse("us-east-1")?,
        ServiceId::parse("example")?,
        OperationId::parse("DeleteThing")?,
        None,
    )?;
    let denied = WorkSpec::new(
        SweepId::new("sweep-a".to_owned())?,
        denied_scope,
        None,
        Vec::new(),
    )?;
    assert!(store.seed_work(denied).await.is_err());

    let first = store
        .claim(&SweepId::new("sweep-a".to_owned())?, 100, 20)
        .await?
        .ok_or("missing first lease")?;
    let second = store
        .claim(&SweepId::new("sweep-a".to_owned())?, 100, 20)
        .await?
        .ok_or("missing second lease")?;

    assert_eq!(first.scope().region().as_str(), "us-east-1");
    assert_eq!(second.scope().region().as_str(), "us-west-2");
    Ok(())
}

#[tokio::test]
async fn given_an_expired_lease_when_reclaimed_then_old_commit_is_fenced_and_cursor_resumes()
-> TestResult {
    let (_directory, store, _) = fixture().await?;
    store.seed_work(work("resume-id", "us-east-1")?).await?;
    let sweep = SweepId::new("resume-id".to_owned())?;
    let stale = store
        .claim(&sweep, 100, 10)
        .await?
        .ok_or("missing first lease")?;
    let current = store
        .claim(&sweep, 110, 10)
        .await?
        .ok_or("missing reclaimed lease")?;

    let stale_commit = page(stale, Vec::new(), true, None, 110);
    assert!(matches!(
        store.commit_page(stale_commit).await,
        Err(StoreError::StaleLease)
    ));
    store
        .commit_page(page(current, Vec::new(), false, Some(vec![9, 8, 7]), 111))
        .await?;
    let resumed = store
        .claim(&sweep, 112, 10)
        .await?
        .ok_or("missing resumed lease")?;

    assert_eq!(resumed.token(), Some([9, 8, 7].as_slice()));
    assert_eq!(store.resume_sweep().await?, Some(sweep));
    Ok(())
}

#[tokio::test]
async fn given_an_active_sweep_when_resumed_then_lease_is_fenced_and_work_is_immediately_claimable()
-> TestResult {
    let (_directory, store, _) = fixture().await?;
    store.seed_work(work("takeover", "us-east-1")?).await?;
    let sweep = SweepId::new("takeover".to_owned())?;
    let old = store
        .claim(&sweep, 100, 100)
        .await?
        .ok_or("missing original lease")?;

    assert_eq!(store.resume_sweep().await?, Some(sweep.clone()));
    let current = store
        .claim(&sweep, 101, 100)
        .await?
        .ok_or("resume did not release original lease")?;
    assert!(matches!(
        store
            .commit_page(page(old, Vec::new(), true, None, 102))
            .await,
        Err(StoreError::StaleLease)
    ));
    store
        .commit_page(page(current, Vec::new(), true, None, 103))
        .await?;

    assert_eq!(store.sweep_summary(&sweep).await?.complete_scopes, 1);
    Ok(())
}

#[tokio::test]
async fn given_a_repeated_token_cycle_when_committed_then_page_data_and_children_persist_as_partial()
-> TestResult {
    let (directory, store, _) = fixture().await?;
    store.seed_work(work("baseline", "us-east-1")?).await?;
    let baseline = store
        .claim(&SweepId::new("baseline".to_owned())?, 90, 50)
        .await?
        .ok_or("missing baseline lease")?;
    store
        .commit_page(page(
            baseline,
            vec![observation("unseen", "baseline")?],
            true,
            None,
            91,
        ))
        .await?;
    let sweep = SweepId::new("cycle".to_owned())?;
    store.seed_work(work("cycle", "us-east-1")?).await?;
    let first = store
        .claim(&sweep, 100, 50)
        .await?
        .ok_or("missing initial cycle lease")?;
    store
        .commit_page(page(first, Vec::new(), false, Some(vec![b'B']), 101))
        .await?;
    let second = store
        .claim(&sweep, 102, 50)
        .await?
        .ok_or("missing second cycle lease")?;
    store
        .commit_page(page(second, Vec::new(), false, Some(vec![b'A']), 103))
        .await?;
    let third = store
        .claim(&sweep, 104, 50)
        .await?
        .ok_or("missing third cycle lease")?;
    let mut repeated = page(
        third,
        vec![observation("cycle-resource", "persisted")?],
        false,
        Some(vec![b'B']),
        105,
    );
    repeated
        .children
        .push(child_work("cycle", &resource("parent")?)?);
    let result = store.commit_page(repeated).await?;

    assert!(result.repeated_token);
    assert_eq!(result.next_generation, None);
    assert_eq!(store.sweep_summary(&sweep).await?.partial_scopes, 1);
    assert_eq!(store.sweep_summary(&sweep).await?.work_pending, 1);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    let misses: i64 =
        sqlx::query_scalar("SELECT missing_complete FROM resource_scopes WHERE id='unseen'")
            .fetch_one(&pool)
            .await?;
    pool.close().await;
    assert_eq!(misses, 0);
    assert!(
        store
            .projection_resource(&resource("cycle-resource")?)
            .await?
            .is_some()
    );
    Ok(())
}
