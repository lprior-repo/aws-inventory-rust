use super::*;

#[path = "../../scalar_replay_contract/snapshot.rs"]
mod snapshot;

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
pub(super) async fn given_a_catalogued_database_when_reopened_then_only_matching_catalog_bytes_resume()
-> TestResult {
    let (directory, store, catalog) = fixture().await?;
    store.seed_work(work("stable-id", "us-east-1")?).await?;
    let sweep = SweepId::new("stable-id".to_owned())?;
    let account = inventory_core::inventory::AccountId::parse("111111111111")?;
    store
        .create_sweep_manifest(&sweep, account.as_str())
        .await?;
    store.validate_catalog(&catalog).await?;
    let database = directory.path().join("inventory.sqlite");
    let reopened = BinaryStore::open(&database).await?;

    let sweep = SweepId::new("stable-id".to_owned())?;
    let account = inventory_core::inventory::AccountId::parse("111111111111")?;
    reopened.resume_sweep(&sweep, &account).await?;
    assert!(matches!(
        reopened.validate_catalog(b"different catalog").await,
        Err(StoreError::CatalogMismatch)
    ));
    Ok(())
}

#[tokio::test]
pub(super) async fn given_a_failed_edge_insert_when_page_commits_then_all_page_writes_rollback()
-> TestResult {
    let (directory, store, _) = fixture().await?;
    store
        .seed_work(relationship_work(
            "rollback",
            "us-east-1",
            None,
            Some("beta"),
        )?)
        .await?;
    let lease = store
        .claim(&SweepId::new("rollback".to_owned())?, 100, 20)
        .await?
        .ok_or("missing lease")?;
    let source = resource("thing-1")?;
    let target = resource("thing-2")?;
    let edge = Edge {
        from: source.clone(),
        to: target.clone(),
        relation: Relation::parse("depends-on")?,
        source: EdgeSource::ReviewedRule,
    };
    let mut failed = page(
        lease.clone(),
        vec![
            observation("thing-2", "beta")?,
            observation("thing-1", "alpha")?,
        ],
        true,
        None,
        101,
    );
    failed.edges.push(edge.clone());
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    // Both observations and the request-origin relation are valid. Fail only at
    // the actual edge insert, after the resource/facet/typed prefix was written.
    sqlx::query("CREATE TRIGGER fail_reviewed_edge BEFORE INSERT ON edges WHEN NEW.relation='depends-on' BEGIN SELECT RAISE(ABORT,'reviewed edge storage failure'); END")
        .execute(&pool).await?;
    let before = snapshot::read_durable_state(&pool).await?;
    match store.commit_page(failed).await {
        Err(StoreError::Sqlite(sqlx::Error::Database(error))) => {
            assert_eq!(error.message(), "reviewed edge storage failure");
        }
        result => return Err(format!("expected exact SQLite edge failure, got {result:?}").into()),
    }
    assert_eq!(snapshot::read_durable_state(&pool).await?, before);
    assert_eq!(store.projection_resource(&source).await?, None);
    assert_eq!(store.projection_resource(&target).await?, None);
    drop(store);
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(snapshot::read_durable_state(&pool).await?, before);
    sqlx::query("DROP TRIGGER fail_reviewed_edge")
        .execute(&pool)
        .await?;
    let mut retry = page(
        lease,
        vec![
            observation("thing-2", "beta")?,
            observation("thing-1", "alpha")?,
        ],
        true,
        None,
        101,
    );
    retry.edges.push(edge);
    assert_eq!(
        store.commit_page(retry).await?.stats,
        inventory_engine::inventory::ChangeStats {
            resources_inserted: 2,
            edges_changed: 1,
            ..Default::default()
        }
    );
    assert_eq!(
        store.projection_resource(&source).await?.map(|row| row.key),
        Some(source)
    );
    assert_eq!(
        store.projection_resource(&target).await?.map(|row| row.key),
        Some(target)
    );
    pool.close().await;
    Ok(())
}

