#[path = "inventory_store/support.rs"]
mod support;

#[path = "scalar_replay_contract/atomicity.rs"]
mod atomicity;
#[path = "scalar_replay_contract/continuations.rs"]
mod continuations;
#[path = "scalar_replay_contract/relationships.rs"]
mod relationships;
#[path = "scalar_replay_contract/resume.rs"]
mod resume;
#[path = "scalar_replay_contract/snapshot.rs"]
mod snapshot;

use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_core::inventory::{
    AccountId, InventoryError, Observation, ResourceId, ResourceKey, ScalarType, ScopeKey,
};
use inventory_core::record::{
    Exclusions, FieldId, Key, Record, Value, ValueRef, scalar_item, scalar_record,
};
use inventory_engine::inventory::{
    BinaryStore, ChangeStats, CommitResult, PendingProjection, ProjectionAck, ProjectionResource,
    SchemaRegistry, StoreError, SweepId, SweepSummary, Tag, TagCoverage, TagObservation, WorkLease,
};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::sync::Arc;
use support::TestResult;

const ID: &str = "arn:aws:example::123456789012:setting/shared";
const ON_CBOR: &[u8] = &[0xa1, 0x61, 0x24, 0x62, 0x6f, 0x6e];
const OFF_CBOR: &[u8] = &[0xa1, 0x61, 0x24, 0x63, 0x6f, 0x66, 0x66];

async fn create_store() -> TestResult<(tempfile::TempDir, BinaryStore)> {
    let bytes = support::scalar_catalog_bytes(ScalarType::Text)?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let registry = SchemaRegistry::new(&bytes, runtime.schemas().to_vec())?;
    let directory = tempfile::tempdir()?;
    let store = BinaryStore::create(&directory.path().join("inventory.sqlite"), registry).await?;
    Ok((directory, store))
}

async fn read_pool(directory: &tempfile::TempDir) -> TestResult<SqlitePool> {
    Ok(SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(directory.path().join("inventory.sqlite"))
                .read_only(true),
        )
        .await?)
}

fn key(region: &str) -> TestResult<ResourceKey> {
    let mut key = support::resource(ID)?;
    key.region = inventory_core::inventory::Region::parse(region)?;
    Ok(key)
}

fn observation(region: &str, value: &str, name: &str) -> TestResult<Observation> {
    Ok(Observation::new(
        key(region)?,
        scalar_record(ValueRef::Text(value))?,
        &Exclusions::none(),
        Some(Arc::from(name)),
    )?)
}

fn projection(region: &str, value: &str, name: &str) -> TestResult<ProjectionResource> {
    let expected = observation(region, value, name)?;
    Ok(ProjectionResource {
        key: expected.key().clone(),
        name: Some(name.to_owned()),
        content_hash: *expected.content_hash().as_bytes(),
    })
}

async fn claim(store: &BinaryStore, sweep: &str, now: i64) -> TestResult<WorkLease> {
    store
        .claim(&SweepId::new(sweep.to_owned())?, now, 20)
        .await?
        .ok_or_else(|| "expected one claimable work item".into())
}

fn scalar_page(
    lease: WorkLease,
    value: &str,
    name: &str,
    source: &str,
    now: i64,
) -> TestResult<inventory_engine::inventory::PageCommit> {
    let region = lease.scope().region().as_str();
    let row = observation(region, value, name)?;
    let tag = TagObservation::new(
        row.key().clone(),
        TagCoverage::Complete,
        vec![Tag::new("source", source)?],
    )?;
    let mut page = support::page(lease, vec![row], true, None, now);
    page.tags.push(tag);
    Ok(page)
}

fn inserted() -> CommitResult {
    CommitResult {
        stats: ChangeStats {
            resources_inserted: 1,
            ..ChangeStats::default()
        },
        next_generation: Some(1),
        repeated_token: false,
    }
}

async fn seed_two_regions(store: &BinaryStore, sweep: &str) -> TestResult {
    let works = [
        support::work(sweep, "us-east-1")?,
        support::work(sweep, "us-west-2")?,
    ];
    store.seed_works(&works).await?;
    store.seed_works(&works).await?;
    Ok(())
}

