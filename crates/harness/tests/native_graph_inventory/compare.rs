use super::{TestResult, native::NativeGraph};
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
            .map(|edge| {
                key_fields(&edge.from)
                    .into_iter()
                    .chain(key_fields(&edge.to))
                    .chain([
                        edge.relation.as_str().to_owned(),
                        edge.source.as_str().to_owned(),
                    ])
                    .collect()
            })
            .collect();
        Ok(Self {
            resources,
            edges,
            safe_records: super::safe_fields::Expectations::new(),
        })
    }
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
    assert_eq!(
        resources.len(),
        expected.resources.len(),
        "SQLite resource cardinality"
    );
    assert_eq!(
        resources
            .into_iter()
            .map(|row| row.key)
            .collect::<BTreeSet<_>>(),
        expected.resources,
        "SQLite must contain exactly the independently seeded full keys, not merely a subset"
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
        "native resource full keys and production digest must match the seed oracle"
    );
    let view = Database::open(database.to_str().ok_or("non-UTF-8 database path")?).await?;
    let mut metadata = BTreeSet::new();
    for key in &expected.resources {
        let detail = view
            .detail(&node(key))
            .await?
            .ok_or("metadata resource missing")?;
        // Names are explicitly absent in the independently frozen fixture policy.
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
    let graph_reader = GraphConnection::connect(graph.endpoint(), graph.name())?;
    let mut sql_edges = BTreeSet::new();
    let mut native_edges = BTreeSet::new();
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
        super::safe_fields::assert_detail(&detail, fields, &catalog)?;
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
        let neighborhood = graph_reader
            .neighbors(
                &node,
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
        native_edges.extend(neighborhood.edges.iter().map(edge_fields));
    }
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
