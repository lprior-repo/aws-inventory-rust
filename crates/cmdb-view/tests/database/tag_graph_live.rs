use super::postgres_support as pg;
use super::*;
use cmdb_view::{GraphConnection, GraphLimits, GraphNode, ResourceTag, ScopedTag, TagAffiliation};
use inventory_core::inventory::{AccountId, Kind, Region, ResourceId, ResourceKey};
use std::fmt::Write;

const ACCOUNT: &str = "111111111111";
const BUCKET: &str = "AWS::Example::Kind0";
const INSTANCE: &str = "AWS::Example::Kind1";
const HOSTILE: &str = inventory_graph::cypher::diagnostics::HOSTILE_TAG_VALUE;

fn node(region: &str, kind: &str, id: &str) -> GraphNode {
    GraphNode {
        account: ACCOUNT.into(),
        region: region.into(),
        kind: kind.into(),
        id: id.into(),
    }
}

fn root() -> GraphNode {
    node("global", BUCKET, "shared")
}

fn digest(node: &GraphNode) -> TestResult<String> {
    let key = ResourceKey {
        account: AccountId::parse(&node.account)?,
        region: Region::parse(&node.region)?,
        kind: Kind::parse(&node.kind)?,
        id: ResourceId::parse(&node.id)?,
    };
    let mut value = String::with_capacity(64);
    key.digest()
        .as_bytes()
        .iter()
        .try_for_each(|byte| write!(value, "{byte:02x}"))?;
    Ok(value)
}

fn literal(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}

fn seed_query() -> TestResult<String> {
    let root_key = digest(&root())?;
    let east_key = digest(&node("us-east-1", BUCKET, "shared"))?;
    let other_key = digest(&node("global", INSTANCE, "shared"))?;
    let hostile_key = digest(&node("global", BUCKET, HOSTILE))?;
    Ok(format!(
        "CYPHER root='{root_key}' east='{east_key}' other='{other_key}' hostile_key='{hostile_key}' hostile={} CREATE \
        (a:Resource {{key:$root,account:'{ACCOUNT}',region:'global',kind:'{BUCKET}',id:'shared'}}),\
        (b:Resource {{key:$east,account:'{ACCOUNT}',region:'us-east-1',kind:'{BUCKET}',id:'shared'}}),\
        (c:Resource {{key:$other,account:'{ACCOUNT}',region:'global',kind:'{INSTANCE}',id:'shared'}}),\
        (h:Resource {{key:$hostile_key,account:'{ACCOUNT}',region:'global',kind:'{BUCKET}',id:$hostile}}),\
        (bad:Resource {{key:$root,account:'{ACCOUNT}',region:'global',kind:'{BUCKET}',id:'wrong-identity'}}),\
        (wrong_key:Resource {{key:'wrong-digest',account:'{ACCOUNT}',region:'global',kind:'{BUCKET}',id:'shared'}}),\
        (owner:Tag {{account:'{ACCOUNT}',region:'global',tag_key:'owner',tag_value:'core'}}),\
        (hostile:Tag {{account:'{ACCOUNT}',region:'global',tag_key:'z',tag_value:$hostile}}),\
        (east:Tag {{account:'{ACCOUNT}',region:'us-east-1',tag_key:'owner',tag_value:'east'}}),\
        (other:Tag {{account:'{ACCOUNT}',region:'global',tag_key:'owner',tag_value:'other-kind'}}),\
        (foreign:Tag {{account:'222222222222',region:'global',tag_key:'owner',tag_value:'foreign-account'}}),\
        (bogus:Tag {{account:'{ACCOUNT}',region:'global',tag_key:'bogus',tag_value:'not-persisted'}}),\
        (a)-[:HAS_TAG {{provenance:'inventory.tags'}}]->(owner),\
        (a)-[:HAS_TAG {{provenance:'inventory.tags'}}]->(hostile),\
        (a)-[:HAS_TAG {{provenance:'inventory.tags'}}]->(east),\
        (a)-[:HAS_TAG {{provenance:'inventory.tags'}}]->(foreign),\
        (a)-[:HAS_TAG {{provenance:'untrusted'}}]->(bogus),\
        (b)-[:HAS_TAG {{provenance:'inventory.tags'}}]->(east),\
        (c)-[:HAS_TAG {{provenance:'inventory.tags'}}]->(other),\
        (h)-[:HAS_TAG {{provenance:'inventory.tags'}}]->(hostile),\
        (bad)-[:HAS_TAG {{provenance:'inventory.tags'}}]->(bogus),\
        (wrong_key)-[:HAS_TAG {{provenance:'inventory.tags'}}]->(bogus),\
        (a)-[:replica {{relation:'replica',provenance:'catalog'}}]->(b)",
        literal(HOSTILE)
    ))
}

