#![cfg(feature = "aws")]
use anyhow::{Context, Result};
use inventory_core::inventory::{AccountId, Kind, Region, ResourceId, ResourceKey};
use inventory_engine::inventory::BinaryStore;
use serde_json::json;
use std::{path::{Path, PathBuf}, time::Duration};
#[path = "sweep_tag_endpoint_contract/fixture.rs"]
mod fixture;
#[path = "sweep_tag_endpoint_contract/peer.rs"]
mod peer;
use fixture::Fixture;
use peer::{Mode, Peer};
const ACCOUNT: &str = "123456789012";
const REGION: &str = "us-east-1";
const OWNER: &str = "arn:aws:taginventory:us-east-1:123456789012:item/same-id";
const PARENT: &str = "AWS::ApiOnly::TagParent";
const OTHER: &str = "AWS::ApiOnly::TagOther";
const DEADLINE: Duration = Duration::from_secs(30);

async fn tags(path: &Path, kind: &str) -> Result<Vec<(String, String)>> {
    let store = BinaryStore::open(path).await?;
    let key = ResourceKey { account: AccountId::parse(ACCOUNT)?, region: Region::parse(REGION)?, kind: Kind::parse(kind)?, id: ResourceId::parse(OWNER)? };
    let mut snapshot = store.begin_projection_snapshot().await?;
    let page = snapshot.tags_for(&key, None, 100).await?;
    assert!(page.next.is_none());
    let tags = page.tags.into_iter().map(|tag| (tag.key, tag.value)).collect();
    snapshot.finish().await?;
    Ok(tags)
}
async fn assert_identity_isolation(path: &Path, expected: &[(&str, &str)]) -> Result<()> {
    assert_eq!(tags(path, PARENT).await?, expected.iter().map(|(key, value)| ((*key).to_owned(), (*value).to_owned())).collect::<Vec<_>>());
    assert_eq!(tags(path, OTHER).await?, vec![("other".into(), "kept".into())]);
    // New handles above intentionally cover persisted readers, not just a live in-memory cache.
    let pool = connect(path).await?;
    assert_eq!(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resources").fetch_one(&pool).await?, 2, "tag responses must not invent resource rows");
    pool.close().await;
    Ok(())
}

async fn connect(path: &Path) -> Result<sqlx::SqlitePool> {
    Ok(sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(path)).await?)
}

#[tokio::test]
async fn given_signed_paginated_tags_when_collected_then_all_pages_replace_only_the_exact_parent_kind() -> Result<()> {
    for (map, mode) in [(false, Mode::Complete), (true, Mode::MapComplete)] {
        let fixture = Fixture::create(map)?;
        let peer = Peer::start(fixture.root.path(), mode).await?;
        let output = fixture.collect(&peer, false, 20).await?;
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(peer.root_calls(), 1);
        assert_eq!(peer.tag_calls(), 2, "child scheduling and signed continuation must both execute");
        assert_identity_isolation(&fixture.database, &[("alpha", "one"), ("omega", "")]).await?;
        peer.stop().await?;
    }
    Ok(())
}

#[tokio::test]
async fn given_strict_wire_failure_absence_or_partial_completion_when_collected_then_previous_known_tags_are_not_replaced() -> Result<()> {
    for (map, mode, code, max_pages, expected_calls, outcome) in [
        (false, Mode::MissingKey, 2, 20, 1, Some("failed")),
        (false, Mode::MissingValue, 2, 20, 1, Some("failed")),
        (false, Mode::MissingCompletion, 2, 20, 1, Some("failed")),
        (false, Mode::NullEntry, 2, 20, 1, Some("failed")),
        (false, Mode::MixedNullEntry, 2, 20, 1, Some("failed")),
        (true, Mode::NullMap, 2, 20, 1, Some("failed")),
        (true, Mode::MixedNullMap, 2, 20, 1, Some("failed")),
        (false, Mode::Absent, 0, 20, 1, None),
        (false, Mode::Repeat, 2, 20, 2, Some("partial")),
        (false, Mode::Denied, 2, 20, 2, Some("inaccessible")),
        (false, Mode::Complete, 2, 1, 1, Some("partial")),
    ] {
        let fixture = Fixture::create(map)?;
        let peer = Peer::start(fixture.root.path(), mode).await?;
        let output = fixture.collect(&peer, false, max_pages).await?;
        assert_eq!(output.status.code(), Some(code), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(peer.tag_calls(), expected_calls);
        assert_identity_isolation(&fixture.database, &[("known", "old")]).await?;
        let pool = connect(&fixture.database).await?;
        let outcomes: Vec<String> = sqlx::query_scalar("SELECT outcome FROM scope_outcomes WHERE outcome<>'complete' ORDER BY outcome").fetch_all(&pool).await?;
        assert_eq!(outcomes, outcome.into_iter().map(str::to_owned).collect::<Vec<_>>());
        pool.close().await;
        peer.stop().await?;
    }
    Ok(())
}

#[tokio::test]
async fn given_kind_colliding_parent_tags_when_explicit_empty_snapshot_commits_then_other_kind_survives_reopen() -> Result<()> {
    let fixture = Fixture::create(false)?;
    let peer = Peer::start(fixture.root.path(), Mode::Empty).await?;
    let output = fixture.collect(&peer, false, 20).await?;
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(peer.tag_calls(), 1);
    assert_identity_isolation(&fixture.database, &[]).await?;
    peer.stop().await
}

#[tokio::test]
async fn given_checkpointed_partial_tags_when_process_restarts_then_pending_pairs_resume_without_applying_partial_or_replaying_parent() -> Result<()> {
    let fixture = Fixture::create(false)?;
    let peer = Peer::start(fixture.root.path(), Mode::Gate).await?;
    let mut child = fixture.spawn(&peer, false, 20)?;
    if let Err(error) = peer.wait_for_checkpoint().await {
        child.kill().await?;
        return Err(error);
    }
    child.kill().await?;
    let pool = connect(&fixture.database).await?;
    let pending: Vec<Vec<u8>> = sqlx::query_scalar("SELECT token FROM work_queue WHERE token IS NOT NULL AND state IN(0,1)").fetch_all(&pool).await?;
    assert_eq!(pending.len(), 1, "first page must atomically checkpoint its tag accumulator");
    let value = inventory_core::record::Record::from_bytes(pending.into_iter().next().ok_or_else(|| anyhow::anyhow!("pending tag checkpoint missing"))?)?;
    let Some(inventory_core::record::ValueRef::Map(accumulation)) = value.root().get_field(inventory_core::record::FieldId::new(65_529)?)? else { anyhow::bail!("pending tag snapshot absent from durable cursor"); };
    let Some(inventory_core::record::ValueRef::List(pairs)) = accumulation.get_field(inventory_core::record::FieldId::new(2)?)? else { anyhow::bail!("pending tag pairs absent"); };
    assert_eq!(pairs.count()?, 1, "pending pairs must be durable work, not authoritative tags");
    pool.close().await;
    assert_identity_isolation(&fixture.database, &[("known", "old")]).await?;
    let output = fixture.collect(&peer, true, 20).await?;
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(peer.root_calls(), 1);
    assert_eq!(peer.tag_calls(), 3);
    assert_identity_isolation(&fixture.database, &[("alpha", "one"), ("omega", "")]).await?;
    peer.stop().await
}
