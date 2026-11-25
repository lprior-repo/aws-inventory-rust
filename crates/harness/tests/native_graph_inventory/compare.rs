use super::{
    TestResult, listed_resources,
    native::NativeGraph,
    runtime::NativeRuntime,
    tags::{TAG_KEY, TagSets},
};
use cmdb_view::{
    Database, EdgePageRequest, GraphConnection, GraphEdge, GraphLimits, GraphNode, PageRequest,
    ResourceFilter,
};
use inventory_core::inventory::ResourceKey;
use inventory_engine::inventory::BinaryStore;
use inventory_harness::inventory_seed::ScenarioSeedOracle;
use std::{collections::BTreeSet, path::Path};

#[derive(Debug)]
pub(super) struct Expected {
    pub(super) resources: BTreeSet<ResourceKey>,
    pub(super) edges: BTreeSet<Vec<String>>,
    pub(super) safe_records: super::safe_fields::Expectations,
    listed: BTreeSet<ResourceKey>,
    listed_edges: BTreeSet<Vec<String>>,
    provider_tags: TagSets,
}

impl Expected {
    pub(super) fn from_oracles(oracles: &[ScenarioSeedOracle]) -> TestResult<Self> {
        assert_eq!(oracles.len(), 17, "all shared recipes must execute");
        let count: usize = oracles.iter().map(|oracle| oracle.resources.len()).sum();
        let resources: BTreeSet<_> = oracles
            .iter()
            .flat_map(|oracle| oracle.resources.iter())
            .map(|resource| resource.key.clone())
            .collect();
        assert_eq!(
            resources.len(),
            count,
            "seed oracle has duplicate full keys"
        );
        let edges = oracles
            .iter()
            .flat_map(|oracle| oracle.edges.iter())
            .map(oracle_edge_fields)
            .collect();
        Ok(Self::new(
            resources,
            edges,
            super::safe_fields::Expectations::new(),
        ))
    }

    pub(super) fn new(
        resources: BTreeSet<ResourceKey>,
        edges: BTreeSet<Vec<String>>,
        safe_records: super::safe_fields::Expectations,
    ) -> Self {
        Self {
            resources,
            edges,
            safe_records,
            listed: BTreeSet::new(),
            listed_edges: BTreeSet::new(),
            provider_tags: TagSets::new(),
        }
    }

    pub(super) async fn refresh_listed(&mut self, native: &NativeRuntime) -> TestResult {
        let listed = listed_resources::read(native).await?;
        self.provider_tags = listed_resources::tags(native).await?;
        let missing: Vec<_> = self
            .resources
            .difference(&self.listed)
            .filter(|key| listed_resources::supports(&key.kind) && !listed.contains_key(*key))
            .collect();
        if !missing.is_empty() {
            return Err(
                format!("independent Describe responses omitted seeded keys: {missing:?}").into(),
            );
        }
        self.listed.retain(|key| {
            if listed.contains_key(key) {
                true
            } else {
                self.resources.remove(key);
                self.safe_records.remove(key);
                false
            }
        });
        self.edges.retain(|edge| !self.listed_edges.contains(edge));
        self.listed_edges.clear();
        for edge in listed_resources::vpc_edges(&listed)?.map(|edge| oracle_edge_fields(&edge)) {
            if self.edges.insert(edge.clone()) {
                self.listed_edges.insert(edge);
            }
        }
        for (key, resource) in listed {
            if !self.resources.contains(&key) {
                self.resources.insert(key.clone());
                self.listed.insert(key.clone());
            } else if !self.listed.contains(&key) {
                continue;
            }
            self.safe_records.insert(key, resource.fields);
        }
        Ok(())
    }

    pub(super) fn with_provider_tags(&self, tags: &TagSets) -> TagSets {
        let mut merged = tags.clone();
        for (resource, provided) in &self.provider_tags {
            if !self.resources.contains(resource) {
                continue;
            }
            let provided: Vec<_> = provided.iter().filter(|(key, _)| *key != TAG_KEY).collect();
            if provided.is_empty() {
                continue;
            }
            let entry = merged.entry(resource.clone()).or_default();
            for (key, value) in provided {
                entry.entry(key.clone()).or_insert_with(|| value.clone());
            }
        }
        merged
    }
}

