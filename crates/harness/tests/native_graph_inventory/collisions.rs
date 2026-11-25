use super::{
    TestResult,
    compare::{self, Expected},
    native::{self, NativeGraph},
    runtime,
    tags::{self, TagSets},
};
use cmdb_view::{Database, GraphConnection, GraphLimits, PageRequest, ResourceFilter, TagFilter};
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    inventory::{
        AccountId, Kind, Observation, OperationId, Region, ResourceId, ResourceKey, ScopeKey,
        ServiceId, TagCoverage,
    },
    record::{Exclusions, Record, Value},
};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkSpec};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(super) async fn create_store(database: &Path) -> TestResult<BinaryStore> {
    let bytes = std::fs::read(runtime::catalog_path())?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let registry = SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?;
    Ok(BinaryStore::create(database, registry).await?)
}

pub(super) async fn assert_scoped_fixture(graph: &mut NativeGraph, directory: &Path) -> TestResult {
    graph
        .select_empty_graph("native_inventory_five_keys")
        .await?;
    let database = directory.join("five-key-inventory.sqlite");
    let store = create_store(&database).await?;
    let keys = keys()?;
    let resources = keys.iter().cloned().collect();
    let expected = Expected {
        resources,
        edges: BTreeSet::new(),
        safe_records: BTreeMap::new(),
    };
    let pairs = baseline(&keys)?;
    let (same_scope, single_scopes) = keys.split_at_checked(2).ok_or("first scope missing")?;
    let groups = std::iter::once(same_scope).chain(single_scopes.chunks(1));
    for (index, group) in groups.enumerate() {
        let sweep = SweepId::new(format!("collision-initial-{index}"))?;
        insert(&store, sweep, group.iter().cloned().map(Ok)).await?;
    }
    initialize_tags(&database, &store, &keys, &pairs).await?;
    native::project(&store, graph).await?;
    assert_fixture(&database, &store, graph, &expected, &pairs).await?;
    assert_tag_count(graph, 5).await?;
    assert_eq!(
        graph
            .rows(inventory_graph::cypher::diagnostics::TAG_PROVENANCE)
            .await?
            .len(),
        6
    );
    let first = keys.first().ok_or("first collision key missing")?;
    assert_tag_replacements(&database, &store, graph, &expected, pairs, first).await
}

async fn assert_tag_count(graph: &NativeGraph, expected: usize) -> TestResult {
    assert_eq!(
        graph
            .rows(inventory_graph::cypher::diagnostics::TAG_KEYS)
            .await?
            .len(),
        expected
    );
    Ok(())
}

async fn assert_tag_replacements(
    database: &Path,
    store: &BinaryStore,
    graph: &NativeGraph,
    expected: &Expected,
    mut pairs: TagSets,
    first: &ResourceKey,
) -> TestResult {
    tags::tag_commit::replace(
        store,
        first,
        vec![("owner", "bob")],
        "collision-replacement",
        200,
    )
    .await?;
    pairs.insert(
        first.clone(),
        BTreeMap::from([("owner".into(), "bob".into())]),
    );
    native::project(store, graph).await?;
    assert_fixture(database, store, graph, expected, &pairs).await?;
    assert_tag_count(graph, 5).await?;
    tags::tag_commit::replace(store, first, Vec::new(), "collision-empty", 300).await?;
    pairs.remove(first);
    native::project(store, graph).await?;
    assert_fixture(database, store, graph, expected, &pairs).await?;
    assert_tag_count(graph, 4).await?;
    tags::tag_commit::replace(
        store,
        first,
        vec![
            ("owner", "alice"),
            ("team", "alice"),
            ("quoted-'\\", "line-one\nline-two"),
        ],
        "collision-keys-and-newline",
        400,
    )
    .await?;
    pairs.insert(
        first.clone(),
        BTreeMap::from([
            ("owner".into(), "alice".into()),
            ("team".into(), "alice".into()),
            ("quoted-'\\".into(), "line-one\nline-two".into()),
        ]),
    );
    native::project(store, graph).await?;
    assert_fixture(database, store, graph, expected, &pairs).await
}

fn keys() -> TestResult<Vec<ResourceKey>> {
    [
        (
            "111111111111",
            Region::parse("us-east-1")?,
            "AWS::EC2::VPC",
            "one",
        ),
        (
            "111111111111",
            Region::parse("us-east-1")?,
            "AWS::EC2::VPC",
            "two",
        ),
        (
            "111111111111",
            Region::parse("us-west-2")?,
            "AWS::EC2::VPC",
            "one",
        ),
        (
            "222222222222",
            Region::parse("us-east-1")?,
            "AWS::EC2::VPC",
            "one",
        ),
        ("111111111111", Region::Global, "AWS::S3::Bucket", "one"),
    ]
    .into_iter()
    .map(|(account, region, kind, id)| {
        Ok(ResourceKey {
            account: AccountId::parse(account)?,
            region,
            kind: Kind::parse(kind)?,
            id: ResourceId::parse(id)?,
        })
    })
    .collect()
}