async fn commit_two_regions(store: &BinaryStore, sweep: &str) -> TestResult {
    seed_two_regions(store, sweep).await?;
    let east = claim(store, sweep, 100).await?;
    assert_eq!(east.scope(), &support::scope("us-east-1")?);
    assert_eq!(
        store
            .commit_page(scalar_page(east, "on", "Enabled", "original", 101)?)
            .await?,
        inserted()
    );
    let west = claim(store, sweep, 100).await?;
    assert_eq!(west.scope(), &support::scope("us-west-2")?);
    assert_eq!(
        store
            .commit_page(scalar_page(west, "on", "Enabled", "original", 101)?)
            .await?,
        inserted()
    );
    Ok(())
}

type TypedRow = (
    String,
    String,
    String,
    String,
    String,
    Vec<u8>,
    i64,
    i64,
    Vec<u8>,
);
type ResourceRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    Vec<u8>,
    i64,
    i64,
);
type ScopeRow = (String, String, String, String, Vec<u8>, i64, i64, String);
type WorkRow = (
    String,
    Vec<u8>,
    Option<Vec<u8>>,
    Vec<u8>,
    i64,
    i64,
    i64,
    Vec<u8>,
);
type ParentRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    Vec<u8>,
);

#[derive(Debug, PartialEq, Eq)]
struct StoredState {
    typed: Vec<TypedRow>,
    resources: Vec<ResourceRow>,
    scopes: Vec<ScopeRow>,
    work: Vec<WorkRow>,
    parents: Vec<ParentRow>,
    tags: Vec<(String, String, String, String, String, String)>,
    outcomes: Vec<(String, Vec<u8>, String, i64)>,
    tokens: Vec<(String, Vec<u8>, Vec<u8>)>,
    outbox: Vec<(Vec<u8>, i64, i64, i64)>,
    metadata: Vec<(String, Vec<u8>)>,
    durable: snapshot::DurableState,
}

async fn stored_state(pool: &SqlitePool) -> TestResult<StoredState> {
    Ok(StoredState {
        typed: sqlx::query_as("SELECT account,region,id,value,typeof(value),content_hash,first_seen,last_changed,record FROM thing_table ORDER BY account,region,id").fetch_all(pool).await?,
        resources: sqlx::query_as("SELECT account,region,kind,id,name,content_hash,first_seen,last_changed FROM resources ORDER BY account,region,kind,id").fetch_all(pool).await?,
        scopes: sqlx::query_as("SELECT account,region,kind,id,scope_key,last_seen,missing_complete,observed_sweep FROM resource_scopes ORDER BY account,region,kind,id,scope_key").fetch_all(pool).await?,
        work: sqlx::query_as("SELECT sweep_id,scope_key,token,bindings,state,generation,lease_until,context FROM work_queue ORDER BY sweep_id,scope_key").fetch_all(pool).await?,
        parents: sqlx::query_as("SELECT parent_account,parent_region,parent_kind,parent_id,child_account,child_region,child_kind,child_id,scope_key FROM resource_parents ORDER BY parent_account,parent_region,parent_kind,parent_id,child_account,child_region,child_kind,child_id,scope_key").fetch_all(pool).await?,
        tags: sqlx::query_as("SELECT account,region,kind,id,tag_key,tag_value FROM tags ORDER BY account,region,kind,id,tag_key").fetch_all(pool).await?,
        outcomes: sqlx::query_as("SELECT sweep_id,scope_key,outcome,observed_at FROM scope_outcomes ORDER BY sweep_id,scope_key").fetch_all(pool).await?,
        tokens: sqlx::query_as("SELECT sweep_id,scope_key,digest FROM work_tokens ORDER BY sweep_id,scope_key,digest").fetch_all(pool).await?,
        outbox: sqlx::query_as("SELECT resource_key,revision,deleted,full_rebuild FROM projection_outbox ORDER BY resource_key").fetch_all(pool).await?,
        metadata: sqlx::query_as("SELECT key,CAST(value AS BLOB) FROM metadata ORDER BY key").fetch_all(pool).await?,
        durable: snapshot::read_durable_state(pool).await?,
    })
}

