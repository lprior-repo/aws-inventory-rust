#[path = "branch_relation_authority/epoch_and_prefix.rs"]
mod epoch_and_prefix;
#[path = "scalar_replay_contract/snapshot.rs"]
mod snapshot;
#[path = "inventory_store/support.rs"]
#[allow(
    dead_code,
    reason = "shared via #[path]; each test target uses a subset of the helpers"
)]
mod support;

use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    inventory::{Edge, EdgeSource, Relation, ScopeOutcome},
    record::{FieldId, Key, Record, Value},
};
use inventory_engine::inventory::{
    BinaryStore, PageCommit, ProjectionEdgeHeader, ProjectionEdgePage, SchemaRegistry, StoreError,
    SweepId, WorkLease, WorkSpec,
};
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::collections::BTreeMap;
use support::*;

async fn branch_fixture(
    base_discovery: bool,
) -> TestResult<(tempfile::TempDir, BinaryStore, SqlitePool)> {
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("inventory.sqlite");
    let bytes = model_relation_catalog(branch_catalog_bytes(base_discovery)?)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .plans()
        .first()
        .ok_or("synthetic callable plan missing")?;
    assert_eq!(plan.branches().len(), 2);
    let registry = SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?;
    let store = BinaryStore::create(&database, registry).await?;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(database))
        .await?;
    Ok((directory, store, pool))
}

async fn stage_lease(
    store: &BinaryStore,
    branch: Option<&str>,
    binding: Option<&str>,
) -> TestResult<WorkLease> {
    let sweep = SweepId::new("stage-authority".to_owned())?;
    let scope = match branch {
        Some(branch) => scope("us-east-1")?.with_branch(branch.to_owned())?,
        None => scope("us-east-1")?,
    };
    let input = Key::Field(FieldId::new(2)?);
    let fields = binding
        .map(|value| (input, Value::Text(value.to_owned())))
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let bindings = Record::encode(&Value::Map(fields))?.into_bytes();
    store
        .seed_work(WorkSpec::new(sweep.clone(), scope, None, bindings)?)
        .await?;
    store
        .claim(&sweep, 100, 20)
        .await?
        .ok_or_else(|| "stage lease missing".into())
}

fn metadata_page(lease: WorkLease, value: &str, relationship: u8) -> TestResult<PageCommit> {
    let mut commit = page(
        lease,
        vec![
            observation("source", "source-name")?,
            observation("target", value)?,
        ],
        true,
        None,
        101,
    );
    match relationship {
        0 => {}
        1 => commit.unresolved.push(unresolved("source", value)?),
        2 => commit.edges.push(Edge {
            from: resource("source")?,
            to: resource("target")?,
            relation: Relation::parse("references")?,
            source: EdgeSource::Model,
        }),
        _ => return Err("unsupported test relationship case".into()),
    }
    Ok(commit)
}

async fn assert_rejected(
    directory: &tempfile::TempDir,
    store: &BinaryStore,
    pool: &SqlitePool,
    commit: PageCommit,
) -> TestResult {
    let before = snapshot::read_durable_state(pool).await?;
    let result = store.commit_page(commit).await;
    assert!(
        matches!(
            &result,
            Err(StoreError::Invalid(
                "relationship evidence is not permitted by the checked catalog"
            ))
        ),
        "unexpected stage result: {result:?}"
    );
    assert_eq!(snapshot::read_durable_state(pool).await?, before);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert!(
        reopened
            .projection_resource(&resource("source")?)
            .await?
            .is_none()
    );
    assert!(
        reopened
            .projection_resource(&resource("target")?)
            .await?
            .is_none()
    );
    assert_eq!(snapshot::read_durable_state(pool).await?, before);
    Ok(())
}

async fn assert_rejected_completion(
    directory: &tempfile::TempDir,
    store: &BinaryStore,
    pool: &SqlitePool,
    lease: WorkLease,
) -> TestResult {
    let before = snapshot::read_durable_state(pool).await?;
    let result = store
        .finish_scope(lease, ScopeOutcome::Complete, 101, None)
        .await;
    assert!(
        matches!(
            &result,
            Err(StoreError::Invalid(
                "relationship evidence is not permitted by the checked catalog"
            ))
        ),
        "unexpected completion result: {result:?}"
    );
    assert_eq!(snapshot::read_durable_state(pool).await?, before);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let summary = reopened
        .sweep_summary(&SweepId::new("stage-authority".to_owned())?)
        .await?;
    assert_eq!(summary.complete_scopes, 0);
    assert_eq!(snapshot::read_durable_state(pool).await?, before);
    Ok(())
}

