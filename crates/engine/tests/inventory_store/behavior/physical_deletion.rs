use super::*;

async fn counts(pool: &sqlx::SqlitePool) -> TestResult<(i64, i64, i64)> {
    let gone: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resources WHERE id='gone'")
        .fetch_one(pool)
        .await?;
    let edges: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM edges")
        .fetch_one(pool)
        .await?;
    let typed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM thing_table")
        .fetch_one(pool)
        .await?;
    Ok((gone, edges, typed))
}

async fn seed_with_edge(store: &BinaryStore) -> TestResult {
    store
        .seed_works(&[relationship_work("s1", "us-east-1", None, Some("beta"))?])
        .await?;
    let lease = store
        .claim(&SweepId::new("s1".to_owned())?, 100, 20)
        .await?
        .ok_or("missing lease")?;
    let mut commit = page(
        lease,
        vec![observation("keep", "beta")?, observation("gone", "alpha")?],
        true,
        None,
        101,
    );
    commit.edges = vec![Edge {
        from: resource("gone")?,
        to: resource("keep")?,
        relation: Relation::parse("depends-on")?,
        source: EdgeSource::ReviewedRule,
    }];
    store.commit_page(commit).await?;
    for key in ["keep", "gone"] {
        assert!(store.projection_resource(&resource(key)?).await?.is_some());
    }
    Ok(())
}

async fn miss_store(store: &BinaryStore, sweep: &str, base: i64) -> TestResult<()> {
    store
        .seed_works(&[relationship_work(sweep, "us-east-1", None, Some("beta"))?])
        .await?;
    let lease = store
        .claim(&SweepId::new(sweep.to_owned())?, base, 20)
        .await?
        .ok_or("missing lease")?;
    store
        .commit_page(page(
            lease,
            vec![observation("keep", "beta")?],
            true,
            None,
            base.saturating_add(1),
        ))
        .await?;
    Ok(())
}

#[tokio::test]
async fn given_vanished_resource_when_two_complete_sweeps_miss_it_then_sqlite_rows_edges_and_kind_table_are_empty()
-> TestResult {
    let (directory, store, _) = fixture().await?;

    seed_with_edge(&store).await?;

    for pending in store.drain_projection(100).await? {
        store
            .ack_projection(match pending.key {
                Some(key) => {
                    inventory_engine::inventory::ProjectionAck::new(key, pending.revision)?
                }
                None => inventory_engine::inventory::ProjectionAck::full_rebuild(pending.revision)?,
            })
            .await?;
    }

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(directory.path().join("inventory.sqlite")),
        )
        .await?;

    let (gone_rows, edge_rows, typed_rows) = counts(&pool).await?;
    assert_eq!(gone_rows, 1);
    assert!(edge_rows >= 1);
    assert_eq!(typed_rows, 2);

    miss_store(&store, "s2", 200).await?;
    assert!(
        store
            .projection_resource(&resource("gone")?)
            .await?
            .is_some()
    );
    let (gone1, edge1, typed1) = counts(&pool).await?;
    assert_eq!(gone1, 1);
    assert_eq!(edge1, edge_rows);
    assert_eq!(typed1, 2);

    miss_store(&store, "s3", 300).await?;
    assert!(
        store
            .projection_resource(&resource("gone")?)
            .await?
            .is_none()
    );
    assert!(
        store
            .projection_resource(&resource("keep")?)
            .await?
            .is_some()
    );
    let (gone2, edge2, typed2) = counts(&pool).await?;
    assert_eq!(
        (gone2, edge2, typed2),
        (0, 0, 1),
        "gone and its edges removed, keep's kind-table row retained"
    );
    pool.close().await;
    Ok(())
}
