#[path = "inventory_store/support.rs"]
mod support;

#[path = "inventory_tags/state.rs"]
mod state;
#[path = "inventory_tags/bounds.rs"]
mod bounds;
#[path = "inventory_tags/lifecycle.rs"]
mod lifecycle;
#[path = "inventory_tags/atomic.rs"]
mod atomic;
#[path = "inventory_tags/scopes.rs"]
mod scopes;
#[path = "inventory_tags/migration.rs"]
mod migration;
#[path = "inventory_tags/constructors.rs"]
mod constructors;

use inventory_core::inventory::{InventoryError, Region, Tag, TagCoverage, TagObservation};
use inventory_engine::inventory::{BinaryStore, ProjectionAck, StoreError, SweepId};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use support::TestResult;

fn snapshot(id: &str, coverage: TagCoverage, pairs: &[(&str, &str)]) -> TestResult<TagObservation> {
    Ok(TagObservation::new(
        support::resource(id)?, coverage,
        pairs.iter().map(|(key, value)| Tag::new(*key, *value)).collect::<Result<Vec<_>, _>>()?,
    )?)
}

async fn commit(
    store: &BinaryStore, sweep: &str, value: &str, tags: Vec<TagObservation>,
) -> TestResult {
    commit_at(store, sweep, value, tags, 11).await
}

async fn commit_at(
    store: &BinaryStore, sweep: &str, value: &str, tags: Vec<TagObservation>, at: i64,
) -> TestResult {
    store.seed_work(support::work(sweep, "us-east-1")?).await?;
    let lease = store.claim(&SweepId::new(sweep.to_owned())?, at.saturating_sub(1), 100).await?
        .ok_or("missing lease")?;
    let mut page = support::page(lease, vec![support::observation("one", value)?, support::observation("two", "stable")?], true, None, at);
    page.tags = tags;
    store.commit_page(page).await?;
    Ok(())
}

async fn tags(directory: &tempfile::TempDir) -> TestResult<Vec<(String, String, String, String)>> {
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(
        SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")).read_only(true),
    ).await?;
    let rows = sqlx::query_as("SELECT region,id,tag_key,tag_value FROM tags ORDER BY region,id,tag_key")
        .fetch_all(&pool).await?;
    pool.close().await;
    Ok(rows)
}

async fn acknowledge(store: &BinaryStore) -> TestResult {
    for pending in store.drain_projection(100).await? {
        let ack = match pending.key {
            Some(key) => ProjectionAck::new(key, pending.revision)?,
            None => ProjectionAck::full_rebuild(pending.revision)?,
        };
        store.ack_projection(ack).await?;
    }
    Ok(())
}

#[tokio::test]
async fn given_complete_tags_when_only_tags_change_then_refresh_and_reopen_preserve_exact_replacement() -> TestResult {
    let (directory, store, _) = support::fixture().await?;
    commit(&store, "initial", "same", vec![snapshot("one", TagCoverage::Complete, &[("old", "gone"), ("keep", "v")])?, snapshot("two", TagCoverage::Complete, &[("other", "safe")])?]).await?;
    let before = store.projection_resource(&support::resource("one")?).await?.ok_or("missing resource")?;
    acknowledge(&store).await?;
    commit(&store, "replacement", "same", vec![snapshot("one", TagCoverage::Complete, &[("keep", "new"), ("empty", "")])?]).await?;
    let after = store.projection_resource(&support::resource("one")?).await?.ok_or("missing resource")?;
    assert_eq!(before.content_hash, after.content_hash);
    let pending = store.drain_projection(100).await?;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending.first().and_then(|item| item.key.as_ref()), Some(&support::resource("one")?));
    assert_eq!(tags(&directory).await?, vec![("us-east-1".into(), "one".into(), "empty".into(), "".into()), ("us-east-1".into(), "one".into(), "keep".into(), "new".into()), ("us-east-1".into(), "two".into(), "other".into(), "safe".into())]);
    drop(store);
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    acknowledge(&store).await?;
    commit(&store, "remove", "same", vec![snapshot("one", TagCoverage::Complete, &[])?]).await?;
    assert_eq!(store.drain_projection(100).await?.len(), 1);
    assert_eq!(tags(&directory).await?, vec![("us-east-1".into(), "two".into(), "other".into(), "safe".into())]);
    acknowledge(&store).await?;
    commit(&store, "same-empty", "same", vec![snapshot("one", TagCoverage::Complete, &[])?]).await?;
    assert!(store.drain_projection(100).await?.is_empty());
    Ok(())
}

