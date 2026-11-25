use super::*;
use cmdb_view::{EdgePageRequest, GraphEdge, GraphNode, ViewError};

fn node(region: &str) -> GraphNode {
    GraphNode {
        account: "111111111111".to_owned(),
        region: region.to_owned(),
        kind: "AWS::S3::Bucket".to_owned(),
        id: "shared".to_owned(),
    }
}

#[tokio::test]
async fn given_pinned_reader_when_sweep_changes_keys_then_pages_keep_original_order() -> TestResult
{
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let db = populated(&path).await?;
    let mut snapshot = db.snapshot().await?;
    let first = snapshot
        .page(
            &ResourceFilter::default(),
            PageRequest {
                limit: 1,
                after: None,
            },
        )
        .await?;
    let pool = writer(&path).await?;
    query("DELETE FROM resources WHERE account='111111111111' AND region='us-east-1'")
        .execute(&pool)
        .await?;
    let bytes = record_bytes()?;
    seed(
        &pool,
        Seed {
            table: "bucket_records",
            account: "111111111111",
            region: "global",
            kind: "AWS::S3::Bucket",
            id: "ahead",
        },
        &bytes,
    )
    .await?;
    seed(
        &pool,
        Seed {
            table: "bucket_records",
            account: "333333333333",
            region: "global",
            kind: "AWS::S3::Bucket",
            id: "later",
        },
        &bytes,
    )
    .await?;
    let second = snapshot
        .page(
            &ResourceFilter::default(),
            PageRequest {
                limit: 3,
                after: first.next,
            },
        )
        .await?;
    assert_eq!(first.total_matching, 4);
    assert_eq!(second.total_matching, 4);
    let identities: Vec<_> = second
        .resources
        .iter()
        .map(|r| (r.key.account.as_str(), r.key.region.as_str()))
        .collect();
    assert_eq!(
        identities,
        vec![
            ("111111111111", "us-east-1"),
            ("222222222222", "eu-west-1"),
            ("222222222222", "us-west-2")
        ]
    );
    assert!(second.next.is_none());
    assert!(snapshot.detail(&node("us-east-1")).await?.is_some());
    snapshot.finish().await?;
    assert_eq!(db.counts().await?.resources, 5);
    assert!(db.detail(&node("us-east-1")).await?.is_none());
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_snapshot_when_first_read_follows_sweep_then_details_keep_begin_state() -> TestResult
{
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let db = populated(&path).await?;
    let mut snapshot = db.snapshot().await?;
    let pool = writer(&path).await?;
    let mut tx = pool.begin().await?;
    query("UPDATE resources SET name='new-name',content_hash=? WHERE region='global'")
        .bind([8_u8; 32].as_slice())
        .execute(&mut *tx)
        .await?;
    query("UPDATE bucket_records SET content_hash=? WHERE region='global'")
        .bind([8_u8; 32].as_slice())
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    let old = snapshot
        .detail(&node("global"))
        .await?
        .ok_or("missing old resource")?;
    let new = db
        .detail(&node("global"))
        .await?
        .ok_or("missing new resource")?;
    assert_eq!(old.name.as_deref(), Some("shared"));
    assert_eq!(old.content_hash.as_bytes(), &[7_u8; 32]);
    assert_eq!(old.record.as_bytes(), record_bytes()?.as_slice());
    assert_eq!(new.name.as_deref(), Some("new-name"));
    assert_eq!(new.content_hash.as_bytes(), &[8_u8; 32]);
    snapshot.finish().await?;
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_edge_cursor_when_sweep_deletes_and_adds_then_snapshot_has_no_gaps() -> TestResult {
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let db = populated(&path).await?;
    let mut snapshot = db.snapshot().await?;
    let root = node("global");
    let first = snapshot
        .edges_page(
            &root,
            EdgePageRequest {
                limit: 1,
                after: None,
            },
        )
        .await?;
    let pool = writer(&path).await?;
    query("DELETE FROM edges WHERE to_account='222222222222'")
        .execute(&pool)
        .await?;
    seed_edge(
        &pool,
        &GraphEdge {
            from: root.clone(),
            to: root.clone(),
            relation: "self".to_owned(),
            provenance: "inferred".to_owned(),
        },
    )
    .await?;
    let second = snapshot
        .edges_page(
            &root,
            EdgePageRequest {
                limit: 1,
                after: first.next,
            },
        )
        .await?;
    assert_eq!(first.total_matching, 2);
    assert_eq!(second.total_matching, 2);
    assert_eq!(
        second.edges.first().map(|edge| edge.to.account.as_str()),
        Some("222222222222")
    );
    assert!(second.next.is_none());
    assert_eq!(snapshot.lod_summary().await?.connected_resources, 3);
    snapshot.finish().await?;
    assert_eq!(db.lod_summary().await?.connected_resources, 2);
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_mismatched_detail_hash_when_reading_then_corruption_is_not_missing() -> TestResult {
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let db = populated(&path).await?;
    let pool = writer(&path).await?;
    query("UPDATE bucket_records SET content_hash=? WHERE region='global'")
        .bind([9_u8; 32].as_slice())
        .execute(&pool)
        .await?;
    assert!(matches!(
        db.detail(&node("global")).await,
        Err(ViewError::StoredRow)
    ));
    pool.close().await;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_pinned_reader_when_sweep_changes_keys_then_pages_keep_begin_state()
-> TestResult {
    use super::postgres_support as pg;
    // Given a snapshot pinned before its caller's first read.
    let fixture = super::support::postgres_fixture().await?;
    let db = fixture.reader().await?;
    let mut snapshot = db.snapshot().await?;
    // When another connection removes one key and PgStore writes keys before and after it.
    pg_change_resources(&fixture).await?;
    let first = snapshot
        .page(
            &ResourceFilter::default(),
            PageRequest {
                limit: 1,
                after: None,
            },
        )
        .await?;
    let second = snapshot
        .page(
            &ResourceFilter::default(),
            PageRequest {
                limit: 3,
                after: first.next,
            },
        )
        .await?;
    // Then resource pages and details retain the old revision; a new reader sees the sweep.
    assert_eq!((first.total_matching, second.total_matching), (4, 4));
    assert_eq!(
        second
            .resources
            .iter()
            .map(|row| (row.key.account.as_str(), row.key.region.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("111111111111", "us-east-1"),
            ("222222222222", "eu-west-1"),
            ("222222222222", "us-west-2")
        ]
    );
    let deleted = pg::node("111111111111", "us-east-1", 0, "shared");
    assert_eq!(
        snapshot.detail(&deleted).await?.map(|row| row.record),
        Some(pg::record("shared", 20)?)
    );
    assert_eq!(second.next, None);
    snapshot.finish().await?;
    assert_eq!(db.detail(&deleted).await?, None);
    assert_eq!(db.counts().await?.resources, 5);
    drop(db);
    fixture.finish().await
}

async fn pg_change_resources(fixture: &super::postgres_support::Fixture) -> TestResult {
    sqlx::query(
        "DELETE FROM inventory.resources WHERE account='111111111111' AND region='us-east-1'",
    )
    .execute(&fixture.pool)
    .await?;
    fixture
        .commit(
            "new-root",
            "111111111111",
            "global",
            0,
            &["ahead", "shared"],
            50,
        )
        .await?;
    fixture
        .commit("later", "333333333333", "global", 0, &["later"], 60)
        .await?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_snapshot_when_first_detail_follows_sweep_then_hash_and_record_are_pinned()
-> TestResult {
    use super::postgres_support as pg;
    // Given a retained snapshot with no application reads yet.
    let fixture = super::support::postgres_fixture().await?;
    let db = fixture.reader().await?;
    let mut snapshot = db.snapshot().await?;
    // When PgStore replaces a record while the reader holds the transaction.
    fixture
        .commit("updated", "111111111111", "global", 0, &["shared"], 50)
        .await?;
    let old = snapshot
        .detail(&pg::root())
        .await?
        .ok_or("missing old record")?;
    let new = db.detail(&pg::root()).await?.ok_or("missing new record")?;
    // Then record bytes and matching hashes belong to their respective revisions.
    assert_eq!(old.record, pg::record("shared", 10)?);
    assert_eq!(new.record, pg::record("shared", 50)?);
    assert_eq!(
        old.content_hash,
        old.record
            .content_hash(&inventory_core::record::Exclusions::none())?
    );
    assert_eq!(
        new.content_hash,
        new.record
            .content_hash(&inventory_core::record::Exclusions::none())?
    );
    snapshot.finish().await?;
    drop(db);
    fixture.finish().await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_edge_cursor_when_sweep_changes_edges_then_snapshot_has_no_gaps()
-> TestResult {
    use super::postgres_support as pg;
    // Given a reader on the original two edges.
    let fixture = super::support::postgres_fixture().await?;
    let db = fixture.reader().await?;
    let mut snapshot = db.snapshot().await?;
    let root = pg::root();
    let first = snapshot
        .edges_page(
            &root,
            EdgePageRequest {
                limit: 1,
                after: None,
            },
        )
        .await?;
    // When the writer deletes an edge and adds a self-loop.
    sqlx::query("DELETE FROM inventory.edges WHERE to_account='222222222222'")
        .execute(&fixture.pool)
        .await?;
    fixture
        .edge(&pg::edge(&root, &root, "self", "inferred"))
        .await?;
    let second = snapshot
        .edges_page(
            &root,
            EdgePageRequest {
                limit: 1,
                after: first.next,
            },
        )
        .await?;
    // Then the cursor and LOD counts still describe the original graph.
    assert_eq!((first.total_matching, second.total_matching), (2, 2));
    assert_eq!(
        second
            .edges
            .iter()
            .map(|edge| edge.to.account.as_str())
            .collect::<Vec<_>>(),
        vec!["222222222222"]
    );
    assert_eq!(second.next, None);
    assert_eq!(snapshot.lod_summary().await?, pg_lod(3, 1));
    snapshot.finish().await?;
    assert_eq!(db.lod_summary().await?, pg_lod(2, 2));
    drop(db);
    fixture.finish().await
}

fn pg_lod(connected_resources: u64, isolated_resources: u64) -> cmdb_view::LodSummary {
    cmdb_view::LodSummary {
        resources: 4,
        edges: 2,
        connected_resources,
        isolated_resources,
    }
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_mismatched_detail_hash_when_reading_then_corruption_is_not_missing()
-> TestResult {
    use super::postgres_support as pg;
    // Given a kind table whose hash differs from the shared resource row.
    let fixture = super::support::postgres_fixture().await?;
    sqlx::query("UPDATE inventory.kind_0 SET content_hash=$1 WHERE region='global'")
        .bind([9_u8; 32].as_slice())
        .execute(&fixture.pool)
        .await?;
    let db = fixture.reader().await?;
    // When fetching the record, corruption is a typed failure rather than absence.
    let error = db
        .detail(&pg::root())
        .await
        .err()
        .ok_or("expected stored-row failure")?;
    // Then the public diagnostic is exact.
    assert_eq!(error.to_string(), "invalid stored resource row");
    drop(db);
    fixture.finish().await
}
