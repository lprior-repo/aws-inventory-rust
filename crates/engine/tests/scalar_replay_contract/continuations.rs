use super::*;
use inventory_engine::inventory::PageCommit;
use sha2::Digest;
use std::collections::BTreeMap;

pub(super) fn checkpoint() -> TestResult<Vec<u8>> {
    // The exact same canonical native tuple is deliberately reused across scopes/sweeps.
    Ok(Record::encode(&Value::Map(BTreeMap::from([
        (
            Key::Field(FieldId::new(1)?),
            Value::Text("shared-cursor".to_owned()),
        ),
        (Key::Field(FieldId::new(2)?), Value::Int(7)),
    ])))?
    .into_bytes())
}

pub(super) fn digest() -> TestResult<Vec<u8>> {
    Ok(sha2::Sha256::digest(checkpoint()?).to_vec())
}

pub(super) fn nonfinal_page(
    lease: WorkLease,
    observations: Vec<Observation>,
    at: i64,
) -> TestResult<PageCommit> {
    Ok(support::page(
        lease,
        observations,
        false,
        Some(checkpoint()?),
        at,
    ))
}

pub(super) async fn claim_pair(
    store: &BinaryStore,
    sweep: &str,
    now: i64,
) -> TestResult<[WorkLease; 2]> {
    // Claim both before committing: a nonfinal first page becomes immediately claimable again.
    let east = claim(store, sweep, now).await?;
    let west = claim(store, sweep, now).await?;
    assert_eq!(east.scope(), &support::scope("us-east-1")?);
    assert_eq!(west.scope(), &support::scope("us-west-2")?);
    Ok([east, west])
}

pub(super) fn encoded_scope(state: &StoredState, region: &str) -> TestResult<Vec<u8>> {
    Ok(state
        .scopes
        .iter()
        .find(|row| row.1 == region && row.3 == ID)
        .ok_or("persisted regional scalar scope missing")?
        .4
        .clone())
}

fn assert_work(
    state: &StoredState,
    sweep: &str,
    scope: &[u8],
    token: Option<Vec<u8>>,
    status: i64,
    generation: i64,
    deadline: i64,
) {
    let actual = state
        .work
        .iter()
        .filter(|row| row.0 == sweep && row.1 == scope)
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        vec![(
            sweep.to_owned(),
            scope.to_vec(),
            token,
            vec![1, 2],
            status,
            generation,
            deadline,
            Vec::new()
        )]
    );
}

async fn commit_shared_pages(
    store: &BinaryStore,
    sweep: &str,
    now: i64,
    value: &str,
    name: &str,
    expected: ChangeStats,
) -> TestResult {
    let leases = claim_pair(store, sweep, now).await?;
    for lease in leases {
        assert_eq!(lease.bindings(), &[1, 2]);
        assert!(lease.context().is_empty());
        let observation = observation(lease.scope().region().as_str(), value, name)?;
        assert_eq!(
            store
                .commit_page(nonfinal_page(
                    lease,
                    vec![observation],
                    now.checked_add(1).ok_or("test timestamp overflow")?
                )?)
                .await?,
            CommitResult {
                stats: expected,
                next_generation: Some(1),
                repeated_token: false,
            }
        );
    }
    Ok(())
}

async fn finish_resumed_regions(
    store: &BinaryStore,
    pool: &SqlitePool,
    east_scope: &[u8],
    west_scope: &[u8],
) -> TestResult {
    let [east, west] = claim_pair(store, "history", 200).await?;
    for lease in [&east, &west] {
        assert_eq!(lease.generation(), 4);
        assert_eq!(lease.token(), Some(checkpoint()?.as_slice()));
        assert_eq!(lease.bindings(), &[1, 2]);
    }
    assert_eq!(
        store
            .commit_page(nonfinal_page(
                east,
                vec![observation("us-east-1", "off", "Disabled")?],
                201
            )?)
            .await?,
        CommitResult {
            stats: ChangeStats {
                resources_changed: 1,
                ..ChangeStats::default()
            },
            next_generation: None,
            repeated_token: true,
        }
    );
    let partial = stored_state(pool).await?;
    assert_eq!(
        partial.outcomes,
        vec![(
            "history".to_owned(),
            east_scope.to_vec(),
            "partial".to_owned(),
            201
        )]
    );
    assert_work(&partial, "history", east_scope, None, 2, 4, 0);
    assert_work(
        &partial,
        "history",
        west_scope,
        Some(checkpoint()?),
        1,
        4,
        220,
    );
    assert_scalar(store, pool, "us-east-1", "off", "Disabled", 201, OFF_CBOR).await?;
    assert_scalar(store, pool, "us-west-2", "on", "Enabled", 101, ON_CBOR).await?;
    assert_eq!(
        store
            .commit_page(support::page(
                west,
                vec![observation("us-west-2", "off", "Disabled")?],
                true,
                None,
                201
            ))
            .await?,
        CommitResult {
            stats: ChangeStats {
                resources_changed: 1,
                ..ChangeStats::default()
            },
            next_generation: Some(4),
            repeated_token: false,
        }
    );
    let completed = stored_state(pool).await?;
    assert_eq!(
        completed.outcomes,
        vec![
            (
                "history".to_owned(),
                east_scope.to_vec(),
                "partial".to_owned(),
                201
            ),
            (
                "history".to_owned(),
                west_scope.to_vec(),
                "complete".to_owned(),
                201
            ),
        ]
    );
    assert_work(&completed, "history", west_scope, None, 2, 4, 0);
    assert_eq!(
        store
            .sweep_summary(&SweepId::new("history".to_owned())?)
            .await?,
        SweepSummary {
            complete_scopes: 1,
            partial_scopes: 1,
            resources_observed: 2,
            ..SweepSummary::default()
        }
    );
    Ok(())
}

