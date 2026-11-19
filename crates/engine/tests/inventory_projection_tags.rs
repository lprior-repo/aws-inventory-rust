#[path = "inventory_store/support.rs"]
mod support;

use inventory_core::inventory::{AccountId, Kind, Region, ResourceKey};
use inventory_engine::inventory::{BinaryStore, ProjectionTag, StoreError};
use sqlx::SqlitePool;
use support::{TestResult, fixture, resource};

async fn tagged_fixture() -> TestResult<(tempfile::TempDir, BinaryStore, SqlitePool)> {
    let (directory, store, _) = fixture().await?;
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    let root = resource("shared-id")?;
    let mut other_account = root.clone();
    other_account.account = AccountId::parse("000000000000")?;
    let mut other_region = root.clone();
    other_region.region = Region::parse("us-west-2")?;
    let mut other_kind = root.clone();
    other_kind.kind = Kind::parse("AWS::Example::Other")?;
    for (key, tags) in [
        (root, vec![("a", "first"), ("b", "second")]),
        (other_account, vec![("a", "account")]),
        (other_region, vec![("a", "region")]),
        (other_kind, vec![("a", "kind")]),
    ] {
        seed_tags(&pool, &key, &tags).await?;
    }
    Ok((directory, store, pool))
}

async fn seed_tags(pool: &SqlitePool, key: &ResourceKey, tags: &[(&str, &str)]) -> TestResult {
    sqlx::query("INSERT INTO resources(account,region,kind,id,content_hash,first_seen,last_changed) VALUES(?1,?2,?3,?4,?5,0,0)")
        .bind(key.account.as_str()).bind(key.region.as_str()).bind(key.kind.as_str())
        .bind(key.id.as_str()).bind(vec![0_u8; 32]).execute(pool).await?;
    for (tag_key, tag_value) in tags {
        sqlx::query(
            "INSERT INTO tags(account,region,kind,id,tag_key,tag_value) VALUES(?1,?2,?3,?4,?5,?6)",
        )
        .bind(key.account.as_str())
        .bind(key.region.as_str())
        .bind(key.kind.as_str())
        .bind(key.id.as_str())
        .bind(tag_key)
        .bind(tag_value)
        .execute(pool)
        .await?;
    }
    Ok(())
}