fn oracle_edge_fields(edge: &inventory_core::inventory::Edge) -> Vec<String> {
    key_fields(&edge.from)
        .into_iter()
        .chain(key_fields(&edge.to))
        .chain([
            edge.relation.as_str().to_owned(),
            edge.source.as_str().to_owned(),
        ])
        .collect()
}

pub(super) fn key_fields(key: &ResourceKey) -> Vec<String> {
    [
        key.account.as_str(),
        key.region.as_str(),
        key.kind.as_str(),
        key.id.as_str(),
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

pub(super) fn node(key: &ResourceKey) -> GraphNode {
    GraphNode {
        account: key.account.as_str().to_owned(),
        region: key.region.as_str().to_owned(),
        kind: key.kind.as_str().to_owned(),
        id: key.id.as_str().to_owned(),
    }
}

fn edge_fields(edge: &GraphEdge) -> Vec<String> {
    [
        &edge.from.account,
        &edge.from.region,
        &edge.from.kind,
        &edge.from.id,
        &edge.to.account,
        &edge.to.region,
        &edge.to.kind,
        &edge.to.id,
        &edge.relation,
        &edge.provenance,
    ]
    .into_iter()
    .cloned()
    .collect()
}

pub(super) async fn all(
    database: &Path,
    store: &BinaryStore,
    graph: &NativeGraph,
    expected: &Expected,
) -> TestResult {
    assert_sql_projection(store, expected).await?;
    assert_native(database, graph, expected).await?;
    assert_read_model(database, graph, expected).await
}

async fn assert_sql_projection(store: &BinaryStore, expected: &Expected) -> TestResult {
    let resources = store.projection_resources(None, 1000).await?;
    assert!(
        resources.len() < 1000,
        "fixture must fit the explicit exact resource budget"
    );
    let count = resources.len();
    let actual = resources
        .into_iter()
        .map(|row| row.key)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        count,
        expected.resources.len(),
        "SQLite resource cardinality; unexpected keys: {:?}; missing keys: {:?}",
        actual.difference(&expected.resources).collect::<Vec<_>>(),
        expected.resources.difference(&actual).collect::<Vec<_>>()
    );
    assert_eq!(
        actual, expected.resources,
        "SQLite full key set must match the independent seed and Describe oracles exactly"
    );
    let page = store.projection_edges(None, 1000).await?;
    assert_eq!(page.next, None, "fixture exceeds exact edge budget");
    let count = page.edges.len();
    let edges: BTreeSet<Vec<String>> = page
        .edges
        .into_iter()
        .map(|edge| {
            key_fields(&edge.from)
                .into_iter()
                .chain(key_fields(&edge.to))
                .chain([edge.relation.as_str().to_owned(), edge.source])
                .collect()
        })
        .collect();
    assert_eq!(count, edges.len(), "duplicate SQLite dependency edge");
    assert_eq!(
        edges, expected.edges,
        "SQLite exact reviewed dependency set"
    );
    Ok(())
}

async fn assert_native(database: &Path, graph: &NativeGraph, expected: &Expected) -> TestResult {
    let rows = graph
        .rows(inventory_graph::cypher::diagnostics::NODE_IDENTITIES)
        .await?;
    let resources: BTreeSet<Vec<String>> = expected
        .resources
        .iter()
        .map(|key| {
            key_fields(key)
                .into_iter()
                .chain([format!(
                    "{:x}",
                    base16ct::HexDisplay(key.digest().as_bytes())
                )])
                .collect()
        })
        .collect();
    assert_eq!(
        rows.len(),
        resources.len(),
        "native resource count includes isolated nodes"
    );
    assert_eq!(
        rows.into_iter().collect::<BTreeSet<_>>(),
        resources,
        "native resource full keys and production digest must match the independent oracles"
    );
    let view = Database::open(database.to_str().ok_or("non-UTF-8 database path")?).await?;
    let mut metadata = BTreeSet::new();
    for key in &expected.resources {
        let detail = view
            .detail(&node(key))
            .await?
            .ok_or("metadata resource missing")?;
        assert_eq!(detail.name, None, "fixture display-name contract: {key:?}");
        metadata.insert(
            key_fields(key)
                .into_iter()
                .chain([format!(
                    "{:x}",
                    base16ct::HexDisplay(detail.content_hash.as_bytes())
                )])
                .collect::<Vec<_>>(),
        );
    }
    let rows = graph
        .rows(inventory_graph::cypher::diagnostics::UNNAMED_RESOURCES)
        .await?;
    assert_eq!(
        rows.len(),
        metadata.len(),
        "null names are distinct from empty strings"
    );
    assert_eq!(
        rows.into_iter().collect::<BTreeSet<_>>(),
        metadata,
        "native names/hashes must match independently checked public canonical records"
    );
    let rows = graph
        .rows(inventory_graph::cypher::diagnostics::RESOURCE_EDGES)
        .await?;
    assert_eq!(
        rows.len(),
        expected.edges.len(),
        "native dependency cardinality"
    );
    assert_eq!(
        rows.into_iter().collect::<BTreeSet<_>>(),
        expected.edges,
        "native directed dependency relation/provenance set"
    );
    Ok(())
}

async fn assert_read_model(
    database: &Path,
    graph: &NativeGraph,
    expected: &Expected,
) -> TestResult {
    let database = Database::open(database.to_str().ok_or("non-UTF-8 database path")?).await?;
    let catalog_bytes = std::fs::read(super::runtime::catalog_path())?;
    let catalog = inventory_core::catalog::runtime::RuntimeCatalog::from_bytes(&catalog_bytes)?;
    assert_eq!(
        expected.safe_records.len(),
        expected.resources.len(),
        "every resource needs independent safe field evidence"
    );
    assert_read_page(&database, expected).await?;
    let graph_reader = GraphConnection::connect(graph.endpoint(), graph.name())?;
    let mut sql_edges = BTreeSet::new();
    let mut native_edges = BTreeSet::new();
    let mut issues = super::safe_shape::Violations::default();
    for key in &expected.resources {
        let node = node(key);
        let detail = database
            .detail(&node)
            .await?
            .ok_or("UI reader lost seeded resource detail")?;
        assert_eq!(&detail.key, key);
        let fields = expected
            .safe_records
            .get(key)
            .ok_or("missing independent safe-field fixture")?;
        super::safe_fields::inspect_detail(&detail, fields, &catalog, &mut issues)?;
        let edges = database
            .edges_page(
                &node,
                EdgePageRequest {
                    limit: 1000,
                    after: None,
                },
            )
            .await?;
        assert_eq!(edges.next, None);
        sql_edges.extend(edges.edges.iter().map(edge_fields));
        extend_ui_neighbors(&graph_reader, &node, key, &mut native_edges).await?;
    }
    issues.finish()?;
    assert_eq!(
        sql_edges, expected.edges,
        "public UI SQLite dependency/provenance set"
    );
    assert_eq!(
        native_edges, expected.edges,
        "public UI actual FalkorDB dependency/provenance set"
    );
    Ok(())
}

async fn extend_ui_neighbors(
    graph: &GraphConnection,
    node: &GraphNode,
    key: &ResourceKey,
    edges: &mut BTreeSet<Vec<String>>,
) -> TestResult {
    let neighborhood = graph
        .neighbors(
            node,
            GraphLimits {
                depth: 3,
                nodes: 500,
                edges: 2000,
            },
        )
        .await?;
    assert!(
        !neighborhood.truncated,
        "fixture native UI neighborhood must be complete: {key:?}"
    );
    edges.extend(neighborhood.edges.iter().map(edge_fields));
    Ok(())
}

async fn assert_read_page(database: &Database, expected: &Expected) -> TestResult {
    let page = database
        .page(
            &ResourceFilter::default(),
            PageRequest {
                limit: 1000,
                after: None,
            },
        )
        .await?;
    assert_eq!(page.next, None);
    assert_eq!(
        page.total_matching,
        u64::try_from(expected.resources.len())?
    );
    assert_eq!(
        page.resources
            .iter()
            .map(|row| row.key.clone())
            .collect::<BTreeSet<_>>(),
        expected.resources,
        "public UI SQLite resource reader must return exact full keys"
    );
    Ok(())
}
