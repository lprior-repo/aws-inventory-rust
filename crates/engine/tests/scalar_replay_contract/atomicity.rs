use super::*;
use inventory_core::inventory::{Edge, EdgeSource, ReferenceTarget, Relation};
use inventory_engine::inventory::PageCommit;

const TARGET: &str = "arn:aws:example::123456789012:setting/target";
const SECOND: &str = "arn:aws:example::123456789012:setting/second";
const ABSENT: &str = "arn:aws:example::123456789012:setting/absent";

fn resource(id: &str) -> TestResult<ResourceKey> {
    let mut resource = key("us-east-1")?;
    resource.id = ResourceId::parse(id)?;
    Ok(resource)
}

fn native_observation(id: &str, value: ValueRef<'_>) -> TestResult<Observation> {
    Ok(Observation::new(
        resource(id)?,
        scalar_record(value)?,
        &Exclusions::none(),
        None,
    )?)
}

async fn related_fixture() -> TestResult<(tempfile::TempDir, BinaryStore)> {
    let (directory, store) = create_store().await?;
    store
        .seed_works(&[
            support::relationship_work(
                "initial",
                "us-east-1",
                Some("missing-target"),
                Some("target"),
            )?,
            support::relationship_work(
                "initial",
                "us-west-2",
                Some("missing-target"),
                Some("target"),
            )?,
        ])
        .await?;
    let east = claim(&store, "initial", 100).await?;
    assert_eq!(east.scope(), &support::scope("us-east-1")?);
    assert_eq!(
        store
            .commit_page(scalar_page(east, "on", "Enabled", "original", 101)?)
            .await?,
        inserted()
    );
    let west = claim(&store, "initial", 100).await?;
    assert_eq!(west.scope(), &support::scope("us-west-2")?);
    assert_eq!(
        store
            .commit_page(scalar_page(west, "on", "Enabled", "original", 101)?)
            .await?,
        inserted()
    );
    store
        .seed_work(support::relationship_work(
            "relations",
            "us-east-1",
            Some("missing-target"),
            Some("target"),
        )?)
        .await?;
    let lease = claim(&store, "relations", 200).await?;
    let mut page = support::page(
        lease,
        vec![
            native_observation(TARGET, ValueRef::Text("target"))?,
            observation("us-east-1", "on", "Enabled")?,
        ],
        true,
        None,
        201,
    );
    page.edges.push(Edge {
        from: key("us-east-1")?,
        to: resource(TARGET)?,
        relation: Relation::parse("depends-on")?,
        source: EdgeSource::ReviewedRule,
    });
    let source = key("us-east-1")?;
    let bytes = support::scalar_catalog_bytes(ScalarType::Text)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let schema = catalog
        .schemas()
        .iter()
        .find(|schema| schema.kind() == &source.kind)
        .ok_or("checked scalar source schema absent")?;
    let mut reference = support::unresolved(ID, "missing-target")?;
    reference.target = ReferenceTarget::root_text(
        source.account,
        source.region,
        schema,
        ResourceId::parse("missing-target")?,
    )?;
    page.unresolved.push(reference);
    assert_eq!(
        store.commit_page(page).await?,
        CommitResult {
            stats: ChangeStats {
                resources_inserted: 1,
                edges_changed: 1,
                ..ChangeStats::default()
            },
            next_generation: Some(1),
            repeated_token: false,
        }
    );
    Ok((directory, store))
}

async fn assert_seeded_relations(pool: &SqlitePool) -> TestResult {
    let edges: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT from_id,to_id,relation,source FROM edges ORDER BY from_id,to_id,relation,source",
    )
    .fetch_all(pool)
    .await?;
    assert_eq!(
        edges,
        vec![(
            ID.to_owned(),
            TARGET.to_owned(),
            "depends-on".to_owned(),
            "reviewed-rule".to_owned()
        )]
    );
    let unresolved: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT from_id,target_value,relation,source FROM unresolved_refs ORDER BY from_id,target_value,relation,source",
    ).fetch_all(pool).await?;
    assert_eq!(
        unresolved,
        vec![
            (
                ID.to_owned(),
                "missing-target".to_owned(),
                "references".to_owned(),
                "reviewed-rule".to_owned()
            ),
            (
                ID.to_owned(),
                "target".to_owned(),
                "depends-on".to_owned(),
                "reviewed-rule".to_owned()
            ),
        ]
    );
    Ok(())
}

fn changed_page(lease: WorkLease, second: ValueRef<'_>) -> TestResult<PageCommit> {
    let mut page = continuations::nonfinal_page(
        lease,
        vec![
            observation("us-east-1", "off", "Disabled")?,
            native_observation(SECOND, second)?,
        ],
        301,
    )?;
    page.tags.push(TagObservation::new(
        key("us-east-1")?,
        TagCoverage::Complete,
        vec![Tag::new("source", "replacement")?],
    )?);
    Ok(page)
}

