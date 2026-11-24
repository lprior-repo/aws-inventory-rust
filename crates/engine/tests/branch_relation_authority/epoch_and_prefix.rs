use super::*;

#[derive(Debug, PartialEq, Eq)]
struct Baseline {
    resources: Vec<(String, Vec<u8>, Option<String>)>,
    typed: Vec<(String, String)>,
    owners: Vec<(String, Vec<u8>, i64, String, String)>,
    pending: Vec<(String, Vec<u8>, i64, String)>,
    legacy: Vec<String>,
    refreshes: Vec<(String, Vec<u8>)>,
}

async fn baseline(pool: &SqlitePool) -> TestResult<Baseline> {
    Ok(Baseline {
        resources: sqlx::query_as("SELECT id,content_hash,name FROM resources ORDER BY id")
            .fetch_all(pool).await?,
        typed: sqlx::query_as("SELECT id,f_1 FROM thing_table ORDER BY id")
            .fetch_all(pool).await?,
        owners: sqlx::query_as("SELECT id,scope_key,missing_complete,observed_sweep,completed_sweep FROM resource_scopes ORDER BY id,scope_key")
            .fetch_all(pool).await?,
        pending: sqlx::query_as("SELECT id,scope_key,v3_misses,v3_completed_sweep FROM legacy_authorities ORDER BY id,scope_key")
            .fetch_all(pool).await?,
        legacy: sqlx::query_scalar("SELECT id FROM legacy_resources ORDER BY id")
            .fetch_all(pool).await?,
        refreshes: sqlx::query_as("SELECT id,scope_key FROM legacy_refreshes ORDER BY id,scope_key")
            .fetch_all(pool).await?,
    })
}

async fn reviewed_lease(store: &BinaryStore, sweep: &str, at: i64) -> TestResult<WorkLease> {
    let sweep = SweepId::new(sweep.to_owned())?;
    let bindings = Record::encode(&Value::Map(BTreeMap::from([(
        Key::Field(FieldId::new(2)?),
        Value::Text("emitting-name".to_owned()),
    )])))?
    .into_bytes();
    let scope = scope("us-east-1")?.with_branch("EMIT".to_owned())?;
    store
        .seed_work(WorkSpec::new(sweep.clone(), scope, None, bindings)?)
        .await?;
    store
        .claim(&sweep, at, 1_000)
        .await?
        .ok_or_else(|| "reviewed lease absent".into())
}

async fn seed_baseline(store: &BinaryStore) -> TestResult {
    let lease = reviewed_lease(store, "baseline", 50).await?;
    store
        .commit_page(page(
            lease,
            vec![
                observation("baseline-a", "alpha")?,
                observation("baseline-b", "beta")?,
            ],
            true,
            None,
            51,
        ))
        .await?;
    Ok(())
}

