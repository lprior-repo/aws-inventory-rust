use super::*;
use inventory_core::inventory::DiscoveryContext;
use inventory_engine::inventory::WorkSpec;

struct ResumeFixture {
    directory: tempfile::TempDir,
    store: BinaryStore,
    pool: SqlitePool,
    east: Vec<u8>,
    west: Vec<u8>,
    context: Vec<u8>,
}

async fn writable_pool(directory: &tempfile::TempDir) -> TestResult<SqlitePool> {
    Ok(SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?)
}

async fn resume_fixture(west_generation: i64) -> TestResult<ResumeFixture> {
    let (directory, store) = create_store().await?;
    commit_two_regions(&store, "initial").await?;
    let discovery = DiscoveryContext::empty();
    let discriminator = discovery.discriminator()?;
    let context = discovery.encode()?;
    assert_ne!(context, Vec::<u8>::new());
    let works = [
        WorkSpec::new(
            SweepId::new("resume".to_owned())?,
            support::scope("us-east-1")?.with_context(discriminator),
            None,
            vec![1, 2],
        )?
        .with_context(context.clone())?,
        WorkSpec::new(
            SweepId::new("resume".to_owned())?,
            support::scope("us-west-2")?.with_context(discriminator),
            None,
            vec![1, 2],
        )?
        .with_context(context.clone())?,
    ];
    store.seed_works(&works).await?;
    store
        .create_sweep_manifest(&SweepId::new("resume".to_owned())?, "111111111111")
        .await?;
    let leases = claim_pair(&store, &context, 200).await?;
    for lease in leases {
        let row = observation(lease.scope().region().as_str(), "on", "Enabled")?;
        assert_eq!(lease.context(), context);
        assert_eq!(
            store
                .commit_page(continuations::nonfinal_page(lease, vec![row], 201)?)
                .await?,
            CommitResult {
                stats: ChangeStats::default(),
                next_generation: Some(1),
                repeated_token: false,
            }
        );
    }
    let leases = claim_pair(&store, &context, 210).await?;
    assert_eq!(leases.map(|lease| lease.generation()), [2, 2]);
    let pool = read_pool(&directory).await?;
    let state = stored_state(&pool).await?;
    let mut scopes = state.work.iter().filter(|row| row.0 == "resume");
    let east = scopes.next().ok_or("east resume work absent")?.1.clone();
    let west = scopes.next().ok_or("west resume work absent")?.1.clone();
    assert!(scopes.next().is_none());
    pool.close().await;
    drop(store);

    // The public API cannot create an exhausted lease. Alter only this fixture
    // generation after establishing real resources/checkpoints/bindings/history.
    let writer = writable_pool(&directory).await?;
    assert_eq!(sqlx::query("UPDATE work_queue SET generation=?1 WHERE sweep_id='resume' AND scope_key=?2 AND state=1 AND generation=2 AND lease_until=230")
        .bind(west_generation).bind(&west).execute(&writer).await?.rows_affected(), 1);
    writer.close().await;
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let pool = read_pool(&directory).await?;
    Ok(ResumeFixture {
        directory,
        store,
        pool,
        east,
        west,
        context,
    })
}

async fn claim_pair(store: &BinaryStore, context: &[u8], at: i64) -> TestResult<[WorkLease; 2]> {
    let discriminator = DiscoveryContext::decode(context)?.discriminator()?;
    let east = claim(store, "resume", at).await?;
    let west = claim(store, "resume", at).await?;
    assert_eq!(
        east.scope(),
        &support::scope("us-east-1")?.with_context(discriminator)
    );
    assert_eq!(
        west.scope(),
        &support::scope("us-west-2")?.with_context(discriminator)
    );
    Ok([east, west])
}