fn expected(region: &str, key: &str, value: &str) -> TagAffiliation {
    TagAffiliation {
        tag: ScopedTag {
            account: ACCOUNT.into(),
            region: region.into(),
            key: key.into(),
            value: value.into(),
        },
        provenance: "inventory.tags".into(),
    }
}

async fn check_membership(graph: &GraphConnection, db: &Database) -> TestResult {
    let complete = graph.tag_affiliations(&root(), 10).await?;
    let bounded = graph.tag_affiliations(&root(), 1).await?;
    let detail = db.detail(&root()).await?.ok_or("SQL root missing")?;
    let dependencies = graph
        .neighbors(
            &root(),
            GraphLimits {
                depth: 1,
                nodes: 2,
                edges: 1,
            },
        )
        .await?;
    assert_eq!(
        complete.affiliations,
        vec![
            expected("global", "owner", "core"),
            expected("global", "z", HOSTILE)
        ]
    );
    assert!(!complete.truncated);
    assert_eq!(
        bounded.affiliations,
        vec![expected("global", "owner", "core")]
    );
    assert!(bounded.truncated);
    assert_eq!(
        detail.tags,
        complete
            .affiliations
            .iter()
            .map(|affiliation| ResourceTag {
                key: affiliation.tag.key.clone(),
                value: affiliation.tag.value.clone()
            })
            .collect::<Vec<_>>()
    );
    assert_eq!(dependencies.edges.len(), 1);
    assert_eq!(
        dependencies
            .edges
            .first()
            .map(|edge| (&*edge.relation, &*edge.provenance)),
        Some(("replica", "catalog"))
    );
    assert!(!dependencies.truncated);
    assert_eq!(
        graph
            .tag_affiliations(&node("us-east-1", BUCKET, "shared"), 10)
            .await?
            .affiliations,
        vec![expected("us-east-1", "owner", "east")]
    );
    assert_eq!(
        graph
            .tag_affiliations(&node("global", INSTANCE, "shared"), 10)
            .await?
            .affiliations,
        vec![expected("global", "owner", "other-kind")]
    );
    assert_eq!(
        graph
            .tag_affiliations(&node("global", BUCKET, HOSTILE), 10)
            .await?
            .affiliations,
        vec![expected("global", "z", HOSTILE)]
    );
    let absent = graph
        .tag_affiliations(&node("global", BUCKET, "absent"), 10)
        .await?;
    assert!(absent.affiliations.is_empty());
    assert!(!absent.truncated);
    Ok(())
}

#[tokio::test]
async fn given_real_native_tags_when_reading_then_affiliations_are_scoped_bounded_and_not_aws_dependencies()
-> TestResult {
    const ENDPOINT_ENV: &str = "CMDB_VIEW_FALKOR_ENDPOINT";
    let endpoint = std::env::var(ENDPOINT_ENV)
        .map_err(|error| format!("native tag graph test requires {ENDPOINT_ENV}: {error}"))?;
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let name = format!("cmdb_view_tags_{}_{suffix}", std::process::id());
    let mut connection = redis::Client::open(endpoint.as_str())?
        .get_multiplexed_async_connection()
        .await?;
    let graph = GraphConnection::connect(&endpoint, &name)?;
    let fixture = pg::Fixture::new().await?;
    let result = async {
        redis::cmd(inventory_graph::cypher::COMMAND)
            .arg(&name)
            .arg(seed_query()?)
            .query_async::<redis::Value>(&mut connection)
            .await?;
        sqlx::query(
            "INSERT INTO inventory.tags (account,region,kind,id,tag_key,tag_value) VALUES \
             ($1,$2,$3,$4,'owner','core'),($1,$2,$3,$4,'z',$5)",
        )
        .bind(ACCOUNT)
        .bind("global")
        .bind(BUCKET)
        .bind("shared")
        .bind(HOSTILE)
        .execute(&fixture.pool)
        .await?;
        let db = fixture.reader().await?;
        check_membership(&graph, &db).await
    }
    .await;
    let cleanup = redis::cmd("GRAPH.DELETE")
        .arg(&name)
        .query_async::<redis::Value>(&mut connection)
        .await;
    let finished = fixture.finish().await;
    match (result.and(finished), cleanup) {
        (Ok(()), Ok(_)) => Ok(()),
        (Err(error), Ok(_)) => Err(error),
        (Ok(()), Err(error)) => Err(error.into()),
        (Err(error), Err(cleanup)) => {
            Err(format!("{error}; graph cleanup failed: {cleanup}").into())
        }
    }
}