#[tokio::test]
async fn given_changed_text_prefix_when_later_native_bool_hits_text_schema_then_entire_page_rolls_back_and_same_lease_can_retry()
-> TestResult {
    let (directory, store) = related_fixture().await?;
    store
        .seed_work(support::relationship_work(
            "late-scalar",
            "us-east-1",
            Some("missing-target"),
            Some("target"),
        )?)
        .await?;
    let lease = claim(&store, "late-scalar", 300).await?;
    let pool = read_pool(&directory).await?;
    assert_seeded_relations(&pool).await?;
    let before = stored_state(&pool).await?;

    assert!(matches!(
        store
            .commit_page(changed_page(lease.clone(), ValueRef::Bool(true))?)
            .await,
        Err(StoreError::Invalid(
            "record scalar does not match text schema"
        ))
    ));

    assert_eq!(stored_state(&pool).await?, before);
    pool.close().await;
    drop(store);
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let pool = read_pool(&directory).await?;
    assert_eq!(stored_state(&pool).await?, before);
    assert_seeded_relations(&pool).await?;
    assert_eq!(
        store
            .commit_page(changed_page(lease, ValueRef::Text("corrected"))?)
            .await?,
        CommitResult {
            stats: ChangeStats {
                resources_changed: 1,
                resources_inserted: 1,
                ..ChangeStats::default()
            },
            next_generation: Some(1),
            repeated_token: false,
        }
    );
    assert_scalar(&store, &pool, "us-east-1", "off", "Disabled", 301, OFF_CBOR).await?;
    let second: (String, String, i64, i64) = sqlx::query_as(
        "SELECT value,typeof(value),first_seen,last_changed FROM thing_table WHERE id=?1",
    )
    .bind(SECOND)
    .fetch_one(&pool)
    .await?;
    assert_eq!(
        second,
        ("corrected".to_owned(), "text".to_owned(), 301, 301)
    );
    assert_eq!(
        store
            .projection_resource(&resource(SECOND)?)
            .await?
            .map(|row| row.key),
        Some(resource(SECOND)?)
    );
    pool.close().await;
    drop(store);
    directory.close()?;
    Ok(())
}

fn changed_tags_page(lease: WorkLease) -> TestResult<PageCommit> {
    let mut page = continuations::nonfinal_page(
        lease,
        vec![observation("us-east-1", "off", "Disabled")?],
        301,
    )?;
    page.tags = vec![
        TagObservation::new(
            key("us-east-1")?,
            TagCoverage::Complete,
            vec![Tag::new("source", "replacement")?],
        )?,
        TagObservation::new(resource(TARGET)?, TagCoverage::Partial, Vec::new())?,
    ];
    Ok(page)
}

async fn tag_coverage(
    pool: &SqlitePool,
) -> TestResult<Vec<(String, String, String, i64, Option<i64>)>> {
    Ok(sqlx::query_as("SELECT region,id,coverage,observed_at,last_complete_at FROM tag_observations ORDER BY region,id")
        .fetch_all(pool).await?)
}

#[tokio::test]
async fn given_earlier_tag_values_and_coverage_updates_when_later_owner_is_absent_then_exact_error_rolls_back_every_table()
-> TestResult {
    let (directory, store) = related_fixture().await?;
    store
        .seed_work(support::relationship_work(
            "late-tags",
            "us-east-1",
            Some("missing-target"),
            Some("target"),
        )?)
        .await?;
    let lease = claim(&store, "late-tags", 300).await?;
    let pool = read_pool(&directory).await?;
    assert_seeded_relations(&pool).await?;
    assert_eq!(
        tag_coverage(&pool).await?,
        vec![
            (
                "us-east-1".to_owned(),
                ID.to_owned(),
                "complete".to_owned(),
                101,
                Some(101)
            ),
            (
                "us-west-2".to_owned(),
                ID.to_owned(),
                "complete".to_owned(),
                101,
                Some(101)
            ),
        ]
    );
    assert_eq!(store.projection_resource(&resource(ABSENT)?).await?, None);
    let before = stored_state(&pool).await?;

    let mut invalid = changed_tags_page(lease.clone())?;
    invalid.tags.push(TagObservation::new(
        resource(ABSENT)?,
        TagCoverage::Complete,
        vec![Tag::new("source", "absent")?],
    )?);
    assert!(matches!(
        store.commit_page(invalid).await,
        Err(StoreError::Invalid(
            "tag observation has no persisted resource"
        ))
    ));

    assert_eq!(stored_state(&pool).await?, before);
    pool.close().await;
    drop(store);
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let pool = read_pool(&directory).await?;
    assert_eq!(stored_state(&pool).await?, before);
    assert_eq!(
        store.commit_page(changed_tags_page(lease)?).await?,
        CommitResult {
            stats: ChangeStats {
                resources_changed: 1,
                ..ChangeStats::default()
            },
            next_generation: Some(1),
            repeated_token: false,
        }
    );
    assert_eq!(
        tag_coverage(&pool).await?,
        vec![
            (
                "us-east-1".to_owned(),
                ID.to_owned(),
                "complete".to_owned(),
                301,
                Some(301)
            ),
            (
                "us-east-1".to_owned(),
                TARGET.to_owned(),
                "partial".to_owned(),
                301,
                None
            ),
            (
                "us-west-2".to_owned(),
                ID.to_owned(),
                "complete".to_owned(),
                101,
                Some(101)
            ),
        ]
    );
    let source: String = sqlx::query_scalar(
        "SELECT tag_value FROM tags WHERE region='us-east-1' AND id=?1 AND tag_key='source'",
    )
    .bind(ID)
    .fetch_one(&pool)
    .await?;
    assert_eq!(source, "replacement");
    pool.close().await;
    drop(store);
    directory.close()?;
    Ok(())
}