#[tokio::test]
async fn given_colliding_resource_ids_when_tags_are_paged_then_only_full_scoped_rows_are_read()
-> TestResult {
    // Given the same ID and tag key in several account/region/kind scopes.
    let (_directory, store, pool) = tagged_fixture().await?;
    let root = resource("shared-id")?;
    let mut snapshot = store.begin_projection_snapshot().await?;
    // When a two-tag resource is read one tag per page.
    let first = snapshot.tags_for(&root, None, 1).await?;
    assert_eq!(
        first.tags,
        vec![ProjectionTag {
            key: "a".into(),
            value: "first".into()
        }]
    );
    let cursor = first.next.ok_or("first tag page lacks continuation")?;
    let last = snapshot.tags_for(&root, Some(cursor.clone()), 1).await?;
    // Then the final full page is terminal, and replay returns exactly that page.
    assert_eq!(
        last.tags,
        vec![ProjectionTag {
            key: "b".into(),
            value: "second".into()
        }]
    );
    assert!(last.next.is_none());
    assert_eq!(snapshot.tags_for(&root, Some(cursor), 1).await?, last);
    let empty = snapshot
        .tags_for(&resource("not-present")?, None, 1)
        .await?;
    assert!(empty.tags.is_empty());
    assert!(empty.next.is_none());
    snapshot.finish().await?;
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_a_tag_snapshot_when_sqlite_changes_then_pages_keep_the_original_revision()
-> TestResult {
    // Given a snapshot pinned by its first SELECT.
    let (_directory, store, pool) = tagged_fixture().await?;
    let root = resource("shared-id")?;
    let mut snapshot = store.begin_projection_snapshot().await?;
    let first = snapshot.tags_for(&root, None, 1).await?;
    // When a writer changes the next tag between page reads.
    sqlx::query("UPDATE tags SET tag_value='updated' WHERE account=?1 AND region=?2 AND kind=?3 AND id=?4 AND tag_key='b'")
        .bind(root.account.as_str()).bind(root.region.as_str()).bind(root.kind.as_str())
        .bind(root.id.as_str()).execute(&pool).await?;
    let next = snapshot.tags_for(&root, first.next, 1).await?;
    // Then the original revision remains visible until the snapshot finishes.
    assert_eq!(
        next.tags,
        vec![ProjectionTag {
            key: "b".into(),
            value: "second".into()
        }]
    );
    snapshot.finish().await?;
    let mut fresh = store.begin_projection_snapshot().await?;
    let tags = fresh.tags_for(&root, None, 1000).await?;
    assert_eq!(
        tags.tags.last().map(|tag| tag.value.as_str()),
        Some("updated")
    );
    assert!(tags.next.is_none());
    fresh.finish().await?;
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_a_tag_snapshot_when_page_limit_is_invalid_then_reads_fail_explicitly() -> TestResult
{
    let (_directory, store, pool) = tagged_fixture().await?;
    let mut snapshot = store.begin_projection_snapshot().await?;
    for limit in [0, 1001] {
        assert!(matches!(
            snapshot
                .tags_for(&resource("shared-id")?, None, limit)
                .await,
            Err(StoreError::Invalid("projection page size exceeds bounds"))
        ));
    }
    snapshot.finish().await?;
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_oversized_legacy_tags_when_projected_then_excluded_without_error() -> TestResult {
    let (directory, store, pool) = tagged_fixture().await?;
    let root = resource("shared-id")?;
    sqlx::query("UPDATE tags SET tag_value=?1 WHERE account=?2 AND region=?3 AND kind=?4 AND id=?5 AND tag_key='a'")
        .bind("x".repeat(inventory_core::inventory::MAX_TAG_VALUE_CHARS + 1))
        .bind(root.account.as_str()).bind(root.region.as_str()).bind(root.kind.as_str())
        .bind(root.id.as_str()).execute(&pool).await?;
    let mut snapshot = store.begin_projection_snapshot().await?;
    // The oversized tag is excluded; remaining tags are still readable.
    let page = snapshot.tags_for(&root, None, 10).await?;
    assert_eq!(page.tags.len(), 1);
    assert_eq!(page.tags[0].key, "b");
    snapshot.finish().await?;
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_multibyte_tags_at_character_boundary_when_projected_then_read_succeeds() -> TestResult
{
    let (directory, store, pool) = tagged_fixture().await?;
    let root = resource("shared-id")?;
    // 128 Unicode characters, 256 UTF-8 bytes. Well within the character limit.
    let key = "é".repeat(128);
    let value = "é".repeat(256);
    // Use ON CONFLICT because the resource already exists from tagged_fixture().
    sqlx::query("INSERT OR IGNORE INTO resources(account,region,kind,id,content_hash,first_seen,last_changed) VALUES(?1,?2,?3,?4,?5,0,0)")
        .bind(root.account.as_str()).bind(root.region.as_str()).bind(root.kind.as_str())
        .bind(root.id.as_str()).bind(vec![0_u8; 32]).execute(&pool).await?;
    sqlx::query(
        "INSERT INTO tags(account,region,kind,id,tag_key,tag_value) VALUES(?1,?2,?3,?4,?5,?6)",
    )
    .bind(root.account.as_str())
    .bind(root.region.as_str())
    .bind(root.kind.as_str())
    .bind(root.id.as_str())
    .bind(&key)
    .bind(&value)
    .execute(&pool)
    .await?;
    let mut snapshot = store.begin_projection_snapshot().await?;
    let page = snapshot.tags_for(&root, None, 10).await?;
    // The fixture created 2 ASCII tags ("a", "b"); we added one multibyte tag.
    // Verify the multibyte tag is present and readable.
    let multibyte = page
        .tags
        .iter()
        .find(|tag| tag.value == value)
        .ok_or("multibyte tag not found in projection")?;
    assert_eq!(multibyte.key, key);
    assert_eq!(multibyte.value, value);
    assert_eq!(page.tags.len(), 3);
    snapshot.finish().await?;
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_oversized_tag_key_when_projected_then_excluded() -> TestResult {
    let (directory, store, pool) = tagged_fixture().await?;
    let root = resource("shared-id")?;
    // Key with 129 characters exceeds the 128 limit.
    let long_key = "é".repeat(129);
    let value = "value";
    sqlx::query(
        "INSERT INTO tags(account,region,kind,id,tag_key,tag_value) VALUES(?1,?2,?3,?4,?5,?6)",
    )
    .bind(root.account.as_str())
    .bind(root.region.as_str())
    .bind(root.kind.as_str())
    .bind(root.id.as_str())
    .bind(&long_key)
    .bind(value)
    .execute(&pool)
    .await?;
    let mut snapshot = store.begin_projection_snapshot().await?;
    let page = snapshot.tags_for(&root, None, 10).await?;
    // The oversized key should be excluded; only the 2 fixture tags remain.
    assert_eq!(page.tags.len(), 2);
    for tag in page.tags.iter() {
        assert_eq!(tag.key.chars().count(), 1);
    }
    snapshot.finish().await?;
    pool.close().await;
    Ok(())
}
