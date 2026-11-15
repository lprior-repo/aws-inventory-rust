use super::*;

#[tokio::test]
pub(super) async fn given_a_missed_parent_with_eligible_child_when_finalized_then_parent_and_child_tables_are_deleted()
-> TestResult {
    let (directory, store, _) = fixture().await?;
    let parent = resource("parent")?;
    store.seed_work(work("parent-seed", "us-east-1")?).await?;
    let root = store
        .claim(&SweepId::new("parent-seed".to_owned())?, 100, 20)
        .await?
        .ok_or("missing parent lease")?;
    store
        .commit_page(page(
            root,
            vec![observation("parent", "root")?],
            true,
            None,
            101,
        ))
        .await?;

    store.seed_work(child_work("child-seed", &parent)?).await?;
    let child = store
        .claim(&SweepId::new("child-seed".to_owned())?, 200, 20)
        .await?
        .ok_or("missing child lease")?;
    store
        .commit_page(page(
            child,
            vec![observation("child", "leaf")?],
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
    sqlx::query("UPDATE resource_scopes SET missing_complete=2 WHERE id='child'")
        .execute(&pool)
        .await?;

    for sweep in ["parent-miss-one", "parent-miss-two"] {
        store.seed_work(work(sweep, "us-east-1")?).await?;
        let lease = store
            .claim(&SweepId::new(sweep.to_owned())?, 300, 20)
            .await?
            .ok_or("missing parent miss lease")?;
        store
            .commit_page(page(lease, Vec::new(), true, None, 301))
            .await?;
    }

    assert_eq!(store.projection_resource(&parent).await?, None);
    assert_eq!(store.projection_resource(&resource("child")?).await?, None);
    let typed_rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM thing_table")
        .fetch_one(&pool)
        .await?;
    pool.close().await;
    assert_eq!(typed_rows, 0);
    Ok(())
}

#[tokio::test]
pub(super) async fn given_projection_outbox_overflow_when_rebuilt_then_marker_is_acknowledged_atomically()
-> TestResult {
    let (directory, store, _) = fixture().await?;
    store.seed_work(work("overflow", "us-east-1")?).await?;
    let lease = store
        .claim(&SweepId::new("overflow".to_owned())?, 100, 20)
        .await?
        .ok_or("missing overflow lease")?;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    sqlx::query("UPDATE metadata SET value=100000 WHERE key='projection_pending'")
        .execute(&pool)
        .await?;
    store
        .commit_page(page(
            lease,
            vec![observation("thing-1", "alpha")?],
            true,
            None,
            101,
        ))
        .await?;

    let marker = store
        .drain_projection(10)
        .await?
        .into_iter()
        .next()
        .ok_or("missing overflow marker")?;
    assert!(marker.full_rebuild);
    assert!(store.projection_state().await?.rebuild_required);
    store
        .ack_projection(inventory_engine::inventory::ProjectionAck::full_rebuild(
            marker.revision,
        )?)
        .await?;
    assert!(!store.projection_state().await?.rebuild_required);
    pool.close().await;
    Ok(())
}
#[tokio::test]
pub(super) async fn given_an_unchanged_record_when_reobserved_then_no_projection_write_is_pending()
-> TestResult {
    let (_directory, store, _) = fixture().await?;
    store.seed_work(work("first", "us-east-1")?).await?;
    let first = store
        .claim(&SweepId::new("first".to_owned())?, 100, 20)
        .await?
        .ok_or("missing first lease")?;
    store
        .commit_page(page(
            first,
            vec![observation("thing-1", "alpha")?],
            true,
            None,
            101,
        ))
        .await?;
    let pending = store.drain_projection(10).await?;
    let item = pending.first().ok_or("missing initial projection row")?;
    store
        .ack_projection(inventory_engine::inventory::ProjectionAck::new(
            item.key.clone().ok_or("missing resource key")?,
            item.revision,
        )?)
        .await?;

    store.seed_work(work("second", "us-east-1")?).await?;
    let second = store
        .claim(&SweepId::new("second".to_owned())?, 200, 20)
        .await?
        .ok_or("missing second lease")?;
    let result = store
        .commit_page(page(
            second,
            vec![observation("thing-1", "alpha")?],
            true,
            None,
            201,
        ))
        .await?;

    assert_eq!(result.stats.resources_inserted, 0);
    assert_eq!(result.stats.resources_changed, 0);
    assert!(store.drain_projection(10).await?.is_empty());
    Ok(())
}

#[tokio::test]
pub(super) async fn given_a_catalogued_database_when_reopened_then_only_matching_catalog_bytes_resume()
-> TestResult {
    let (directory, store, catalog) = fixture().await?;
    store.seed_work(work("stable-id", "us-east-1")?).await?;
    store.validate_catalog(&catalog).await?;
    let database = directory.path().join("inventory.sqlite");
    let reopened = BinaryStore::open(&database).await?;

    assert_eq!(
        reopened.resume_sweep().await?,
        Some(SweepId::new("stable-id".to_owned())?)
    );
    assert!(matches!(
        reopened.validate_catalog(b"different catalog").await,
        Err(StoreError::CatalogMismatch)
    ));
    Ok(())
}

#[tokio::test]
pub(super) async fn given_a_newer_projection_revision_when_old_revision_is_acked_then_new_revision_remains()
-> TestResult {
    let (_directory, store, _) = fixture().await?;
    store.seed_work(work("revision-one", "us-east-1")?).await?;
    let first = store
        .claim(&SweepId::new("revision-one".to_owned())?, 100, 20)
        .await?
        .ok_or("missing first lease")?;
    store
        .commit_page(page(
            first,
            vec![observation("thing-1", "alpha")?],
            true,
            None,
            101,
        ))
        .await?;
    let old = store
        .drain_projection(10)
        .await?
        .into_iter()
        .next()
        .ok_or("missing first outbox revision")?;
    let old_key = old.key.clone().ok_or("missing resource key")?;
    store
        .ack_projection(inventory_engine::inventory::ProjectionAck::new(
            old_key.clone(),
            old.revision,
        )?)
        .await?;

    store.seed_work(work("revision-two", "us-east-1")?).await?;
    let second = store
        .claim(&SweepId::new("revision-two".to_owned())?, 200, 20)
        .await?
        .ok_or("missing second lease")?;
    store
        .commit_page(page(
            second,
            vec![observation("thing-1", "beta")?],
            true,
            None,
            201,
        ))
        .await?;
    store
        .ack_projection(inventory_engine::inventory::ProjectionAck::new(
            old_key,
            old.revision,
        )?)
        .await?;
    let current = store
        .drain_projection(10)
        .await?
        .into_iter()
        .next()
        .ok_or("new outbox revision was removed")?;

    assert!(current.revision > old.revision);
    assert!(!current.deleted);
    Ok(())
}

#[tokio::test]
pub(super) async fn given_a_failed_edge_insert_when_page_commits_then_all_page_writes_rollback()
-> TestResult {
    let (_directory, store, _) = fixture().await?;
    store.seed_work(work("rollback", "us-east-1")?).await?;
    let lease = store
        .claim(&SweepId::new("rollback".to_owned())?, 100, 20)
        .await?
        .ok_or("missing lease")?;
    let source = resource("thing-1")?;
    let missing = ResourceKey {
        id: ResourceId::parse("missing")?,
        ..source.clone()
    };
    let edge = Edge {
        from: source.clone(),
        to: missing,
        relation: Relation::parse("depends-on")?,
        source: EdgeSource::Model,
    };
    let mut failed = page(
        lease,
        vec![observation("thing-1", "alpha")?],
        true,
        None,
        101,
    );
    failed.edges.push(edge);

    assert!(store.commit_page(failed).await.is_err());
    assert_eq!(store.projection_resource(&source).await?, None);
    Ok(())
}