#[tokio::test]
pub(super) async fn given_a_higher_priority_target_facet_when_a_direct_edge_uses_shadowed_target_data_then_no_edge_is_projected()
-> TestResult {
    let (directory, store, _) = declared_branch_fixture(&["detail", "list"]).await?;
    let source = resource("thing-1")?;
    let target = resource("thing-2")?;
    // Both authorities must be declared in the checked catalog; root bindings
    // alone do not create independent facets.
    let branch_work =
        |sweep: &str, branch: &str, dependency: Option<&str>| -> TestResult<WorkSpec> {
            use inventory_core::record::{FieldId, Key, Record, Value};
            let mut fields = std::collections::BTreeMap::new();
            if let Some(value) = dependency {
                fields.insert(Key::Field(FieldId::new(3)?), Value::Text(value.to_owned()));
            }
            Ok(WorkSpec::new(
                SweepId::new(sweep.to_owned())?,
                scope("us-east-1")?.with_branch(branch.to_owned())?,
                None,
                Record::encode(&Value::Map(fields))?.into_bytes(),
            )?)
        };
    store
        .seed_work(branch_work("higher-target", "detail", None)?)
        .await?;
    let lease = store
        .claim(&SweepId::new("higher-target".to_owned())?, 100, 20)
        .await?
        .ok_or("missing higher-priority target lease")?;
    assert_eq!(
        store
            .commit_page(page(
                lease,
                vec![observation("thing-2", "gamma")?.with_facet_precedence(100)],
                true,
                None,
                101,
            ))
            .await?
            .stats
            .resources_inserted,
        1
    );

    store
        .seed_work(branch_work("shadowed-target", "list", Some("beta"))?)
        .await?;
    let lease = store
        .claim(&SweepId::new("shadowed-target".to_owned())?, 200, 20)
        .await?
        .ok_or("missing lower-priority edge lease")?;
    let mut lower = page(
        lease,
        vec![
            observation("thing-1", "alpha")?,
            observation("thing-2", "beta")?,
            observation("thing-3", "beta")?,
        ],
        true,
        None,
        201,
    );
    lower.edges.push(Edge {
        from: source.clone(),
        to: target.clone(),
        relation: Relation::parse("depends-on")?,
        source: EdgeSource::ReviewedRule,
    });
    assert_eq!(store.commit_page(lower).await?.stats.resources_inserted, 2);
    let effective = store
        .projection_resource(&target)
        .await?
        .ok_or("higher-priority target disappeared")?;
    let higher_hash = effective.content_hash;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    let value: String = sqlx::query_scalar("SELECT f_1 FROM thing_table WHERE id='thing-2'")
        .fetch_one(&pool)
        .await?;
    assert_eq!(value, "gamma");
    let facets: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resource_facets WHERE id='thing-2'")
        .fetch_one(&pool)
        .await?;
    assert_eq!(
        facets, 2,
        "the test must retain independent detail and list facets"
    );
    pool.close().await;
    assert_eq!(
        store.projection_edges_for(&source, None, 10).await?,
        inventory_engine::inventory::ProjectionEdgePage {
            edges: Vec::new(),
            next: None
        }
    );
    drop(store);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(
        reopened.projection_edges_for(&source, None, 10).await?,
        inventory_engine::inventory::ProjectionEdgePage {
            edges: Vec::new(),
            next: None
        }
    );
    let effective = reopened
        .projection_resource(&target)
        .await?
        .ok_or("reopened target disappeared")?;
    assert_eq!(effective.content_hash, higher_hash);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    let expected_edge = inventory_engine::inventory::ProjectionEdgeHeader {
        from: source.clone(),
        to: target.clone(),
        relation: Relation::parse("depends-on")?,
        source: "reviewed-rule".to_owned(),
    };
    for (sweep, at, expected_value, expected_edges) in [
        ("higher-miss-one", 301, "gamma", Vec::new()),
        ("higher-miss-two", 401, "beta", vec![expected_edge.clone()]),
    ] {
        reopened
            .seed_work(branch_work(sweep, "detail", None)?)
            .await?;
        let lease = reopened
            .claim(&SweepId::new(sweep.to_owned())?, at - 1, 20)
            .await?
            .ok_or("missing higher-priority retirement lease")?;
        reopened
            .commit_page(page(lease, Vec::new(), true, None, at))
            .await?;
        let value: String = sqlx::query_scalar("SELECT f_1 FROM thing_table WHERE id='thing-2'")
            .fetch_one(&pool)
            .await?;
        assert_eq!(value, expected_value);
        assert_eq!(
            reopened.projection_edges_for(&source, None, 10).await?,
            inventory_engine::inventory::ProjectionEdgePage {
                edges: expected_edges,
                next: None
            }
        );
    }
    let facets: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resource_facets WHERE id='thing-2'")
        .fetch_one(&pool)
        .await?;
    assert_eq!(
        facets, 1,
        "retiring detail must preserve the lower list facet"
    );
    pool.close().await;
    drop(reopened);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(
        reopened.projection_edges_for(&source, None, 10).await?,
        inventory_engine::inventory::ProjectionEdgePage {
            edges: vec![expected_edge],
            next: None
        }
    );
    Ok(())
}
