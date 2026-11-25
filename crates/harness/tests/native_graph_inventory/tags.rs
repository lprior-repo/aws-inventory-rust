#[path = "tag_commit.rs"]
pub(super) mod tag_commit;
use super::{
    TestResult,
    compare::{self, Expected},
    native::{self, NativeGraph},
    runtime::NativeRuntime,
};
use cmdb_view::{Database, GraphConnection};
use inventory_core::{
    inventory::ResourceKey,
    record::{Key, Value},
};
use inventory_engine::inventory::{BinaryStore, run_projection};
use inventory_harness::inventory_seed::{
    FieldPath, SeedInput, SeedInputValue, SeedScenario, SeedStep, StepId,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(super) const TAG_KEY: &str = "inventory-native-'\\-key";
const ORIGINAL: &str = "shared-'\\-value\nretained-line";
const UPDATED: &str = "updated-'\\-value";
const PROVENANCE: &str = "inventory.tags";
pub(super) type TagSets = BTreeMap<ResourceKey, BTreeMap<String, String>>;

pub(super) fn targets(expected: &Expected, subnet: &ResourceKey) -> TestResult<Vec<ResourceKey>> {
    let vpcs: Vec<_> = expected
        .resources
        .iter()
        .filter(|key| key.kind.as_str() == "AWS::EC2::VPC")
        .cloned()
        .collect();
    assert_eq!(
        vpcs.len(),
        3,
        "shared EC2, ELBV2 and EKS recipes each create a VPC"
    );
    assert!(expected.resources.contains(subnet));
    assert_eq!(subnet.kind.as_str(), "AWS::EC2::Subnet");
    Ok(vpcs.into_iter().take(2).chain([subnet.clone()]).collect())
}

pub(super) async fn create(native: &NativeRuntime, targets: &[ResourceKey]) -> TestResult {
    mutate(native, "CreateTags", targets, ORIGINAL).await
}

pub(super) async fn replace_and_remove(
    native: &NativeRuntime,
    targets: &[ResourceKey],
) -> TestResult {
    let [removed, changed, _] = targets else {
        return Err("tag target cardinality differs from three".into());
    };
    mutate(
        native,
        "DeleteTags",
        std::slice::from_ref(removed),
        ORIGINAL,
    )
    .await?;
    mutate(native, "CreateTags", std::slice::from_ref(changed), UPDATED).await
}

async fn mutate(
    native: &NativeRuntime,
    operation: &str,
    targets: &[ResourceKey],
    value: &str,
) -> TestResult {
    let resources = targets
        .iter()
        .map(|key| Value::Text(key.id.as_str().to_owned()))
        .collect();
    let tag = Value::Map(BTreeMap::from([
        (Key::Name("Key".into()), Value::Text(TAG_KEY.into())),
        (Key::Name("Value".into()), Value::Text(value.into())),
    ]));
    let scenario = SeedScenario::new(vec![SeedStep::auxiliary(
        StepId::new(0),
        "ec2".into(),
        operation.into(),
        vec![
            SeedInput::new(
                FieldPath::new(&["Resources"])?,
                SeedInputValue::Literal(Value::List(resources)),
            ),
            SeedInput::new(
                FieldPath::new(&["Tags"])?,
                SeedInputValue::Literal(Value::List(vec![tag])),
            ),
        ],
    )])?;
    let oracle = native.seed(scenario).await?;
    assert_eq!(
        oracle.resources.len(),
        0,
        "tag mutation must not invent resource identities"
    );
    assert_eq!(
        oracle.edges.len(),
        0,
        "tags are affiliations, not AWS dependency edges"
    );
    Ok(())
}

fn expected_tags(targets: &[ResourceKey], initial: bool) -> TestResult<TagSets> {
    let [removed, changed, retained] = targets else {
        return Err("tag target cardinality differs from three".into());
    };
    let mut tags = BTreeMap::from([
        (
            changed.clone(),
            BTreeMap::from([(
                TAG_KEY.into(),
                if initial { ORIGINAL } else { UPDATED }.into(),
            )]),
        ),
        (
            retained.clone(),
            BTreeMap::from([(TAG_KEY.into(), ORIGINAL.into())]),
        ),
    ]);
    if initial {
        tags.insert(
            removed.clone(),
            BTreeMap::from([(TAG_KEY.into(), ORIGINAL.into())]),
        );
    }
    Ok(tags)
}

pub(super) async fn assert_all(
    database: &Path,
    graph: &NativeGraph,
    expected: &Expected,
    targets: &[ResourceKey],
    initial: bool,
) -> TestResult {
    let tags = expected_tags(targets, initial)?
        .into_iter()
        .filter(|(key, _)| expected.resources.contains(key))
        .collect();
    assert_sets(database, graph, expected, &tags).await
}

pub(super) async fn assert_sets(
    database: &Path,
    graph: &NativeGraph,
    expected: &Expected,
    tags: &TagSets,
) -> TestResult {
    let tags = &expected.with_provider_tags(tags);
    let associations: BTreeSet<Vec<String>> = tags
        .iter()
        .flat_map(|(resource, tags)| {
            tags.iter().map(move |(key, value)| {
                compare::key_fields(resource)
                    .into_iter()
                    .chain([key.clone(), value.clone(), PROVENANCE.into()])
                    .collect()
            })
        })
        .collect();
    let rows = graph
        .rows(inventory_graph::cypher::diagnostics::AFFILIATIONS)
        .await?;
    let count = rows.len();
    assert_eq!(
        rows.into_iter().collect::<BTreeSet<_>>(),
        associations,
        "exact native scoped tag affiliations"
    );
    assert_eq!(
        count,
        associations.len(),
        "exact native tag affiliation count"
    );
    let vertices: BTreeSet<Vec<String>> = tags
        .iter()
        .flat_map(|(resource, tags)| {
            tags.iter().map(move |(key, value)| {
                vec![
                    resource.account.as_str().into(),
                    resource.region.as_str().into(),
                    key.clone(),
                    value.clone(),
                ]
            })
        })
        .collect();
    let rows = graph
        .rows(inventory_graph::cypher::diagnostics::TAG_ROWS)
        .await?;
    assert_eq!(
        rows.len(),
        vertices.len(),
        "no duplicate or orphan native tag vertices"
    );
    assert_eq!(
        rows.into_iter().collect::<BTreeSet<_>>(),
        vertices,
        "exact scoped native tag vertex set"
    );
    assert_total_topology(graph, expected, associations.len(), vertices.len()).await?;
    assert_view_tags(database, graph, expected, tags).await
}

async fn assert_total_topology(
    graph: &NativeGraph,
    expected: &Expected,
    associations: usize,
    vertices: usize,
) -> TestResult {
    let nodes = graph
        .rows(inventory_graph::cypher::diagnostics::NODE_LABELS)
        .await?;
    assert_eq!(
        nodes.len(),
        expected
            .resources
            .len()
            .checked_add(vertices)
            .ok_or("native node count overflow")?,
        "no unexpected native node labels"
    );
    assert!(
        nodes
            .iter()
            .all(|row| matches!(row.as_slice(), [label] if label == "Resource" || label == "Tag"))
    );
    let relationships = graph
        .rows(inventory_graph::cypher::diagnostics::RELATION_TYPES)
        .await?;
    assert_eq!(
        relationships.len(),
        expected
            .edges
            .len()
            .checked_add(associations)
            .ok_or("native relationship count overflow")?,
        "no quadratic tag clique or extra graph relationship"
    );
    Ok(())
}

async fn assert_view_tags(
    database: &Path,
    graph: &NativeGraph,
    expected: &Expected,
    tags: &TagSets,
) -> TestResult {
    let database = Database::open(database.to_str().ok_or("non-UTF-8 database path")?).await?;
    let graph_reader = GraphConnection::connect(graph.endpoint(), graph.name())?;
    for resource in &expected.resources {
        let node = compare::node(resource);
        let detail = database
            .detail(&node)
            .await?
            .ok_or("UI tag detail missing")?;
        assert!(!detail.tags_truncated, "fixture tags must fit UI page");
        let expected = tags.get(resource).cloned().unwrap_or_default();
        assert_eq!(
            detail.tags.len(),
            expected.len(),
            "UI SQL tags duplicate/lost: {resource:?}"
        );
        assert_eq!(
            detail
                .tags
                .into_iter()
                .map(|tag| (tag.key, tag.value))
                .collect::<BTreeMap<_, _>>(),
            expected,
            "UI SQL tag set: {resource:?}"
        );
        let affiliations = graph_reader.tag_affiliations(&node, 1000).await?;
        assert!(!affiliations.truncated);
        assert_eq!(
            affiliations.affiliations.len(),
            expected.len(),
            "UI native tag cardinality: {resource:?}"
        );
        let actual = affiliations
            .affiliations
            .into_iter()
            .map(|affiliation| {
                assert_eq!(affiliation.provenance, PROVENANCE);
                assert_eq!(affiliation.tag.account, resource.account.as_str());
                assert_eq!(affiliation.tag.region, resource.region.as_str());
                (affiliation.tag.key, affiliation.tag.value)
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(actual, expected, "UI native tag set: {resource:?}");
    }
    assert_tag_filters(&database, tags).await
}

async fn assert_tag_filters(database: &Database, tags: &TagSets) -> TestResult {
    use cmdb_view::{PageRequest, ResourceFilter, TagFilter};
    for value in [
        None,
        Some(ORIGINAL),
        Some(UPDATED),
        Some("missing-native-value"),
    ] {
        let filter = ResourceFilter {
            tag: Some(TagFilter {
                key: TAG_KEY.into(),
                value: value.map(str::to_owned),
            }),
            ..ResourceFilter::default()
        };
        let expected: BTreeSet<_> = tags
            .iter()
            .filter_map(|(resource, tags)| {
                tags.get(TAG_KEY)
                    .filter(|stored| value.is_none_or(|requested| stored.as_str() == requested))
                    .map(|_| resource.clone())
            })
            .collect();
        let page = database
            .page(
                &filter,
                PageRequest {
                    limit: 1000,
                    after: None,
                },
            )
            .await?;
        assert!(
            page.next.is_none(),
            "fixture tag filter must fit one public page"
        );
        assert_eq!(
            page.resources.len(),
            expected.len(),
            "public SQL tag filter cardinality: {value:?}"
        );
        assert_eq!(
            page.resources
                .into_iter()
                .map(|row| row.key)
                .collect::<BTreeSet<_>>(),
            expected,
            "public SQL exact tag filter resource set: {value:?}"
        );
    }
    Ok(())
}

pub(super) async fn replay(
    store: &BinaryStore,
    graph: &NativeGraph,
    database: &Path,
    expected: &Expected,
    targets: &[ResourceKey],
    initial: bool,
) -> TestResult {
    let backend = inventory_graph::falkor::FalkorDb::connect(graph.endpoint(), graph.name())?;
    let report = run_projection(store, &backend, 1000).await?;
    assert_eq!(
        report.acknowledged, 0,
        "fully acked replay has no pending mutations"
    );
    assert_eq!(report.pending, 0);
    assert!(!report.offline);
    assert_all(database, graph, expected, targets, initial).await
}

pub(super) async fn assert_tag_only_refresh(
    store: &BinaryStore,
    graph: &NativeGraph,
    database: &Path,
    expected: &Expected,
    targets: &[ResourceKey],
) -> TestResult<TagSets> {
    let target = targets.last().ok_or("no retained target")?;
    let before = store
        .projection_resource(target)
        .await?
        .ok_or("tag-only target missing")?;
    let view = Database::open(database.to_str().ok_or("non-UTF-8 database path")?).await?;
    let detail_before = view
        .detail(&compare::node(target))
        .await?
        .ok_or("tag-only public detail missing")?;
    tag_commit::replace(store, target, Vec::new(), "native-tag-empty", 300).await?;
    assert!(
        store.projection_state().await?.pending > 0,
        "tag-only removal must publish even without content changes"
    );
    let old_ack = super::recovery::old_ack(store, target).await?;
    super::recovery::deliver_without_ack(store, graph, database).await?;
    let mut tags = expected_tags(targets, false)?;
    tags.remove(target);
    assert_sets(database, graph, expected, &tags).await?;
    assert_observation(
        database,
        target,
        Some((
            inventory_core::inventory::TagCoverage::Complete,
            300,
            Some(300),
        )),
    )
    .await?;
    tag_commit::replace(
        store,
        target,
        vec![(TAG_KEY, UPDATED)],
        "native-tag-superseding",
        400,
    )
    .await?;
    tags.insert(
        target.clone(),
        BTreeMap::from([(TAG_KEY.into(), UPDATED.into())]),
    );
    super::recovery::assert_superseded(database, old_ack).await?;
    super::recovery::offline(store, database, graph).await?;
    native::project(store, graph).await?;
    assert_sets(database, graph, expected, &tags).await?;
    assert_observation(
        database,
        target,
        Some((
            inventory_core::inventory::TagCoverage::Complete,
            400,
            Some(400),
        )),
    )
    .await?;
    let after = store
        .projection_resource(target)
        .await?
        .ok_or("tag-only target removed resource")?;
    assert_eq!(
        after.content_hash, before.content_hash,
        "tag-only update must not rewrite record identity"
    );
    let detail_after = Database::open(database.to_str().ok_or("non-UTF-8 database path")?)
        .await?
        .detail(&compare::node(target))
        .await?
        .ok_or("tag-only public detail disappeared")?;
    assert_eq!(
        detail_after.record.as_bytes(),
        detail_before.record.as_bytes(),
        "tag-only update must preserve exact canonical record bytes"
    );
    assert_eq!(
        detail_after.name, detail_before.name,
        "tag-only update must preserve independent resource name"
    );
    compare::all(database, store, graph, expected).await?;
    let bucket = expected
        .resources
        .iter()
        .find(|key| key.kind.as_str() == "AWS::S3::Bucket")
        .ok_or("shared17 has no global scope resource")?;
    tag_commit::replace(
        store,
        bucket,
        vec![(TAG_KEY, UPDATED)],
        "native-tag-global",
        500,
    )
    .await?;
    native::project(store, graph).await?;
    tags.insert(
        bucket.clone(),
        BTreeMap::from([(TAG_KEY.into(), UPDATED.into())]),
    );
    assert_sets(database, graph, expected, &tags).await?;
    assert_non_authoritative_observations(store, graph, database, expected, &tags, bucket).await?;
    Ok(tags)
}

async fn assert_non_authoritative_observations(
    store: &BinaryStore,
    graph: &NativeGraph,
    database: &Path,
    expected: &Expected,
    tags: &TagSets,
    resource: &ResourceKey,
) -> TestResult {
    use inventory_core::inventory::TagCoverage;
    assert_observation(
        database,
        resource,
        Some((TagCoverage::Complete, 500, Some(500))),
    )
    .await?;
    for (coverage, sweep, at) in [
        (TagCoverage::Partial, "native-tag-partial", 600),
        (TagCoverage::Failed, "native-tag-failed", 700),
        (TagCoverage::Unobserved, "native-tag-unobserved", 800),
    ] {
        tag_commit::observe(store, resource, Some(coverage), Vec::new(), sweep, at).await?;
        assert_eq!(
            store.projection_state().await?.pending,
            0,
            "non-authoritative tags must preserve stored tags without publishing empty snapshots"
        );
        native::project(store, graph).await?;
        assert_sets(database, graph, expected, tags).await?;
        assert_observation(database, resource, Some((coverage, at, Some(500)))).await?;
    }
    tag_commit::observe(store, resource, None, Vec::new(), "native-tag-absent", 900).await?;
    assert_observation(
        database,
        resource,
        Some((TagCoverage::Unobserved, 800, Some(500))),
    )
    .await?;
    assert_sets(database, graph, expected, tags).await?;
    Ok(())
}

pub(super) async fn assert_observation(
    database: &Path,
    resource: &ResourceKey,
    expected: Option<(inventory_core::inventory::TagCoverage, i64, Option<i64>)>,
) -> TestResult {
    let detail = Database::open(database.to_str().ok_or("non-UTF-8 database path")?)
        .await?
        .detail(&compare::node(resource))
        .await?
        .ok_or("coverage target missing from public reader")?;
    assert_eq!(
        detail.tag_observation.map(|row| (
            row.coverage,
            row.observed_at_ms,
            row.last_complete_at_ms
        )),
        expected,
        "exact persisted tag coverage/status/timestamps: {resource:?}"
    );
    Ok(())
}

pub(super) fn now_ms() -> TestResult<i64> {
    Ok(i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis(),
    )?)
}

pub(super) async fn assert_collector_coverage(
    database: &Path,
    targets: &[ResourceKey],
    started: i64,
) -> TestResult {
    let finished = now_ms()?;
    let view = Database::open(database.to_str().ok_or("non-UTF-8 database path")?).await?;
    for target in targets {
        let detail = view
            .detail(&compare::node(target))
            .await?
            .ok_or("collector tag target missing")?;
        let observed = detail
            .tag_observation
            .ok_or("actual collector did not persist tag coverage")?;
        assert_eq!(
            observed.coverage,
            inventory_core::inventory::TagCoverage::Complete
        );
        assert!(
            (started..=finished).contains(&observed.observed_at_ms),
            "collector observation must come from this actual sweep"
        );
        assert_eq!(observed.last_complete_at_ms, Some(observed.observed_at_ms));
        assert_observation(
            database,
            target,
            Some((
                observed.coverage,
                observed.observed_at_ms,
                observed.last_complete_at_ms,
            )),
        )
        .await?;
    }
    Ok(())
}
