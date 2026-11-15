use anyhow::{Context, Result, ensure};
use cmdb_view::{Database, GraphConnection, GraphLimits, GraphNode, PageRequest, ResourceFilter};
use inventory_core::inventory::EdgeSource;
use inventory_engine::inventory::{BinaryStore, ProjectionEdgeHeader};
use inventory_harness::inventory_seed::TypedSeedOracle;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::path::Path;

type MissCounters = Vec<(String, String, String, String, Vec<u8>, i64)>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct InventorySnapshot {
    pub(super) resources: Vec<(inventory_core::inventory::ResourceKey, [u8; 32])>,
    pub(super) edges: Vec<ProjectionEdgeHeader>,
    pub(super) miss_counters: MissCounters,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ActiveCheckpoint {
    pub(super) sweep_id: String,
    pub(super) resources_observed: u64,
    pub(super) work_pending: u64,
}
pub(super) async fn database_snapshot(
    path: &Path,
    oracle: &TypedSeedOracle,
) -> Result<InventorySnapshot> {
    let database = Database::open(path).await?;
    let counts = database.counts().await?;
    let page = database
        .page(
            &ResourceFilter::default(),
            PageRequest {
                limit: 1000,
                after: None,
            },
        )
        .await?;
    ensure!(
        page.next.is_none(),
        "E2E inventory exceeds the bounded one-page snapshot"
    );
    let rows = u64::try_from(page.resources.len())?;
    ensure!(
        page.total_matching == counts.resources && counts.resources == rows && counts.resources > 0,
        "SQL resource counts must equal the persisted page rows"
    );
    ensure!(
        counts.accounts == 1 && counts.regions > 0 && counts.services > 0,
        "SQL account, region, and service counts must reflect persisted rows"
    );
    assert_seeded_rows(oracle, &page.resources)?;
    let store = BinaryStore::open(path).await?;
    let edge_page = store.projection_edges(None, 1000).await?;
    ensure!(
        edge_page.next.is_none(),
        "E2E relationships exceed the bounded one-page snapshot"
    );
    ensure!(
        oracle
            .pairs
            .iter()
            .all(|pair| edge_page.edges.iter().any(|edge| related(edge, pair))),
        "SQLite is missing a seeded VPC-to-subnet relationship"
    );
    let resources = page
        .resources
        .into_iter()
        .map(|row| (row.key, *row.content_hash.as_bytes()))
        .collect();
    let miss_counters = miss_counters(path).await?;
    Ok(InventorySnapshot {
        resources,
        edges: edge_page.edges,
        miss_counters,
    })
}
fn assert_seeded_rows(oracle: &TypedSeedOracle, rows: &[cmdb_view::StoredResource]) -> Result<()> {
    ensure!(
        oracle.pairs.len() == 6,
        "live seed must return all six typed VPC/subnet pairs"
    );
    ensure!(
        oracle
            .pairs
            .first()
            .is_some_and(|pair| pair.vpc_id == oracle.vpc_id
                && pair.subnet_id == oracle.subnet_id
                && pair.vpc_key == oracle.vpc_key
                && pair.subnet_key == oracle.subnet_key
                && pair.relationship == oracle.relationship),
        "typed seed primary output must match its first actual resource pair"
    );
    let unique_vpcs = oracle
        .pairs
        .iter()
        .map(|pair| &pair.vpc_key)
        .collect::<std::collections::BTreeSet<_>>();
    let unique_subnets = oracle
        .pairs
        .iter()
        .map(|pair| &pair.subnet_key)
        .collect::<std::collections::BTreeSet<_>>();
    ensure!(
        unique_vpcs.len() == oracle.pairs.len() && unique_subnets.len() == oracle.pairs.len(),
        "seeded resource identities must all be distinct"
    );
    ensure!(
        oracle.pairs.iter().all(|pair| rows
            .iter()
            .any(|row| row.key == pair.vpc_key && pair.vpc_id == pair.vpc_key.id)),
        "seeded VPC IDs and scoped keys are absent from SQLite"
    );
    ensure!(
        oracle.pairs.iter().all(|pair| rows
            .iter()
            .any(|row| row.key == pair.subnet_key && pair.subnet_id == pair.subnet_key.id)),
        "seeded subnet IDs and scoped keys are absent from SQLite"
    );
    ensure!(
        oracle
            .pairs
            .iter()
            .all(|pair| pair.vpc_key.kind.as_str() == "AWS::EC2::VPC"
                && pair.subnet_key.kind.as_str() == "AWS::EC2::Subnet"
                && pair.relationship.as_str() == "IN_VPC"),
        "seed keys must preserve modeled EC2 kinds and relationship"
    );
    ensure!(
        oracle
            .pairs
            .iter()
            .all(|pair| pair.vpc_key.account == oracle.account_id
                && pair.subnet_key.account == oracle.account_id),
        "seed keys must preserve returned account identity"
    );
    ensure!(
        oracle
            .pairs
            .iter()
            .all(|pair| pair.vpc_key.region == oracle.region
                && pair.subnet_key.region == oracle.region),
        "seed keys must preserve returned region identity"
    );
    Ok(())
}

async fn miss_counters(path: &Path) -> Result<MissCounters> {
    let pool = readonly_pool(path).await?;
    let rows = sqlx::query_as::<_, (String, String, String, String, Vec<u8>, i64)>("SELECT account,region,kind,id,scope_key,missing_complete FROM resource_scopes ORDER BY account,region,kind,id,scope_key").fetch_all(&pool).await?;
    pool.close().await;
    Ok(rows)
}

async fn readonly_pool(path: &Path) -> Result<sqlx::SqlitePool> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .read_only(true)
        .pragma("query_only", "ON");
    Ok(SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?)
}

