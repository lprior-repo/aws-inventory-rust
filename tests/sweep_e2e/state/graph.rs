use super::*;
use cmdb_view::{GraphConnection, GraphLimits, GraphNode};
use inventory_core::inventory::{EdgeSource, ResourceKey};
use inventory_harness::inventory_seed::SeedResourcePair;

pub(crate) async fn assert_graph(
    database: &Database,
    endpoint: &str,
    graph: &str,
    oracle: &TypedSeedOracle,
) -> Result<()> {
    ensure!(
        oracle.pairs.len() == 6,
        "live seed must return all six typed resource pairs"
    );
    let edges = database.edges().await?;
    let connection = GraphConnection::connect(endpoint, graph)?;
    for pair in &oracle.pairs {
        verify_pair(&connection, pair, &edges).await?;
    }
    Ok(())
}

async fn verify_pair(
    connection: &GraphConnection,
    pair: &SeedResourcePair,
    edges: &[ProjectionEdgeHeader],
) -> Result<()> {
    let expected = expected(edges, pair)?;
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
    verify_edges(&actual.edges, expected, edges)
}

fn expected<'a>(
    edges: &'a [ProjectionEdgeHeader],
    pair: &SeedResourcePair,
) -> Result<&'a ProjectionEdgeHeader> {
    let expected = edges
        .iter()
        .find(|edge| related(edge, pair))
        .context("SQL has no seeded VPC-to-subnet relationship")?;
    ensure!(
        pair.edge.source == EdgeSource::ReviewedRule,
        "typed seed must use reviewed relationship provenance"
    );
    ensure!(
        expected.source == pair.edge.source.as_str(),
        "SQL edge provenance must match the typed seed edge"
    );
    Ok(expected)
}

fn verify_edges(
    actual: &[cmdb_view::GraphEdge],
    expected: &ProjectionEdgeHeader,
    edges: &[ProjectionEdgeHeader],
) -> Result<()> {
    ensure!(
        actual
            .iter()
            .all(|edge| edges.iter().any(|expected| graph_matches(edge, expected))),
        "Falkor contains a seeded relationship absent from SQL"
    );
    ensure!(
        actual.iter().any(|edge| graph_matches(edge, expected)),
        "Falkor relationship and provenance differ from SQL"
    );
    Ok(())
}

fn related(edge: &ProjectionEdgeHeader, pair: &SeedResourcePair) -> bool {
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

fn node(key: &ResourceKey) -> GraphNode {
    GraphNode {
        account: key.account.as_str().to_owned(),
        region: key.region.as_str().to_owned(),
        kind: key.kind.as_str().to_owned(),
        id: key.id.as_str().to_owned(),
    }
}