async fn downgrade_empty_relations(pool: &SqlitePool) -> TestResult {
    // With no relationship entries, field 2's empty list is also the exact V2
    // codec: no V3 authority cell needs truncating or fabricating. Rebuild the
    // actual old schema/marker, preserving every durable cursor and lease epoch.
    let relations: Vec<Vec<u8>> = sqlx::query_scalar("SELECT relations FROM resource_facets")
        .fetch_all(pool)
        .await?;
    assert_eq!(relations.len(), 2);
    for bytes in relations {
        let record = Record::from_bytes(bytes)?;
        let Some(inventory_core::record::ValueRef::List(entries)) =
            record.root().get_field(FieldId::new(2)?)?
        else {
            return Err("baseline facet relationship list absent".into());
        };
        assert_eq!(entries.count()?, 0);
    }
    let mut tx = pool.begin().await?;
    for statement in [
        "DROP TABLE reviewed_reference_evidence",
        "DROP TABLE reviewed_reference_grants",
        "CREATE TABLE reviewed_reference_grants(proof BLOB NOT NULL CHECK(length(proof)=32),target_id TEXT CHECK(target_id IS NULL OR length(target_id)>0))",
        "CREATE TABLE reviewed_reference_evidence(proof BLOB NOT NULL CHECK(length(proof)=32),from_account TEXT NOT NULL,from_region TEXT NOT NULL,from_kind TEXT NOT NULL,from_id TEXT NOT NULL,target_id TEXT CHECK(target_id IS NULL OR length(target_id)>0))",
        "ALTER TABLE resource_facets DROP COLUMN source_authority",
        "ALTER TABLE work_queue DROP COLUMN authority_generation",
        "ALTER TABLE work_queue DROP COLUMN authority_complete",
        "DROP TABLE legacy_observations",
        "ALTER TABLE legacy_authorities DROP COLUMN v3_misses",
        "ALTER TABLE legacy_authorities DROP COLUMN v3_completed_sweep",
        "DELETE FROM metadata WHERE key='canonical_facet_dependencies_v3'",
        "INSERT INTO metadata(key,value) VALUES('canonical_facet_dependencies_v2',X'01') ON CONFLICT(key) DO UPDATE SET value=excluded.value",
    ] {
        sqlx::query(statement).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(())
}

#[tokio::test]
async fn given_a_pending_v2_baseline_when_invalid_stage_finishes_failed_or_partial_then_the_sweep_settles_without_certifying_that_baseline()
-> TestResult {
    for outcome in [ScopeOutcome::Failed, ScopeOutcome::Partial] {
        for (branch, binding) in [
            (None, Some("base-name")),
            (Some("UNKNOWN"), Some("base-name")),
            (Some("EMIT"), Some("base-name")),
        ] {
            let (directory, original, pool) = branch_fixture(false).await?;
            seed_baseline(&original).await?;
            drop(original);
            downgrade_empty_relations(&pool).await?;
            let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
            let before = baseline(&pool).await?;
            assert_eq!(before.pending.len(), 2);
            assert_eq!(
                before.legacy,
                vec!["baseline-a".to_owned(), "baseline-b".to_owned()]
            );
            let lease = stage_lease(&store, branch, binding).await?;
            store.finish_scope(lease, outcome, 101, None).await?;
            assert_eq!(baseline(&pool).await?, before);
            let terminal: (i64, i64, String, i64) = sqlx::query_as(
                "SELECT work.state,work.lease_until,outcomes.outcome,(SELECT COUNT(*) FROM work_queue WHERE state<>2) FROM work_queue work JOIN scope_outcomes outcomes USING(sweep_id,scope_key) WHERE work.sweep_id='stage-authority'",
            ).fetch_one(&pool).await?;
            assert_eq!(terminal, (2, 0, outcome.as_str().to_owned(), 0));
            let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
            assert_eq!(baseline(&pool).await?, before);
            assert_eq!(reopened.unattributed_authority_count().await?, 2);
            pool.close().await;
        }
    }
    Ok(())
}

#[tokio::test]
async fn given_a_pre_v3_generation_valid_lease_when_failed_or_partial_is_reported_after_migration_then_the_new_authority_epoch_fences_it()
-> TestResult {
    let (directory, original, pool) = branch_fixture(false).await?;
    seed_baseline(&original).await?;
    let old = stage_lease(&original, Some("EMIT"), Some("emitting-name")).await?;
    drop(original);
    downgrade_empty_relations(&pool).await?;
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let epoch: (i64, i64) = sqlx::query_as(
        "SELECT generation,authority_generation FROM work_queue WHERE sweep_id='stage-authority'",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(epoch, (old.generation(), 0));
    let before = snapshot::read_durable_state(&pool).await?;
    assert!(matches!(
        store
            .commit_page(page(old.clone(), Vec::new(), true, None, 101))
            .await,
        Err(StoreError::StaleLease)
    ));
    assert_eq!(snapshot::read_durable_state(&pool).await?, before);
    for outcome in [ScopeOutcome::Failed, ScopeOutcome::Partial] {
        assert!(matches!(
            store.finish_scope(old.clone(), outcome, 101, None).await,
            Err(StoreError::StaleLease)
        ));
        assert_eq!(snapshot::read_durable_state(&pool).await?, before);
    }
    let sweep = SweepId::new("stage-authority".to_owned())?;
    let account = inventory_core::inventory::AccountId::parse("111111111111")?;
    store
        .create_sweep_manifest(&sweep, account.as_str())
        .await?;
    store.resume_sweep(&sweep, &account).await?;
    let current = store
        .claim(&sweep, 102, 100)
        .await?
        .ok_or("fresh authority claim absent")?;
    assert!(current.generation() > old.generation());
    store
        .finish_scope(current, ScopeOutcome::Partial, 103, None)
        .await?;
    assert_eq!(store.sweep_summary(&sweep).await?.work_pending, 0);
    assert_eq!(store.unattributed_authority_count().await?, 2);
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_a_v2_cursor_with_an_old_head_when_only_the_migrated_tail_completes_then_it_cannot_count_as_a_fresh_full_negative_observation()
-> TestResult {
    let (directory, original, pool) = branch_fixture(false).await?;
    seed_baseline(&original).await?;
    let head = reviewed_lease(&original, "interrupted", 60).await?;
    let cursor = vec![7, 8, 9];
    original
        .commit_page(page(
            head,
            vec![observation("baseline-a", "alpha")?],
            false,
            Some(cursor.clone()),
            61,
        ))
        .await?;
    let old_work: (Option<Vec<u8>>, i64, i64) = sqlx::query_as(
        "SELECT token,generation,state FROM work_queue WHERE sweep_id='interrupted'",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(old_work.0.as_deref(), Some(cursor.as_slice()));
    drop(original);
    downgrade_empty_relations(&pool).await?;
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let migrated_work: (Option<Vec<u8>>, i64, i64) = sqlx::query_as(
        "SELECT token,generation,state FROM work_queue WHERE sweep_id='interrupted'",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(migrated_work, old_work);
    let before = baseline(&pool).await?;
    assert_eq!(before.pending.len(), 2);
    let sweep = SweepId::new("interrupted".to_owned())?;
    let account = inventory_core::inventory::AccountId::parse("111111111111")?;
    store
        .create_sweep_manifest(&sweep, account.as_str())
        .await?;
    store.resume_sweep(&sweep, &account).await?;
    let tail = store
        .claim(&sweep, 100, 100)
        .await?
        .ok_or("tail claim absent")?;
    assert_eq!(tail.token(), Some(cursor.as_slice()));
    let complete: i64 = sqlx::query_scalar(
        "SELECT authority_complete FROM work_queue WHERE sweep_id='interrupted'",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(complete, 0);
    store
        .commit_page(page(tail, Vec::new(), true, None, 101))
        .await?;
    assert_eq!(baseline(&pool).await?, before);
    for (sweep, at) in [("fresh-full-miss-one", 200), ("fresh-full-miss-two", 300)] {
        let lease = reviewed_lease(&store, sweep, at).await?;
        assert_eq!(lease.token(), None);
        store
            .commit_page(page(lease, Vec::new(), true, None, at + 1))
            .await?;
        if at == 200 {
            let misses: Vec<(String, i64)> =
                sqlx::query_as("SELECT id,v3_misses FROM legacy_authorities ORDER BY id")
                    .fetch_all(&pool)
                    .await?;
            assert_eq!(
                misses,
                vec![("baseline-a".to_owned(), 1), ("baseline-b".to_owned(), 1)]
            );
            assert_eq!(store.unattributed_authority_count().await?, 2);
        }
    }
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(reopened.unattributed_authority_count().await?, 0);
    assert!(
        reopened
            .projection_resource(&resource("baseline-a")?)
            .await?
            .is_none()
    );
    assert!(
        reopened
            .projection_resource(&resource("baseline-b")?)
            .await?
            .is_none()
    );
    assert!(reopened.projection_edges(None, 10).await?.edges.is_empty());
    pool.close().await;
    Ok(())
}