fn baseline(keys: &[ResourceKey]) -> TestResult<TagSets> {
    let [r1, r2, r3, r4, global] = keys else {
        return Err("literal five-key cardinality changed".into());
    };
    let owner = BTreeMap::from([("owner".into(), "alice".into())]);
    Ok(BTreeMap::from([
        (
            r1.clone(),
            BTreeMap::from([
                ("owner".into(), "alice".into()),
                ("team".into(), "core".into()),
            ]),
        ),
        (r2.clone(), owner.clone()),
        (r3.clone(), owner.clone()),
        (r4.clone(), owner.clone()),
        (global.clone(), owner),
    ]))
}

pub(super) async fn insert(
    store: &BinaryStore,
    sweep: SweepId,
    keys: impl IntoIterator<Item = TestResult<ResourceKey>>,
) -> TestResult {
    let mut keys = keys.into_iter();
    let first = keys.next().ok_or("empty resource scope")??;
    let (service, operation) = if first.region == Region::Global {
        ("s3", "ListBuckets")
    } else {
        ("ec2", "DescribeVpcs")
    };
    let scope = ScopeKey::new(
        first.account.clone(),
        first.region.clone(),
        ServiceId::parse(service)?,
        OperationId::parse(operation)?,
        None,
    )?;
    let record = Record::encode(&Value::Map(BTreeMap::new()))?;
    store
        .seed_work(WorkSpec::new(
            sweep.clone(),
            scope,
            None,
            record.as_bytes().to_vec(),
        )?)
        .await?;
    let lease = store
        .claim(&sweep, 100, 10_000)
        .await?
        .ok_or("fixture scope unclaimable")?;
    let observations = std::iter::once(Ok(first))
        .chain(keys)
        .map(|key| {
            let key = key?;
            let arn = key.id.as_str().into();
            Ok(Observation::new(
                key,
                record.clone(),
                &Exclusions::none(),
                Some(arn),
            )?)
        })
        .collect::<TestResult<Vec<_>>>()?;
    store
        .commit_page(PageCommit {
            lease,
            observations,
            tags: Vec::new(),
            edges: Vec::new(),
            unresolved: Vec::new(),
            next_token: None,
            continuation_digest: None,
            children: Vec::new(),
            done: true,
            observed_at_ms: 100,
        })
        .await?;
    Ok(())
}

async fn initialize_tags(
    database: &Path,
    store: &BinaryStore,
    keys: &[ResourceKey],
    pairs: &TagSets,
) -> TestResult {
    for key in keys {
        tags::assert_observation(database, key, None).await?;
    }
    let first = keys.first().ok_or("absent-observation fixture is empty")?;
    tags::tag_commit::observe(
        store,
        first,
        None,
        Vec::new(),
        "collision-no-observation",
        105,
    )
    .await?;
    tags::assert_observation(database, first, None).await?;
    tags::tag_commit::observe(
        store,
        first,
        Some(TagCoverage::Unobserved),
        Vec::new(),
        "collision-unobserved",
        110,
    )
    .await?;
    tags::assert_observation(database, first, Some((TagCoverage::Unobserved, 110, None))).await?;
    for (index, key) in keys.iter().enumerate() {
        let values = pairs
            .get(key)
            .ok_or("literal baseline tags missing")?
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect();
        tags::tag_commit::replace(
            store,
            key,
            values,
            &format!("collision-complete-{index}"),
            120,
        )
        .await?;
        tags::assert_observation(database, key, Some((TagCoverage::Complete, 120, Some(120))))
            .await?;
    }
    Ok(())
}