async fn assert_exact_edge(store: &BinaryStore, source: &str) -> TestResult {
    assert!(
        store
            .projection_resource(&resource("source")?)
            .await?
            .is_some()
    );
    assert!(
        store
            .projection_resource(&resource("target")?)
            .await?
            .is_some()
    );
    assert_eq!(
        store
            .projection_edges_for(&resource("source")?, None, 10)
            .await?,
        ProjectionEdgePage {
            edges: vec![ProjectionEdgeHeader {
                from: resource("source")?,
                to: resource("target")?,
                relation: Relation::parse("references")?,
                source: source.to_owned(),
            }],
            next: None
        }
    );
    Ok(())
}

#[tokio::test]
async fn given_base_emits_but_exact_stage_is_discovery_when_metadata_is_submitted_then_every_table_is_unchanged()
-> TestResult {
    for relationship in [0, 1, 2] {
        let (directory, store, pool) = branch_fixture(false).await?;
        let lease = stage_lease(&store, Some("DISCOVER"), Some("discovery-name")).await?;
        assert_rejected(
            &directory,
            &store,
            &pool,
            metadata_page(lease, "discovery-name", relationship)?,
        )
        .await?;
        pool.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn given_base_is_discovery_but_exact_stage_emits_when_matching_evidence_is_committed_then_projection_survives_reopen()
-> TestResult {
    for (relationship, source) in [(1, "reviewed-rule"), (2, "model")] {
        let (directory, store, pool) = branch_fixture(true).await?;
        let lease = stage_lease(&store, Some("EMIT"), Some("emitting-name")).await?;
        store
            .commit_page(metadata_page(lease, "emitting-name", relationship)?)
            .await?;
        assert_exact_edge(&store, source).await?;
        let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
        assert_exact_edge(&reopened, source).await?;
        pool.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn given_only_named_stages_exist_when_none_or_an_unknown_stage_submits_metadata_then_no_fallback_authority_is_used()
-> TestResult {
    for branch in [None, Some("UNKNOWN")] {
        for relationship in [0, 1, 2] {
            let (directory, store, pool) = branch_fixture(false).await?;
            let lease = stage_lease(&store, branch, Some("base-name")).await?;
            assert_rejected(
                &directory,
                &store,
                &pool,
                metadata_page(lease, "base-name", relationship)?,
            )
            .await?;
            pool.close().await;
        }
    }
    Ok(())
}

#[tokio::test]
async fn given_an_emitting_stage_when_request_uses_base_other_stage_or_missing_fixed_input_then_full_page_is_rejected()
-> TestResult {
    for binding in [Some("base-name"), Some("discovery-name"), None] {
        for relationship in [0, 1, 2] {
            let (directory, store, pool) = branch_fixture(false).await?;
            let lease = stage_lease(&store, Some("EMIT"), binding).await?;
            assert_rejected(
                &directory,
                &store,
                &pool,
                metadata_page(lease, binding.unwrap_or("emitting-name"), relationship)?,
            )
            .await?;
            pool.close().await;
        }
    }
    Ok(())
}

#[tokio::test]
async fn given_the_exact_emitting_stage_when_reference_claims_another_stage_value_then_evidence_is_not_authorized()
-> TestResult {
    for value in ["base-name", "discovery-name"] {
        for relationship in [1, 2] {
            let (directory, store, pool) = branch_fixture(false).await?;
            let lease = stage_lease(&store, Some("EMIT"), Some("emitting-name")).await?;
            assert_rejected(
                &directory,
                &store,
                &pool,
                metadata_page(lease, value, relationship)?,
            )
            .await?;
            pool.close().await;
        }
    }
    Ok(())
}

#[tokio::test]
async fn given_an_exact_discovery_stage_when_its_page_contains_no_metadata_then_completion_is_still_allowed()
-> TestResult {
    let (directory, store, pool) = branch_fixture(false).await?;
    let lease = stage_lease(&store, Some("DISCOVER"), Some("discovery-name")).await?;
    store
        .commit_page(page(lease, Vec::new(), true, None, 101))
        .await?;
    let summary = store
        .sweep_summary(&SweepId::new("stage-authority".to_owned())?)
        .await?;
    assert_eq!(summary.complete_scopes, 1);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let reopened_summary = reopened
        .sweep_summary(&SweepId::new("stage-authority".to_owned())?)
        .await?;
    assert_eq!(reopened_summary.complete_scopes, 1);
    assert_eq!(reopened_summary.partial_scopes, 0);
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_a_claimed_lease_without_exact_reviewed_stage_authority_when_finished_complete_then_no_durable_state_changes()
-> TestResult {
    for (branch, binding) in [
        (None, Some("base-name")),
        (Some("UNKNOWN"), Some("base-name")),
        (Some("EMIT"), Some("base-name")),
        (Some("EMIT"), Some("discovery-name")),
        (Some("EMIT"), None),
        (Some("DISCOVER"), Some("emitting-name")),
    ] {
        let (directory, store, pool) = branch_fixture(false).await?;
        let lease = stage_lease(&store, branch, binding).await?;
        assert_rejected_completion(&directory, &store, &pool, lease).await?;
        pool.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn given_an_exact_discovery_only_stage_when_finished_complete_then_its_terminal_outcome_survives_reopen()
-> TestResult {
    let (directory, store, pool) = branch_fixture(false).await?;
    let lease = stage_lease(&store, Some("DISCOVER"), Some("discovery-name")).await?;
    let stats = store
        .finish_scope(lease, ScopeOutcome::Complete, 101, None)
        .await?;
    assert_eq!(
        (
            stats.resources_inserted,
            stats.resources_changed,
            stats.resources_deleted,
            stats.edges_changed
        ),
        (0, 0, 0, 0)
    );
    let summary = store
        .sweep_summary(&SweepId::new("stage-authority".to_owned())?)
        .await?;
    assert_eq!(summary.complete_scopes, 1);
    assert_eq!(summary.partial_scopes, 0);
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let summary = reopened
        .sweep_summary(&SweepId::new("stage-authority".to_owned())?)
        .await?;
    assert_eq!(summary.complete_scopes, 1);
    assert_eq!(summary.partial_scopes, 0);
    pool.close().await;
    Ok(())
}

async fn assert_non_authoritative_completion(
    directory: &tempfile::TempDir,
    store: &BinaryStore,
    pool: &SqlitePool,
    lease: WorkLease,
    outcome: ScopeOutcome,
) -> TestResult {
    let stats = store.finish_scope(lease, outcome, 101, None).await?;
    assert_eq!(
        (
            stats.resources_inserted,
            stats.resources_changed,
            stats.resources_deleted,
            stats.edges_changed
        ),
        (0, 0, 0, 0)
    );
    let settled: (i64, i64, String, i64, i64, i64) = sqlx::query_as(
        "SELECT work.state,work.lease_until,outcomes.outcome,\
         (SELECT COUNT(*) FROM work_queue WHERE state<>2),\
         (SELECT COUNT(*) FROM resources),(SELECT COUNT(*) FROM edges) \
         FROM work_queue work JOIN scope_outcomes outcomes USING(sweep_id,scope_key) \
         WHERE work.sweep_id='stage-authority'",
    )
    .fetch_one(pool)
    .await?;
    assert_eq!(settled, (2, 0, outcome.as_str().to_owned(), 0, 0, 0));
    let settled_state = snapshot::read_durable_state(pool).await?;
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let summary = reopened
        .sweep_summary(&SweepId::new("stage-authority".to_owned())?)
        .await?;
    assert_eq!(summary.work_pending, 0);
    assert_eq!(summary.complete_scopes, 0);
    assert_eq!(
        summary.failed_scopes,
        u64::from(outcome == ScopeOutcome::Failed)
    );
    assert_eq!(
        summary.partial_scopes,
        u64::from(outcome == ScopeOutcome::Partial)
    );
    assert_eq!(snapshot::read_durable_state(pool).await?, settled_state);
    Ok(())
}

#[tokio::test]
async fn given_a_claimed_invalid_stage_when_finished_failed_or_partial_then_it_settles_without_metadata_or_complete_authority()
-> TestResult {
    for outcome in [ScopeOutcome::Failed, ScopeOutcome::Partial] {
        for (branch, binding) in [
            (None, Some("base-name")),
            (Some("UNKNOWN"), Some("base-name")),
            (Some("EMIT"), Some("base-name")),
        ] {
            let (directory, store, pool) = branch_fixture(false).await?;
            let lease = stage_lease(&store, branch, binding).await?;
            assert_non_authoritative_completion(&directory, &store, &pool, lease, outcome).await?;
            pool.close().await;
        }
    }
    Ok(())
}