async fn accept_same_cursor_in_later_sweep(
    store: &BinaryStore,
    pool: &SqlitePool,
    east: &[u8],
    west: &[u8],
) -> TestResult {
    seed_two_regions(store, "later").await?;
    commit_shared_pages(
        store,
        "later",
        300,
        "off",
        "Disabled",
        ChangeStats::default(),
    )
    .await?;
    let state = stored_state(pool).await?;
    assert_eq!(state.resources.len(), 2);
    assert_eq!(
        state.tokens,
        vec![
            ("history".to_owned(), east.to_vec(), digest()?),
            ("history".to_owned(), west.to_vec(), digest()?),
            ("later".to_owned(), east.to_vec(), digest()?),
            ("later".to_owned(), west.to_vec(), digest()?),
        ]
    );
    for scope in [east, west] {
        assert_work(&state, "history", scope, None, 2, 4, 0);
        assert_work(&state, "later", scope, Some(checkpoint()?), 0, 1, 0);
    }
    let leases = claim_pair(store, "later", 310).await?;
    for lease in leases {
        assert_eq!(lease.generation(), 2);
        assert_eq!(lease.token(), Some(checkpoint()?.as_slice()));
        let row = observation(lease.scope().region().as_str(), "off", "Disabled")?;
        assert_eq!(
            store
                .commit_page(support::page(lease, vec![row], true, None, 311))
                .await?,
            CommitResult {
                stats: ChangeStats::default(),
                next_generation: Some(2),
                repeated_token: false,
            }
        );
    }
    assert_eq!(
        store
            .sweep_summary(&SweepId::new("later".to_owned())?)
            .await?,
        SweepSummary {
            complete_scopes: 2,
            resources_observed: 2,
            ..SweepSummary::default()
        }
    );
    let finished = stored_state(pool).await?;
    assert_eq!(
        finished
            .outcomes
            .iter()
            .filter(|row| row.0 == "later")
            .cloned()
            .collect::<Vec<_>>(),
        vec![
            (
                "later".to_owned(),
                east.to_vec(),
                "complete".to_owned(),
                311
            ),
            (
                "later".to_owned(),
                west.to_vec(),
                "complete".to_owned(),
                311
            ),
        ]
    );
    Ok(())
}

#[tokio::test]
async fn given_equal_native_cursors_in_two_regions_when_reopened_resumed_and_reused_then_only_same_scope_repetition_is_partial()
-> TestResult {
    // Given: both regions insert identical nonfinal native tuples in one database/sweep.
    // Kills HISTORY-SWEEP-WIDE and HISTORY-GLOBAL, whose second insert would conflict.
    let (directory, store) = create_store().await?;
    seed_two_regions(&store, "history").await?;
    commit_shared_pages(
        &store,
        "history",
        100,
        "on",
        "Enabled",
        ChangeStats {
            resources_inserted: 1,
            ..ChangeStats::default()
        },
    )
    .await?;
    let pool = read_pool(&directory).await?;
    let initial = stored_state(&pool).await?;
    let east = encoded_scope(&initial, "us-east-1")?;
    let west = encoded_scope(&initial, "us-west-2")?;
    assert_ne!(east, west);
    assert_eq!(
        initial.tokens,
        vec![
            ("history".to_owned(), east.clone(), digest()?),
            ("history".to_owned(), west.clone(), digest()?)
        ]
    );
    assert_work(&initial, "history", &east, Some(checkpoint()?), 0, 1, 0);
    assert_work(&initial, "history", &west, Some(checkpoint()?), 0, 1, 0);
    assert_eq!(initial.outcomes, Vec::new());
    assert_scalar(&store, &pool, "us-east-1", "on", "Enabled", 101, ON_CBOR).await?;
    assert_scalar(&store, &pool, "us-west-2", "on", "Enabled", 101, ON_CBOR).await?;
    let inflight = claim_pair(&store, "history", 110).await?;
    assert_eq!(inflight.map(|lease| lease.generation()), [2, 2]);
    let before_restart = stored_state(&pool).await?;
    pool.close().await;
    drop(store);

    // When: resume both durable in-flight checkpoints, then repeat in only the east.
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let pool = read_pool(&directory).await?;
    assert_eq!(stored_state(&pool).await?, before_restart);
    let sweep = SweepId::new("history".to_owned())?;
    let account = inventory_core::inventory::AccountId::parse("111111111111")?;
    store
        .create_sweep_manifest(&sweep, account.as_str())
        .await?;
    store.resume_sweep(&sweep, &account).await?;
    finish_resumed_regions(&store, &pool, &east, &west).await?;

    // Then: the west completes normally and both scopes accept the digest in a later sweep.
    accept_same_cursor_in_later_sweep(&store, &pool, &east, &west).await?;
    let finished = stored_state(&pool).await?;
    pool.close().await;
    drop(store);
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let pool = read_pool(&directory).await?;
    assert_eq!(stored_state(&pool).await?, finished);
    // No active sweep to resume.
    store
        .resume_sweep(
            &SweepId::new("dummy".to_owned())?,
            &inventory_core::inventory::AccountId::parse("111111111111")?,
        )
        .await?;
    pool.close().await;
    drop(store);
    directory.close()?;
    Ok(())
}