async fn assert_fixture(
    database: &Path,
    store: &BinaryStore,
    graph: &NativeGraph,
    expected: &Expected,
    pairs: &TagSets,
) -> TestResult {
    tags::assert_sets(database, graph, expected, pairs).await?;
    let view = Database::open(database.to_str().ok_or("non-UTF-8 database path")?).await?;
    let record = Record::encode(&Value::Map(BTreeMap::new()))?;
    let hash = record.content_hash(&Exclusions::none())?;
    let metadata: BTreeSet<Vec<String>> = expected
        .resources
        .iter()
        .map(|key| {
            compare::key_fields(key)
                .into_iter()
                .chain([
                    key.id.as_str().into(),
                    format!("{:x}", base16ct::HexDisplay(hash.as_bytes())),
                ])
                .collect()
        })
        .collect();
    let rows = graph
        .rows(inventory_graph::cypher::diagnostics::RESOURCE_ROWS)
        .await?;
    assert_eq!(rows.len(), 5);
    assert_eq!(rows.into_iter().collect::<BTreeSet<_>>(), metadata);
    let mut projection = store.begin_projection_snapshot().await?;
    let reader = GraphConnection::connect(graph.endpoint(), graph.name())?;
    for key in &expected.resources {
        let detail = view
            .detail(&compare::node(key))
            .await?
            .ok_or("collision detail missing")?;
        assert_eq!(detail.key, *key);
        assert_eq!(detail.name.as_deref(), Some(key.id.as_str()));
        assert_eq!(detail.record.as_bytes(), record.as_bytes());
        assert_eq!(detail.content_hash, hash);
        assert_projection_pairs(&mut projection, key, pairs.get(key)).await?;
        let dependencies = reader
            .neighbors(
                &compare::node(key),
                GraphLimits {
                    depth: 3,
                    nodes: 500,
                    edges: 2000,
                },
            )
            .await?;
        assert!(
            !dependencies.truncated && dependencies.edges.is_empty(),
            "shared tags are never resource dependencies"
        );
    }
    projection.finish().await?;
    assert_pages(&view, expected, pairs).await
}

async fn assert_projection_pairs(
    snapshot: &mut inventory_engine::inventory::ProjectionSnapshot,
    key: &ResourceKey,
    expected: Option<&BTreeMap<String, String>>,
) -> TestResult {
    let mut cursor = None;
    let mut actual = BTreeMap::new();
    for _ in 0..=100 {
        let page = snapshot.tags_for(key, cursor, 1).await?;
        assert!(page.tags.len() <= 1);
        for tag in page.tags {
            assert!(
                actual.insert(tag.key, tag.value).is_none(),
                "projection pagination repeated a pair"
            );
        }
        cursor = page.next;
        if cursor.is_none() {
            break;
        }
    }
    assert!(
        cursor.is_none(),
        "projection tag pagination did not terminate"
    );
    assert_eq!(actual, expected.cloned().unwrap_or_default());
    Ok(())
}

async fn assert_pages(view: &Database, expected: &Expected, pairs: &TagSets) -> TestResult {
    for (account, region, kind) in [
        (None, None, None),
        (Some("111111111111"), None, None),
        (
            Some("222222222222"),
            Some("us-east-1"),
            Some("AWS::EC2::VPC"),
        ),
        (
            Some("111111111111"),
            Some("us-east-1"),
            Some("AWS::EC2::VPC"),
        ),
        (Some("111111111111"), Some("us-west-2"), None),
        (
            Some("111111111111"),
            Some("global"),
            Some("AWS::S3::Bucket"),
        ),
    ] {
        for value in [None, Some("alice"), Some("bob"), Some("absent")] {
            let filter = ResourceFilter {
                account: account.map(str::to_owned),
                region: region.map(str::to_owned),
                kind: kind.map(str::to_owned),
                tag: Some(TagFilter {
                    key: "owner".into(),
                    value: value.map(str::to_owned),
                }),
                ..ResourceFilter::default()
            };
            let wanted: BTreeSet<_> = expected
                .resources
                .iter()
                .filter(|key| {
                    account.is_none_or(|v| key.account.as_str() == v)
                        && region.is_none_or(|v| key.region.as_str() == v)
                        && kind.is_none_or(|v| key.kind.as_str() == v)
                        && pairs
                            .get(*key)
                            .and_then(|tags| tags.get("owner"))
                            .is_some_and(|tag| value.is_none_or(|v| tag == v))
                })
                .cloned()
                .collect();
            let mut actual = BTreeSet::new();
            let mut after = None;
            for _ in 0..=5 {
                let page = view.page(&filter, PageRequest { limit: 1, after }).await?;
                assert_eq!(page.total_matching, u64::try_from(wanted.len())?);
                assert!(page.resources.len() <= 1);
                for row in page.resources {
                    assert!(actual.insert(row.key), "public page repeated a full key");
                }
                after = page.next;
                if after.is_none() {
                    break;
                }
            }
            assert!(
                after.is_none(),
                "public full-key pagination did not terminate"
            );
            assert_eq!(
                actual, wanted,
                "public account/region/kind/tag page must match independent literal fixture"
            );
        }
    }
    Ok(())
}