#[tokio::test]
async fn given_known_tags_when_resource_changes_without_complete_tags_then_known_tags_survive() -> TestResult {
    let (directory, store, _) = support::fixture().await?;
    commit(&store, "initial", "a", vec![snapshot("one", TagCoverage::Complete, &[("known", "value")])?]).await?;
    let expected = tags(&directory).await?;
    for (sweep, coverage) in [("partial", TagCoverage::Partial), ("failed", TagCoverage::Failed), ("unobserved", TagCoverage::Unobserved)] {
        commit(&store, sweep, sweep, vec![snapshot("one", coverage, &[("not-authoritative", "ignored")])?]).await?;
        assert_eq!(tags(&directory).await?, expected);
    }
    commit(&store, "absent", "b", vec![]).await?;
    assert_eq!(tags(&directory).await?, expected);
    Ok(())
}

#[tokio::test]
async fn given_invalid_page_snapshots_when_committed_then_resource_tags_and_outbox_are_atomic() -> TestResult {
    let (directory, store, _) = support::fixture().await?;
    commit(&store, "initial", "old", vec![snapshot("one", TagCoverage::Complete, &[("safe", "old")])?]).await?;
    acknowledge(&store).await?;
    store.seed_work(support::work("invalid", "us-east-1")?).await?;
    let lease = store.claim(&SweepId::new("invalid".to_owned())?, 10, 100).await?.ok_or("missing lease")?;
    let mut page = support::page(lease.clone(), vec![support::observation("one", "changed")?], true, None, 11);
    page.tags = vec![snapshot("one", TagCoverage::Complete, &[])?, snapshot("one", TagCoverage::Complete, &[("second", "bad")])?];
    assert!(matches!(store.commit_page(page).await, Err(StoreError::Invalid("page contains duplicate resource tag observations"))));
    let mut page = support::page(lease.clone(), vec![support::observation("one", "changed")?], true, None, 11);
    page.tags = vec![snapshot("one", TagCoverage::Complete, &[])?, snapshot("missing", TagCoverage::Complete, &[])?];
    assert!(matches!(store.commit_page(page).await, Err(StoreError::Invalid("tag observation has no persisted resource"))));
    let mut key = support::resource("one")?;
    key.region = Region::parse("us-west-2")?;
    let mut page = support::page(lease, vec![], true, None, 11);
    page.tags = vec![TagObservation::new(key, TagCoverage::Complete, vec![])?];
    assert!(matches!(store.commit_page(page).await, Err(StoreError::Invalid("page contains data outside its leased account and region"))));
    assert_eq!(tags(&directory).await?, vec![("us-east-1".into(), "one".into(), "safe".into(), "old".into())]);
    assert_eq!(store.projection_resource(&support::resource("one")?).await?.ok_or("missing resource")?.content_hash, *support::observation("one", "old")?.content_hash().as_bytes());
    assert!(store.drain_projection(100).await?.is_empty());
    Ok(())
}

#[test]
fn given_malformed_pairs_when_normalized_then_duplicates_and_bounds_are_rejected() -> TestResult {
    assert!(matches!(Tag::new("", "value"), Err(InventoryError::TagKey)));
    assert!(matches!(Tag::new("x".repeat(129), "value"), Err(InventoryError::TagKey)));
    assert!(matches!(Tag::new("key", "x".repeat(257)), Err(InventoryError::TagValue)));
    assert!(matches!(Tag::new("key", "bad\0value"), Err(InventoryError::TagValue)));
    assert!(matches!(TagObservation::new(support::resource("one")?, TagCoverage::Complete, vec![Tag::new("same", "1")?, Tag::new("same", "2")?]), Err(InventoryError::DuplicateTag)));
    let pairs = (0..101).map(|number| Tag::new(number.to_string(), "v")).collect::<Result<Vec<_>, _>>()?;
    assert!(matches!(TagObservation::new(support::resource("one")?, TagCoverage::Complete, pairs), Err(InventoryError::TagCount)));
    let exact = Tag::new("x".repeat(128), "y".repeat(256))?;
    assert_eq!(exact.key().len(), 128);
    assert_eq!(exact.value().len(), 256);
    Ok(())
}