async fn assert_scalar(
    store: &BinaryStore,
    pool: &SqlitePool,
    region: &str,
    value: &str,
    name: &str,
    changed: i64,
    cbor: &[u8],
) -> TestResult {
    assert_eq!(
        store.projection_resource(&key(region)?).await?,
        Some(projection(region, value, name)?)
    );
    let row: TypedRow = sqlx::query_as("SELECT account,region,id,value,typeof(value),content_hash,first_seen,last_changed,record FROM thing_table WHERE account=?1 AND region=?2 AND id=?3")
        .bind("123456789012").bind(region).bind(ID).fetch_one(pool).await?;
    assert_eq!(
        row,
        (
            "123456789012".to_owned(),
            region.to_owned(),
            ID.to_owned(),
            value.to_owned(),
            "text".to_owned(),
            projection(region, value, name)?.content_hash.to_vec(),
            101,
            changed,
            cbor.to_vec(),
        )
    );
    let decoded = Record::from_bytes(row.8)?;
    assert_eq!(
        scalar_item(decoded.root())?
            .ok_or("selected scalar missing")?
            .to_value()?,
        Value::Text(value.to_owned())
    );
    Ok(())
}

#[tokio::test]
async fn given_same_canonical_scalar_key_in_two_regions_when_reopened_and_reobserved_then_rows_stay_separate_and_idempotent()
-> TestResult {
    // Given: two scopes deliberately share the same canonical singleton identity.
    let (directory, store) = create_store().await?;
    commit_two_regions(&store, "initial").await?;
    drop(store);

    // When: reopen the actual SQLite file, then replay identical observations in a new sweep.
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let pool = read_pool(&directory).await?;
    assert_scalar(&store, &pool, "us-east-1", "on", "Enabled", 101, ON_CBOR).await?;
    assert_scalar(&store, &pool, "us-west-2", "on", "Enabled", 101, ON_CBOR).await?;
    let pending = store.drain_projection(10).await?;
    assert_eq!(
        pending,
        vec![
            PendingProjection {
                key: Some(key("us-east-1")?),
                revision: 1,
                deleted: false,
                full_rebuild: false
            },
            PendingProjection {
                key: Some(key("us-west-2")?),
                revision: 2,
                deleted: false,
                full_rebuild: false
            },
        ]
    );
    store
        .ack_projection(ProjectionAck::new(key("us-east-1")?, 1)?)
        .await?;
    store
        .ack_projection(ProjectionAck::new(key("us-west-2")?, 2)?)
        .await?;
    let before = stored_state(&pool).await?;
    seed_two_regions(&store, "replay").await?;
    for region in ["us-east-1", "us-west-2"] {
        let lease = claim(&store, "replay", 200).await?;
        assert_eq!(lease.scope(), &support::scope(region)?);
        let mut page = scalar_page(lease, "on", "Enabled", "original", 201)?;
        page.observations
            .push(observation(region, "on", "Enabled")?);
        assert_eq!(
            store.commit_page(page).await?,
            CommitResult {
                stats: ChangeStats::default(),
                next_generation: Some(1),
                repeated_token: false,
            }
        );
    }

    // Then: work seeding and duplicate observations do not create rows or projection revisions.
    let after = stored_state(&pool).await?;
    assert_eq!(after.typed, before.typed);
    assert_eq!(after.resources, before.resources);
    assert_eq!(after.tags, before.tags);
    assert_eq!(after.metadata, before.metadata);
    assert_eq!(after.scopes.len(), 2);
    assert_eq!(after.work.len(), 4);
    assert_eq!(
        after
            .scopes
            .iter()
            .map(|row| (row.5, row.6, row.7.as_str()))
            .collect::<Vec<_>>(),
        vec![(201, 0, "replay"), (201, 0, "replay")]
    );
    assert_eq!(store.drain_projection(10).await?, Vec::new());
    assert_eq!(
        store
            .sweep_summary(&SweepId::new("replay".to_owned())?)
            .await?,
        SweepSummary {
            complete_scopes: 2,
            resources_observed: 2,
            ..SweepSummary::default()
        }
    );
    assert_eq!(
        store
            .claim(&SweepId::new("replay".to_owned())?, 300, 20)
            .await?
            .map(|lease| lease.generation()),
        None
    );
    pool.close().await;
    drop(store);
    directory.close()?;
    Ok(())
}