pub(super) async fn active_checkpoint(path: &Path) -> Result<ActiveCheckpoint> {
    let pool = readonly_pool(path).await?;
    let mut ids = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT sweep_id FROM work_queue WHERE state IN(0,1)",
    )
    .fetch_all(&pool)
    .await?;
    ensure!(
        ids.len() == 1,
        "interrupted CLI must leave exactly one active sweep"
    );
    let sweep_id = ids.pop().context("active sweep identifier is missing")?;
    let observed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM (SELECT account,region,kind,id FROM resource_scopes WHERE observed_sweep=?1 GROUP BY account,region,kind,id)").bind(&sweep_id).fetch_one(&pool).await?;
    let pending: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM work_queue WHERE sweep_id=?1 AND state IN(0,1)")
            .bind(&sweep_id)
            .fetch_one(&pool)
            .await?;
    pool.close().await;
    Ok(ActiveCheckpoint {
        sweep_id,
        resources_observed: u64::try_from(observed)?,
        work_pending: u64::try_from(pending)?,
    })
}

pub(super) async fn assert_sweep_finished(path: &Path, sweep_id: &str) -> Result<()> {
    let pool = readonly_pool(path).await?;
    let unfinished: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM work_queue WHERE sweep_id=?1 AND state<>2")
            .bind(sweep_id)
            .fetch_one(&pool)
            .await?;
    let outcomes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM scope_outcomes WHERE sweep_id=?1")
        .bind(sweep_id)
        .fetch_one(&pool)
        .await?;
    pool.close().await;
    ensure!(
        unfinished == 0 && outcomes > 0,
        "resume must complete the exact interrupted sweep ID"
    );
    Ok(())
}
pub(super) async fn assert_graph(
    path: &Path,
    endpoint: &str,
    graph: &str,
    oracle: &TypedSeedOracle,
) -> Result<()> {
    let store = BinaryStore::open(path).await?;
    let database_edges = store.projection_edges(None, 1000).await?.edges;
    let connection = GraphConnection::connect(endpoint, graph)?;
    for pair in &oracle.pairs {
        let expected = database_edges
            .iter()
            .find(|edge| related(edge, pair))
            .ok_or_else(|| anyhow::anyhow!("SQLite has no seeded VPC-to-subnet relationship"))?;
        ensure!(
            pair.edge.source == EdgeSource::ReviewedRule,
            "typed seed must use reviewed relationship provenance"
        );
        ensure!(
            expected.source == pair.edge.source.as_str(),
            "SQLite edge provenance must match the typed seed edge"
        );
        let actual = connection
            .neighbors(
                &node(&pair.edge.from),
                GraphLimits {
                    depth: 1,
                    nodes: 20,
                    edges: 20,
                },
            )
            .await?;
        ensure!(
            actual.edges.iter().all(|edge| database_edges
                .iter()
                .any(|expected| graph_matches(edge, expected))),
            "Falkor contains a seeded relationship absent from SQLite"
        );
        ensure!(
            actual
                .edges
                .iter()
                .any(|edge| graph_matches(edge, expected)),
            "Falkor relationship and provenance differ from SQLite"
        );
    }
    Ok(())
}

fn related(
    edge: &ProjectionEdgeHeader,
    pair: &inventory_harness::inventory_seed::SeedResourcePair,
) -> bool {
    edge.from == pair.edge.from
        && edge.to == pair.edge.to
        && edge.relation == pair.edge.relation
        && edge.source == pair.edge.source.as_str()
}

fn graph_matches(edge: &cmdb_view::GraphEdge, expected: &ProjectionEdgeHeader) -> bool {
    edge.from == node(&expected.from)
        && edge.to == node(&expected.to)
        && edge.relation == expected.relation.as_str()
        && edge.provenance == expected.source
}

fn node(key: &inventory_core::inventory::ResourceKey) -> GraphNode {
    GraphNode {
        account: key.account.as_str().to_owned(),
        region: key.region.as_str().to_owned(),
        kind: key.kind.as_str().to_owned(),
        id: key.id.as_str().to_owned(),
    }
}