fn assert_queue(
    state: &StoredState,
    fixture: &ResumeFixture,
    east_generation: i64,
    west_generation: i64,
    status: i64,
    deadline: i64,
    has_token: bool,
) -> TestResult {
    let token = if has_token {
        Some(continuations::checkpoint()?)
    } else {
        None
    };
    assert_eq!(
        state
            .work
            .iter()
            .filter(|row| row.0 == "resume")
            .cloned()
            .collect::<Vec<_>>(),
        vec![
            (
                "resume".to_owned(),
                fixture.east.clone(),
                token.clone(),
                vec![1, 2],
                status,
                east_generation,
                deadline,
                fixture.context.clone()
            ),
            (
                "resume".to_owned(),
                fixture.west.clone(),
                token,
                vec![1, 2],
                status,
                west_generation,
                deadline,
                fixture.context.clone()
            ),
        ]
    );
    assert_eq!(
        state.tokens,
        vec![
            (
                "resume".to_owned(),
                fixture.east.clone(),
                continuations::digest()?
            ),
            (
                "resume".to_owned(),
                fixture.west.clone(),
                continuations::digest()?
            ),
        ]
    );
    assert_eq!(state.resources.len(), 2);
    assert_eq!(state.typed.len(), 2);
    assert_eq!(state.tags.len(), 2);
    Ok(())
}

async fn assert_reopened_state(
    directory: &tempfile::TempDir,
    expected: &StoredState,
) -> TestResult {
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let pool = read_pool(directory).await?;
    assert_eq!(&stored_state(&pool).await?, expected);
    pool.close().await;
    drop(store);
    Ok(())
}

#[tokio::test]
async fn given_ordinary_and_exhausted_inflight_leases_when_resumed_then_exact_overflow_leaves_every_durable_row_unchanged()
-> TestResult {
    // Given: open succeeds with real resource/tag/facet state and nonempty token history/context.
    let fixture = resume_fixture(i64::MAX).await?;
    let before = stored_state(&fixture.pool).await?;
    assert_queue(&before, &fixture, 2, i64::MAX, 1, 230, true)?;

    // When: overflow is detected inside resume_sweep, not opening or claiming the fixture.
    // Kills RESUME-SKIPS-GENERATION-GUARD, including partial release of the ordinary row.
    let sweep = SweepId::new("resume".to_owned())?;
    let account = inventory_core::inventory::AccountId::parse("111111111111")?;
    assert!(matches!(
        fixture.store.resume_sweep(&sweep, &account).await,
        Err(StoreError::LeaseGenerationOverflow)
    ));

    // Then: the ordinary row remains in flight and all persisted tables survive reopen unchanged.
    assert_eq!(stored_state(&fixture.pool).await?, before);
    fixture.pool.close().await;
    drop(fixture.store);
    assert_reopened_state(&fixture.directory, &before).await?;
    fixture.directory.close()?;
    Ok(())
}

async fn add_second_active_sweep(directory: &tempfile::TempDir, scope: &[u8]) -> TestResult {
    let writer = writable_pool(directory).await?;
    let mut tx = writer.begin().await?;
    assert_eq!(sqlx::query("INSERT INTO work_queue(sweep_id,scope_key,token,bindings,state,generation,lease_until,context) SELECT 'other-active',scope_key,token,bindings,state,generation,lease_until,context FROM work_queue WHERE sweep_id='resume' AND scope_key=?1")
        .bind(scope).execute(&mut *tx).await?.rows_affected(), 1);
    assert_eq!(sqlx::query("INSERT INTO work_tokens(sweep_id,scope_key,digest) SELECT 'other-active',scope_key,digest FROM work_tokens WHERE sweep_id='resume' AND scope_key=?1")
        .bind(scope).execute(&mut *tx).await?.rows_affected(), 1);
    tx.commit().await?;
    writer.close().await;
    Ok(())
}