#[tokio::test]
async fn given_regional_scalar_rows_when_one_value_and_source_change_then_reopened_projection_and_cbor_update_only_that_scope()
-> TestResult {
    // Given: equal canonical IDs are persisted independently in two regions.
    let (directory, store) = create_store().await?;
    commit_two_regions(&store, "initial").await?;
    let pool = read_pool(&directory).await?;
    let before = stored_state(&pool).await?;
    pool.close().await;
    drop(store);

    // When: after reopen, only the east value, display name, and source tag change.
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    store
        .seed_work(support::work("change", "us-east-1")?)
        .await?;
    let lease = claim(&store, "change", 200).await?;
    assert_eq!(
        store
            .commit_page(scalar_page(lease, "off", "Disabled", "replacement", 201)?)
            .await?,
        CommitResult {
            stats: ChangeStats {
                resources_changed: 1,
                ..ChangeStats::default()
            },
            next_generation: Some(1),
            repeated_token: false,
        }
    );
    drop(store);

    // Then: SQLite typed rows, canonical CBOR, projection names/hashes and provenance agree.
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let pool = read_pool(&directory).await?;
    assert_scalar(&store, &pool, "us-east-1", "off", "Disabled", 201, OFF_CBOR).await?;
    assert_scalar(&store, &pool, "us-west-2", "on", "Enabled", 101, ON_CBOR).await?;
    let after = stored_state(&pool).await?;
    assert_eq!(after.typed.len(), 2);
    assert_eq!(after.typed.get(1), before.typed.get(1));
    assert_eq!(after.resources.get(1), before.resources.get(1));
    assert_eq!(after.scopes.get(1), before.scopes.get(1));
    assert_eq!(
        after
            .tags
            .iter()
            .map(|row| (row.1.as_str(), row.4.as_str(), row.5.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("us-east-1", "source", "replacement"),
            ("us-west-2", "source", "original"),
        ]
    );
    assert_eq!(
        store.projection_resources(None, 10).await?,
        vec![
            projection("us-east-1", "off", "Disabled")?,
            projection("us-west-2", "on", "Enabled")?
        ]
    );
    assert_eq!(
        store.drain_projection(10).await?,
        vec![
            PendingProjection {
                key: Some(key("us-west-2")?),
                revision: 2,
                deleted: false,
                full_rebuild: false
            },
            PendingProjection {
                key: Some(key("us-east-1")?),
                revision: 3,
                deleted: false,
                full_rebuild: false
            },
        ]
    );
    pool.close().await;
    drop(store);
    directory.close()?;
    Ok(())
}

#[tokio::test]
async fn given_inflight_scalar_work_when_reopened_and_resumed_then_old_lease_cannot_overwrite_reclaimed_observation()
-> TestResult {
    // Given: durable scalar rows and an unfinished next-sweep lease.
    let (directory, store) = create_store().await?;
    commit_two_regions(&store, "initial").await?;
    store
        .seed_work(support::work("inflight", "us-east-1")?)
        .await?;
    let old = claim(&store, "inflight", 200).await?;
    assert_eq!(old.generation(), 1);
    drop(store);

    // When: restart fencing invalidates the old lease and a new worker commits a change.
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(
        store.resume_sweep().await?,
        Some(SweepId::new("inflight".to_owned())?)
    );
    let pool = read_pool(&directory).await?;
    let before = stored_state(&pool).await?;
    assert!(matches!(
        store
            .commit_page(scalar_page(old.clone(), "off", "Stale", "stale", 201)?)
            .await,
        Err(StoreError::StaleLease)
    ));
    assert_eq!(stored_state(&pool).await?, before);
    let current = claim(&store, "inflight", 200).await?;
    assert_eq!(current.generation(), 3);
    assert_eq!(current.scope(), old.scope());
    assert_eq!(
        store
            .commit_page(scalar_page(
                current.clone(),
                "off",
                "Disabled",
                "current",
                201
            )?)
            .await?,
        CommitResult {
            stats: ChangeStats {
                resources_changed: 1,
                ..ChangeStats::default()
            },
            next_generation: Some(3),
            repeated_token: false,
        }
    );

    // Then: replaying either the completed lease or its predecessor has no durable side effects.
    let committed = stored_state(&pool).await?;
    for lease in [old, current] {
        assert!(matches!(
            store
                .commit_page(scalar_page(lease, "on", "Stale", "stale", 202)?)
                .await,
            Err(StoreError::StaleLease)
        ));
        assert_eq!(stored_state(&pool).await?, committed);
    }
    assert_scalar(&store, &pool, "us-east-1", "off", "Disabled", 201, OFF_CBOR).await?;
    assert_scalar(&store, &pool, "us-west-2", "on", "Enabled", 101, ON_CBOR).await?;
    pool.close().await;
    drop(store);
    directory.close()?;
    Ok(())
}

#[tokio::test]
async fn given_scalar_page_with_foreign_region_when_committed_then_exact_scope_error_leaves_all_state_unchanged()
-> TestResult {
    // Given: a live east lease over existing regional scalar rows.
    let (directory, store) = create_store().await?;
    commit_two_regions(&store, "initial").await?;
    store
        .seed_work(support::work("bounded", "us-east-1")?)
        .await?;
    let lease = claim(&store, "bounded", 200).await?;
    let pool = read_pool(&directory).await?;
    let before = stored_state(&pool).await?;

    // When: a page includes both a legitimate change and an out-of-region observation.
    let mut invalid = scalar_page(lease.clone(), "off", "Disabled", "replacement", 201)?;
    invalid
        .observations
        .push(observation("us-west-2", "off", "Foreign")?);
    assert!(matches!(
        store.commit_page(invalid).await,
        Err(StoreError::Invalid(
            "page contains data outside its leased account and region"
        ))
    ));

    // Then: no row, work generation, timestamp, source tag, or projection revision changes.
    assert_eq!(stored_state(&pool).await?, before);
    assert_eq!(
        store
            .commit_page(scalar_page(lease, "off", "Disabled", "replacement", 201)?)
            .await?,
        CommitResult {
            stats: ChangeStats {
                resources_changed: 1,
                ..ChangeStats::default()
            },
            next_generation: Some(1),
            repeated_token: false,
        }
    );
    assert_scalar(&store, &pool, "us-west-2", "on", "Enabled", 101, ON_CBOR).await?;
    pool.close().await;
    drop(store);
    directory.close()?;
    Ok(())
}

#[tokio::test]
async fn given_scalar_parent_from_another_account_when_work_scope_is_constructed_then_parent_account_error_preserves_store()
-> TestResult {
    // Given: stored scalar identities and a foreign-account parent candidate.
    let (directory, store) = create_store().await?;
    commit_two_regions(&store, "initial").await?;
    let pool = read_pool(&directory).await?;
    let before = stored_state(&pool).await?;
    let scope = support::scope("us-east-1")?;
    let mut foreign = key("us-east-1")?;
    foreign.account = AccountId::parse("999999999999")?;
    foreign.id = ResourceId::parse("foreign-parent")?;

    // When/Then: the public constructor prevents malformed parent-scoped work.
    assert!(matches!(
        ScopeKey::new(
            scope.account().clone(),
            scope.region().clone(),
            scope.service().clone(),
            scope.operation().clone(),
            Some(foreign),
        ),
        Err(InventoryError::ParentAccount)
    ));
    assert_eq!(stored_state(&pool).await?, before);
    pool.close().await;
    drop(store);
    directory.close()?;
    Ok(())
}

#[tokio::test]
async fn given_parent_scoped_scalar_work_when_resumed_then_fenced_replay_cannot_change_child_or_parent_metadata()
-> TestResult {
    // Given: a real persisted parent and duplicate seeding of its child enumeration.
    let (directory, store) = create_store().await?;
    commit_two_regions(&store, "initial").await?;
    let parent = key("us-east-1")?;
    let work = support::child_work("child", &parent)?;
    store.seed_works(&[work.clone(), work]).await?;
    let old = claim(&store, "child", 200).await?;
    assert_eq!(old.scope().parent(), Some(&parent));
    let mut child = parent.clone();
    child.id = ResourceId::parse("arn:aws:example::123456789012:setting/child")?;
    let child_observation = || -> TestResult<Observation> {
        Ok(Observation::new(
            child.clone(),
            scalar_record(ValueRef::Text("on"))?,
            &Exclusions::none(),
            None,
        )?)
    };
    drop(store);

    // When: resume the persisted parent-specific work and reject its pre-restart lease.
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(
        store.resume_sweep().await?,
        Some(SweepId::new("child".to_owned())?)
    );
    let pool = read_pool(&directory).await?;
    let before = stored_state(&pool).await?;
    assert!(matches!(
        store
            .commit_page(support::page(
                old,
                vec![child_observation()?],
                true,
                None,
                201
            ))
            .await,
        Err(StoreError::StaleLease)
    ));
    assert_eq!(stored_state(&pool).await?, before);
    let lease = claim(&store, "child", 200).await?;
    assert_eq!(lease.scope().parent(), Some(&parent));
    assert_eq!(lease.generation(), 3);
    assert_eq!(lease.bindings(), &[1]);
    assert_eq!(
        store
            .commit_page(support::page(
                lease.clone(),
                vec![child_observation()?, child_observation()?],
                true,
                None,
                201
            ))
            .await?,
        CommitResult {
            stats: ChangeStats {
                resources_inserted: 1,
                ..ChangeStats::default()
            },
            next_generation: Some(3),
            repeated_token: false,
        }
    );

    // Then: exactly one scalar child is linked to the exact parent and persisted scope.
    let after = stored_state(&pool).await?;
    let scope_bytes = before
        .work
        .iter()
        .find(|row| row.0 == "child")
        .ok_or("child work missing")?
        .1
        .clone();
    assert_eq!(lease.scope().branch(), None);
    assert_eq!(lease.scope().context(), None);
    // The execution key stays unchanged across resume, but a parented source
    // owns metadata under the semantic authority of its actual input bindings.
    // Derive that authority rather than pinning an opaque persisted byte blob.
    let mut source_key = scope_bytes;
    source_key.push(1); // Scope extension.
    source_key.extend_from_slice(&0_u32.to_be_bytes()); // No named branch.
    source_key.push(1); // Input authority is present.
    let mut authority = <sha2::Sha256 as sha2::Digest>::new();
    sha2::Digest::update(&mut authority, b"aws-inventory/semantic-authority/v1\0");
    sha2::Digest::update(&mut authority, lease.bindings());
    source_key.extend_from_slice(&sha2::Digest::finalize(authority));
    assert_eq!(
        after.parents,
        vec![(
            parent.account.as_str().to_owned(),
            parent.region.as_str().to_owned(),
            parent.kind.as_str().to_owned(),
            parent.id.as_str().to_owned(),
            child.account.as_str().to_owned(),
            child.region.as_str().to_owned(),
            child.kind.as_str().to_owned(),
            child.id.as_str().to_owned(),
            source_key,
        )]
    );
    let child_record: (String, Vec<u8>, i64, i64) = sqlx::query_as("SELECT value,record,first_seen,last_changed FROM thing_table WHERE account=?1 AND region=?2 AND id=?3")
        .bind(child.account.as_str()).bind(child.region.as_str()).bind(child.id.as_str()).fetch_one(&pool).await?;
    assert_eq!(child_record, ("on".to_owned(), ON_CBOR.to_vec(), 201, 201));
    assert_eq!(
        store.projection_resource(&child).await?,
        Some(ProjectionResource {
            key: child.clone(),
            name: None,
            content_hash: *child_observation()?.content_hash().as_bytes(),
        })
    );
    assert!(matches!(
        store
            .commit_page(support::page(
                lease,
                vec![child_observation()?],
                true,
                None,
                202
            ))
            .await,
        Err(StoreError::StaleLease)
    ));
    assert_eq!(stored_state(&pool).await?, after);
    pool.close().await;
    drop(store);
    directory.close()?;
    Ok(())
}
