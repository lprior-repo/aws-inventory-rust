use super::*;
use cmdb_view::{GraphNode, ResourceTag, TagFilter};

fn root() -> GraphNode {
    GraphNode { account: "111111111111".into(), region: "global".into(),
        kind: "AWS::S3::Bucket".into(), id: "shared".into() }
}

fn tag(key: &str, value: &str) -> ResourceTag {
    ResourceTag { key: key.into(), value: value.into() }
}

fn filter(key: &str, value: Option<&str>) -> ResourceFilter {
    ResourceFilter { tag: Some(TagFilter { key: key.into(), value: value.map(str::to_owned) }),
        ..ResourceFilter::default() }
}

fn request(limit: usize) -> PageRequest { PageRequest { limit, after: None } }

async fn insert(pool: &SqlitePool, node: &GraphNode, key: &str, value: &str) -> TestResult {
    query("INSERT INTO tags VALUES (?,?,?,?,?,?)")
        .bind(&node.account).bind(&node.region).bind(&node.kind).bind(&node.id)
        .bind(key).bind(value).execute(pool).await?;
    Ok(())
}

#[tokio::test]
async fn given_shared_ids_when_filtering_tags_then_all_four_resource_key_fields_scope_matches() -> TestResult {
    // Given: tags on resources sharing an ID and selected identity components.
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let db = populated(&path).await?;
    let pool = writer(&path).await?;
    let selected = root();
    let other_kind = GraphNode { kind: "AWS::EC2::Instance".into(), ..root() };
    seed(&pool, Seed { table: "instance_records", account: &other_kind.account,
        region: &other_kind.region, kind: &other_kind.kind, id: &other_kind.id }, &record_bytes()?).await?;
    insert(&pool, &selected, "env", "selected").await?;
    insert(&pool, &other_kind, "env", "other-kind").await?;
    insert(&pool, &GraphNode { region: "us-east-1".into(), ..root() }, "env", "other-region").await?;
    insert(&pool, &GraphNode { account: "222222222222".into(), region: "eu-west-1".into(), ..root() }, "env", "other-account").await?;
    // When: select exactly the stored tag key/value through the public page API.
    let found = db.page(&filter("env", Some("selected")), request(10)).await?;
    let detail = db.detail(&selected).await?.ok_or("selected resource missing")?;
    // Then: neither other kind/region/account joins through the shared ID.
    assert_eq!(found.total_matching, 1);
    assert_eq!(found.resources, vec![detail.clone()]);
    assert_eq!(detail.tags, vec![tag("env", "selected")]);
    assert!(!detail.tags_truncated);
    assert_eq!(db.detail(&other_kind).await?.ok_or("other kind missing")?.tags,
        vec![tag("env", "other-kind")]);
    assert_eq!(db.lod_summary().await?.edges, 2);
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_literal_tag_values_when_filtering_then_empty_exact_values_and_keyset_counts_agree() -> TestResult {
    // Given: hostile values, empty values, and another key with the desired value.
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let db = populated(&path).await?;
    let pool = writer(&path).await?;
    let hostile = "team'\\%_`) OR 1=1 --";
    insert(&pool, &root(), hostile, "").await?;
    insert(&pool, &root(), "another", "desired").await?;
    let east = GraphNode { region: "us-east-1".into(), ..root() };
    insert(&pool, &east, hostile, "desired").await?;
    // When: key-only pagination and literal key/value restrictions use the reader.
    let first = db.page(&filter(hostile, None), request(1)).await?;
    let second = db.page(&filter(hostile, None), PageRequest { limit: 1, after: first.next.clone() }).await?;
    let empty = db.page(&filter(hostile, Some("")), request(10)).await?;
    let desired = db.page(&filter(hostile, Some("desired")), request(10)).await?;
    // Then: exact tuple semantics, no SQL expansion, and accurate bounded totals.
    assert_eq!((first.total_matching, second.total_matching), (2, 2));
    assert!(first.next.is_some());
    assert!(second.next.is_none());
    assert_eq!(empty.resources.first().map(|row| row.key.region.as_str()), Some("global"));
    assert_eq!(empty.total_matching, 1);
    assert_eq!(desired.resources.first().map(|row| row.key.region.as_str()), Some("us-east-1"));
    assert_eq!(desired.total_matching, 1);
    assert_eq!(db.page(&filter("' OR 1=1 --", None), request(10)).await?.total_matching, 0);
    assert_eq!(db.page(&filter(hostile, Some("%")), request(10)).await?.total_matching, 0);
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_retained_tag_snapshot_when_only_tags_change_then_filters_and_details_keep_opening_tags() -> TestResult {
    // Given: the transaction is pinned before any tag read.
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let db = populated(&path).await?;
    let pool = writer(&path).await?;
    insert(&pool, &root(), "revision", "old").await?;
    let mut retained = db.snapshot().await?;
    // When: the resource record is unchanged but the writer replaces the tags.
    query("UPDATE tags SET tag_value='new'").execute(&pool).await?;
    let old = retained.detail(&root()).await?.ok_or("retained resource missing")?;
    let new = db.detail(&root()).await?.ok_or("fresh resource missing")?;
    // Then: both filters and detail use their own consistent transaction revision.
    assert_eq!(old.tags, vec![tag("revision", "old")]);
    assert_eq!(new.tags, vec![tag("revision", "new")]);
    assert_eq!(old.content_hash, new.content_hash);
    assert_eq!(old.record, new.record);
    assert_eq!(retained.page(&filter("revision", Some("old")), request(10)).await?.resources, vec![old]);
    assert_eq!(retained.page(&filter("revision", Some("new")), request(10)).await?.total_matching, 0);
    assert_eq!(db.page(&filter("revision", Some("new")), request(10)).await?.resources, vec![new]);
    retained.finish().await?;
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_oversized_persisted_tag_set_when_reading_then_details_and_pages_expose_bounded_truncation() -> TestResult {
    // Given: a real SQLite rowset exceeds the consumer defense-in-depth cap.
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let db = populated(&path).await?;
    let pool = writer(&path).await?;
    query("WITH RECURSIVE n(i) AS (VALUES(0) UNION ALL SELECT i+1 FROM n WHERE i<1000) INSERT INTO tags SELECT '111111111111','global','AWS::S3::Bucket','shared',printf('key-%04d',i),'value' FROM n").execute(&pool).await?;
    // When: public detail and filtered page reads decode the persisted tags.
    let detail = db.detail(&root()).await?.ok_or("resource missing")?;
    let found = db.page(&filter("key-1000", Some("value")), request(10)).await?;
    // Then: detail is bounded while filtering still matches the unabridged store.
    assert_eq!(detail.tags.len(), cmdb_view::MAX_PAGE_LIMIT);
    assert_eq!(detail.tags.first(), Some(&tag("key-0000", "value")));
    assert_eq!(detail.tags.last(), Some(&tag("key-0999", "value")));
    assert!(detail.tags_truncated);
    assert_eq!(found.resources, vec![detail]);
    assert_eq!(found.total_matching, 1);
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_invalid_tag_read_limits_when_requested_then_reader_rejects_before_native_network_access() -> TestResult {
    // Given: a deliberately unavailable endpoint.
    let graph = cmdb_view::GraphConnection::connect("redis://127.0.0.1:1", "tags")?;
    // When/Then: invalid bounds fail at the public boundary, not on connection.
    assert!(matches!(graph.tag_affiliations(&root(), 0).await, Err(cmdb_view::ViewError::PageLimit)));
    assert!(matches!(graph.tag_affiliations(&root(), cmdb_view::MAX_PAGE_LIMIT.saturating_add(1)).await,
        Err(cmdb_view::ViewError::PageLimit)));
    let malformed = GraphNode { account: "not-an-account".into(), ..root() };
    assert!(matches!(graph.tag_affiliations(&malformed, 1).await, Err(cmdb_view::ViewError::Identity(_))));
    Ok(())
}

#[tokio::test]
async fn given_unknown_tag_coverage_when_status_is_persisted_then_retained_reader_does_not_infer_completeness() -> TestResult {
    // Given: a real resource without any persisted tag observation.
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let db = populated(&path).await?;
    let pool = writer(&path).await?;
    let mut retained = db.snapshot().await?;
    // When: the writer records a complete-empty result after the snapshot opens.
    query("INSERT INTO tag_observations VALUES ('111111111111','global','AWS::S3::Bucket','shared','complete',42,42)")
        .execute(&pool).await?;
    let old = retained.detail(&root()).await?.ok_or("retained root missing")?;
    let new = db.detail(&root()).await?.ok_or("fresh root missing")?;
    // Then: equal empty pair sets have distinct durable knowledge/revision states.
    assert_eq!(old.tags, vec![]);
    assert_eq!(old.tag_observation, None);
    assert_eq!(new.tags, vec![]);
    assert_eq!(new.tag_observation, Some(cmdb_view::StoredTagObservation {
        coverage: cmdb_view::TagCoverage::Complete, observed_at_ms: 42, last_complete_at_ms: Some(42),
    }));
    let other = GraphNode { region: "us-east-1".into(), ..root() };
    assert_eq!(db.detail(&other).await?.and_then(|resource| resource.tag_observation), None);
    retained.finish().await?;
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_invalid_stored_tag_coverage_when_detail_is_requested_then_corruption_is_not_unknown() -> TestResult {
    // Given: a malformed durable coverage row.
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let db = populated(&path).await?;
    let pool = writer(&path).await?;
    query("INSERT INTO tag_observations VALUES ('111111111111','global','AWS::S3::Bucket','shared','made-up',42,NULL)")
        .execute(&pool).await?;
    // When/Then: public detail fails precisely instead of inventing coverage.
    assert!(matches!(db.detail(&root()).await, Err(cmdb_view::ViewError::StoredRow)));
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_schema14_before_writer_migration_when_reader_opens_then_upgrade_requirement_is_explicit() -> TestResult {
    // Given: a legacy database that has not yet been reopened by the writer.
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let pool = writer(&path).await?;
    query("PRAGMA user_version=14").execute(&pool).await?;
    pool.close().await;
    // When/Then: no reader-side mutation, inference, or silent metadata fallback.
    assert!(matches!(Database::open(&path).await, Err(cmdb_view::ViewError::Version(14))));
    Ok(())
}