#[tokio::test]
async fn given_two_active_sweeps_when_resumed_then_exact_multiple_active_error_has_no_side_effects()
-> TestResult {
    // Given: a deliberately inconsistent durable queue; public seeding correctly forbids this state.
    let fixture = resume_fixture(2).await?;
    let initial = stored_state(&fixture.pool).await?;
    assert_queue(&initial, &fixture, 2, 2, 1, 230, true)?;
    fixture.pool.close().await;
    drop(fixture.store);
    add_second_active_sweep(&fixture.directory, &fixture.west).await?;
    let store = BinaryStore::open(&fixture.directory.path().join("inventory.sqlite")).await?;
    let pool = read_pool(&fixture.directory).await?;
    let active: Vec<(String, i64)> = sqlx::query_as("SELECT sweep_id,COUNT(*) FROM work_queue WHERE state IN(0,1) GROUP BY sweep_id ORDER BY sweep_id")
        .fetch_all(&pool).await?;
    assert_eq!(
        active,
        vec![("other-active".to_owned(), 1), ("resume".to_owned(), 2)]
    );
    let before = stored_state(&pool).await?;

    // When: Kills RESUME-FIRST-ACTIVE-WINS; choosing either identity must not release its rows.
    let sweep = SweepId::new("resume".to_owned())?;
    let account = inventory_core::inventory::AccountId::parse("111111111111")?;
    assert!(matches!(
        store.resume_sweep(&sweep, &account).await,
        Err(StoreError::MultipleActiveSweeps)
    ));

    // Then: queue generations/deadlines, both sweeps' history, and resource state are unchanged.
    assert_eq!(stored_state(&pool).await?, before);
    pool.close().await;
    drop(store);
    assert_reopened_state(&fixture.directory, &before).await?;
    fixture.directory.close()?;
    Ok(())
}

#[tokio::test]
async fn given_near_max_inflight_generation_when_resumed_and_claimed_then_max_generation_can_still_commit()
-> TestResult {
    // Given: one ordinary lease and one at MAX-2, with genuine persisted checkpoints/history.
    let near_max = i64::MAX.checked_sub(2).ok_or("test generation underflow")?;
    let after_resume = i64::MAX.checked_sub(1).ok_or("test generation underflow")?;
    let fixture = resume_fixture(near_max).await?;
    let before = stored_state(&fixture.pool).await?;
    assert_queue(&before, &fixture, 2, near_max, 1, 230, true)?;

    // When: resume advances once and a real claim advances again to the inclusive maximum.
    let sweep = SweepId::new("resume".to_owned())?;
    let account = inventory_core::inventory::AccountId::parse("111111111111")?;
    fixture.store.resume_sweep(&sweep, &account).await?;
    let resumed = stored_state(&fixture.pool).await?;
    assert_queue(&resumed, &fixture, 3, after_resume, 0, 0, true)?;
    let [east, west] = claim_pair(&fixture.store, &fixture.context, 300).await?;
    assert_eq!([east.generation(), west.generation()], [4, i64::MAX]);
    for lease in [east, west] {
        assert_eq!(lease.token(), Some(continuations::checkpoint()?.as_slice()));
        assert_eq!(lease.bindings(), &[1, 2]);
        assert_eq!(lease.context(), fixture.context);
        let generation = lease.generation();
        let row = observation(lease.scope().region().as_str(), "on", "Enabled")?;
        assert_eq!(
            fixture
                .store
                .commit_page(support::page(lease, vec![row], true, None, 301))
                .await?,
            CommitResult {
                stats: ChangeStats::default(),
                next_generation: Some(generation),
                repeated_token: false,
            }
        );
    }

    // Then: exact resources/history remain; both independent scopes finish without over-rejection.
    let completed = stored_state(&fixture.pool).await?;
    assert_queue(&completed, &fixture, 4, i64::MAX, 2, 0, false)?;
    assert_eq!(completed.resources, before.resources);
    assert_eq!(completed.typed, before.typed);
    assert_eq!(completed.tags, before.tags);
    assert_eq!(completed.outbox, before.outbox);
    assert_eq!(completed.metadata, before.metadata);
    assert_eq!(
        fixture
            .store
            .sweep_summary(&SweepId::new("resume".to_owned())?)
            .await?,
        SweepSummary {
            complete_scopes: 2,
            resources_observed: 2,
            ..SweepSummary::default()
        }
    );
    fixture.pool.close().await;
    drop(fixture.store);
    assert_reopened_state(&fixture.directory, &completed).await?;
    fixture.directory.close()?;
    Ok(())
}
