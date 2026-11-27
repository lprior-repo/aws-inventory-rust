use super::postgres_support as pg;
use super::*;
use cmdb_view::{GraphConnection, GraphEdge, GraphLimits, GraphNode};
use inventory_core::inventory::{AccountId, Kind, Region, Relation, ResourceId, ResourceKey};
use inventory_graph::{EdgeRow, NodeRow, Provenance};

fn nodes() -> [GraphNode; 4] {
    [
        pg::node("111111111111", "global", 0, "shared"),
        pg::node("111111111111", "us-east-1", 0, "shared"),
        pg::node("222222222222", "eu-west-1", 0, "shared"),
        pg::node("222222222222", "us-west-2", 1, "shared"),
    ]
}

fn edge(from: &GraphNode, to: &GraphNode) -> GraphEdge {
    GraphEdge {
        from: from.clone(),
        to: to.clone(),
        relation: "replica".to_owned(),
        provenance: "catalog".to_owned(),
    }
}

fn expected() -> Vec<GraphEdge> {
    let [a, b, c, d] = nodes();
    vec![edge(&a, &b), edge(&a, &c), edge(&b, &c), edge(&c, &d)]
}

fn resource_key(node: &GraphNode) -> Result<ResourceKey, Box<dyn Error>> {
    Ok(ResourceKey {
        account: AccountId::parse(&node.account)?,
        region: Region::parse(&node.region)?,
        kind: Kind::parse(&node.kind)?,
        id: ResourceId::parse(&node.id)?,
    })
}

async fn seed_graph(connection: &mut redis::aio::MultiplexedConnection, name: &str) -> TestResult {
    let nodes = nodes()
        .iter()
        .map(|node| {
            Ok(NodeRow {
                key: resource_key(node)?,
                name: None,
                content_hash: [0; 32],
            })
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    let edges = expected()
        .iter()
        .map(|edge| {
            Ok(EdgeRow {
                from: resource_key(&edge.from)?,
                to: resource_key(&edge.to)?,
                relation: Relation::parse(&edge.relation)?,
                provenance: Provenance::parse(&edge.provenance)?,
            })
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    for query in inventory_graph::cypher::diagnostics::resource_fixture(&nodes, &edges) {
        redis::cmd(inventory_graph::cypher::COMMAND)
            .arg(name)
            .arg(query?)
            .query_async::<redis::Value>(&mut *connection)
            .await?;
    }
    Ok(())
}

async fn check_neighbors(graph: &GraphConnection) -> TestResult {
    let [root, _, _, _] = nodes();
    let complete = graph
        .neighbors(
            &root,
            GraphLimits {
                depth: 3,
                nodes: 4,
                edges: 4,
            },
        )
        .await?;
    if complete.truncated
        || complete.edges.len() != 4
        || !expected().iter().all(|edge| complete.edges.contains(edge))
    {
        return Err(format!("inexact complete neighborhood: {complete:?}").into());
    }
    let shallow = graph
        .neighbors(
            &root,
            GraphLimits {
                depth: 1,
                nodes: 4,
                edges: 4,
            },
        )
        .await?;
    let [a, b, c, _] = nodes();
    if !shallow.truncated
        || shallow.edges.len() != 3
        || !shallow.edges.contains(&edge(&a, &b))
        || !shallow.edges.contains(&edge(&a, &c))
        || !shallow.edges.contains(&edge(&b, &c))
    {
        return Err(format!("inexact shallow neighborhood: {shallow:?}").into());
    }
    let bounded = graph
        .neighbors(
            &root,
            GraphLimits {
                depth: 3,
                nodes: 4,
                edges: 2,
            },
        )
        .await?;
    if !bounded.truncated || bounded.edges.len() != 2 {
        return Err(format!("inexact edge bound: {bounded:?}").into());
    }
    Ok(())
}

async fn check_path_and_blast(graph: &GraphConnection) -> TestResult {
    let [_a, b, c, d] = nodes();
    let bounds = GraphLimits {
        depth: 3,
        nodes: 4,
        edges: 4,
    };
    let result = graph.path(&b, &d, bounds).await?;
    let cmdb_view::PathResult::Path(edges) = result else {
        return Err(format!("expected Path variant, got {result:?}").into());
    };
    if edges != vec![edge(&b, &c), edge(&c, &d)] {
        return Err(format!("inexact shortest path: {edges:?}").into());
    }
    let shallow = graph
        .path(
            &b,
            &d,
            GraphLimits {
                depth: 1,
                nodes: 4,
                edges: 4,
            },
        )
        .await?;
    if !matches!(shallow, cmdb_view::PathResult::NotFound) {
        return Err(format!("expected NotFound, got {shallow:?}").into());
    }
    let same = graph.path(&b, &b, bounds).await?;
    if !matches!(same, cmdb_view::PathResult::SameNode) {
        return Err(format!("expected SameNode, got {same:?}").into());
    }
    let [_, _, _, d] = nodes();
    let blast = graph.blast_radius(&d, bounds).await?;
    if blast.truncated
        || blast.edges.len() != 4
        || !expected().iter().all(|edge| blast.edges.contains(edge))
    {
        return Err(format!("inexact blast radius: {blast:?}").into());
    }
    Ok(())
}

async fn compare_sql(db: &Database, graph: &GraphConnection) -> TestResult {
    let [root, _, _, _] = nodes();
    let graph_edges = graph
        .neighbors(
            &root,
            GraphLimits {
                depth: 3,
                nodes: 4,
                edges: 4,
            },
        )
        .await?
        .edges;
    let sql_edges = db
        .edges_page(
            &root,
            cmdb_view::EdgePageRequest {
                limit: 10,
                after: None,
            },
        )
        .await?
        .edges;
    if sql_edges.len() != 2 || !sql_edges.iter().all(|edge| graph_edges.contains(edge)) {
        return Err("SQL and graph resource identities disagree".into());
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires CMDB_VIEW_FALKOR_ENDPOINT and INVENTORY_POSTGRES_URL with CREATEDB"]
async fn given_seeded_native_graph_when_reading_then_neighbors_paths_blast_and_sql_agree()
-> TestResult {
    let endpoint = std::env::var("CMDB_VIEW_FALKOR_ENDPOINT")?;
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let name = format!("cmdb_view_queries_{}_{suffix}", std::process::id());
    let mut connection = redis::Client::open(endpoint.as_str())?
        .get_multiplexed_async_connection()
        .await?;
    let graph = GraphConnection::connect(&endpoint, &name)?;
    let fixture = pg::Fixture::new().await?;
    let result = async {
        let db = fixture.reader().await?;
        seed_graph(&mut connection, &name).await?;
        check_neighbors(&graph).await?;
        check_path_and_blast(&graph).await?;
        compare_sql(&db, &graph).await
    }
    .await;
    let cleanup = redis::cmd("GRAPH.DELETE")
        .arg(&name)
        .query_async::<redis::Value>(&mut connection)
        .await;
    let finished = fixture.finish().await;
    finish_test(result.and(finished), cleanup)
}

fn finish_test(result: TestResult, cleanup: Result<redis::Value, redis::RedisError>) -> TestResult {
    match (result, cleanup) {
        (Ok(()), Ok(_)) => Ok(()),
        (Err(error), Ok(_)) => Err(error),
        (Ok(()), Err(error)) => Err(error.into()),
        (Err(error), Err(cleanup)) => {
            Err(format!("{error}; graph cleanup failed: {cleanup}").into())
        }
    }
}
