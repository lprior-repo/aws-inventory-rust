use super::*;
use cmdb_view::{GraphConnection, GraphEdge, GraphLimits, GraphNode};
use inventory_core::inventory::{AccountId, Kind, Region, ResourceId, ResourceKey};
use std::fmt::Write;

fn node(account: &str, region: &str, kind: &str) -> GraphNode {
    GraphNode {
        account: account.to_owned(),
        region: region.to_owned(),
        kind: kind.to_owned(),
        id: "shared".to_owned(),
    }
}

fn nodes() -> [GraphNode; 4] {
    [
        node("111111111111", "global", "AWS::S3::Bucket"),
        node("111111111111", "us-east-1", "AWS::S3::Bucket"),
        node("222222222222", "eu-west-1", "AWS::S3::Bucket"),
        node("222222222222", "us-west-2", "AWS::EC2::Instance"),
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

fn resource_node(alias: &str, node: &GraphNode) -> Result<String, Box<dyn Error>> {
    let key = ResourceKey {
        account: AccountId::parse(&node.account)?,
        region: Region::parse(&node.region)?,
        kind: Kind::parse(&node.kind)?,
        id: ResourceId::parse(&node.id)?,
    };
    let mut digest = String::with_capacity(64);
    key.digest()
        .as_bytes()
        .iter()
        .try_for_each(|byte| write!(digest, "{byte:02x}"))?;
    Ok(format!(
        "({alias}:Resource {{key:'{digest}',account:'{}',region:'{}',kind:'{}',id:'{}'}})",
        node.account, node.region, node.kind, node.id
    ))
}

fn seed_query() -> Result<String, Box<dyn Error>> {
    let [a, b, c, d] = nodes();
    Ok(format!(
        "CREATE {},{},{},{},(a)-[:replica {{relation:'replica',provenance:'catalog'}}]->(b),(a)-[:replica {{relation:'replica',provenance:'catalog'}}]->(c),(b)-[:replica {{relation:'replica',provenance:'catalog'}}]->(c),(c)-[:replica {{relation:'replica',provenance:'catalog'}}]->(d)",
        resource_node("a", &a)?,
        resource_node("b", &b)?,
        resource_node("c", &c)?,
        resource_node("d", &d)?
    ))
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
        || shallow.edges.len() != 2
        || !shallow.edges.contains(&edge(&a, &b))
        || !shallow.edges.contains(&edge(&a, &c))
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
    // UI-04: Same node returns SameNode variant (zero-hop path).
    let same = graph.path(&b, &b, bounds).await?;
    if !matches!(same, cmdb_view::PathResult::SameNode) {
        return Err(format!("expected SameNode, got {same:?}").into());
    }
    // UI-04: Directed blast radius - follow incoming edges to find
    // upstream dependents. For graph A->B, A->C, B->C, C->D (edges
    // represent "source depends on target"), blast radius of D should
    // find all 4 edges (everything depends on D transitively).
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
#[ignore = "requires native FalkorDB at CMDB_VIEW_FALKOR_ENDPOINT"]
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
    let result = async {
        redis::cmd("GRAPH.QUERY")
            .arg(&name)
            .arg(seed_query()?)
            .query_async::<redis::Value>(&mut connection)
            .await?;
        let dir = tempdir()?;
        let db = populated(&dir.path().join("inventory.sqlite")).await?;
        check_neighbors(&graph).await?;
        check_path_and_blast(&graph).await?;
        compare_sql(&db, &graph).await
    }
    .await;
    let cleanup = redis::cmd("GRAPH.DELETE")
        .arg(&name)
        .query_async::<redis::Value>(&mut connection)
        .await;
    finish_test(result, cleanup)
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