#[tokio::test]
async fn given_same_id_in_two_regions_when_east_tags_are_removed_then_west_tags_survive() -> TestResult {
    let (directory, store, _) = support::fixture().await?;
    commit(&store, "east", "value", vec![snapshot("one", TagCoverage::Complete, &[("scope", "east")])?]).await?;
    store.seed_work(support::work("west", "us-west-2")?).await?;
    let lease = store.claim(&SweepId::new("west".to_owned())?, 10, 100).await?.ok_or("missing west lease")?;
    let mut west = support::resource("one")?;
    west.region = Region::parse("us-west-2")?;
    let observation = inventory_core::inventory::Observation::new(
        west.clone(), support::observation("one", "value")?.record().clone(),
        &inventory_core::record::Exclusions::none(), None,
    )?;
    let mut page = support::page(lease, vec![observation], true, None, 11);
    page.tags = vec![TagObservation::new(west.clone(), TagCoverage::Complete, vec![Tag::new("scope", "west")?])?];
    store.commit_page(page).await?;
    acknowledge(&store).await?;
    commit(&store, "remove-east", "value", vec![snapshot("one", TagCoverage::Complete, &[])?]).await?;
    assert_eq!(tags(&directory).await?, vec![("us-west-2".into(), "one".into(), "scope".into(), "west".into())]);
    let pending = store.drain_projection(100).await?;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending.first().and_then(|item| item.key.as_ref()), Some(&support::resource("one")?));
    assert!(store.projection_resource(&west).await?.is_some());
    Ok(())
}

async fn coverage(directory: &tempfile::TempDir) -> TestResult<Vec<(String, String, i64, Option<i64>)>> {
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(
        SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")).read_only(true),
    ).await?;
    let rows = sqlx::query_as("SELECT id,coverage,observed_at,last_complete_at FROM tag_observations ORDER BY id")
        .fetch_all(&pool).await?;
    pool.close().await;
    Ok(rows)
}

#[tokio::test]
async fn given_known_empty_snapshot_when_later_fetch_is_partial_then_durable_coverage_keeps_both_facts() -> TestResult {
    let (directory, store, _) = support::fixture().await?;
    commit_at(&store, "complete-empty", "same", vec![snapshot("one", TagCoverage::Complete, &[])?], 11).await?;
    commit_at(&store, "partial", "same", vec![snapshot("one", TagCoverage::Partial, &[("partial", "ignored")])?], 21).await?;
    commit_at(&store, "failed", "same", vec![snapshot("two", TagCoverage::Failed, &[])?], 31).await?;
    let expected = vec![("one".to_owned(), "partial".to_owned(), 21, Some(11)), ("two".to_owned(), "failed".to_owned(), 31, None)];
    assert_eq!(coverage(&directory).await?, expected);
    assert!(tags(&directory).await?.is_empty());
    drop(store);
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(coverage(&directory).await?, expected);
    commit_at(&store, "complete-again", "same", vec![snapshot("one", TagCoverage::Complete, &[])?], 41).await?;
    assert_eq!(coverage(&directory).await?, vec![("one".to_owned(), "complete".to_owned(), 41, Some(41)), ("two".to_owned(), "failed".to_owned(), 31, None)]);
    Ok(())
}

#[tokio::test]
async fn given_valid_schema14_with_tags_when_opened_then_migrates_without_inventing_coverage() -> TestResult {
    let (directory, store, _) = support::fixture().await?;
    commit(&store, "before-upgrade", "same", vec![snapshot("one", TagCoverage::Complete, &[("legacy", "retained")])?]).await?;
    let expected_tags = tags(&directory).await?;
    let expected_outbox = store.drain_projection(100).await?;
    drop(store);
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(
        SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
    ).await?;
    migration::reconstruct_schema14(&pool).await?;
    pool.close().await;
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(tags(&directory).await?, expected_tags);
    assert!(coverage(&directory).await?.is_empty());
    assert_eq!(store.drain_projection(100).await?, expected_outbox);
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(
        SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")).read_only(true),
    ).await?;
    let version: i64 = sqlx::query_scalar("PRAGMA user_version").fetch_one(&pool).await?;
    assert_eq!(version, 16);
    pool.close().await;
    commit_at(&store, "after-upgrade", "same", vec![snapshot("one", TagCoverage::Complete, &[])?], 21).await?;
    assert!(tags(&directory).await?.is_empty());
    Ok(())
}
