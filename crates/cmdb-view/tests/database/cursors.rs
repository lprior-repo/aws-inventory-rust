use super::*;
use cmdb_view::{EdgePageRequest, GraphEdge, GraphNode, ViewError};

fn node(kind: &str, region: &str) -> GraphNode {
    GraphNode { account: "111111111111".to_owned(), region: region.to_owned(), kind: kind.to_owned(), id: "shared".to_owned() }
}

#[tokio::test]
async fn given_parallel_edges_and_self_loop_when_paging_then_every_identity_appears_once() -> TestResult {
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let db = populated(&path).await?;
    let pool = writer(&path).await?;
    let root = node("AWS::S3::Bucket", "global");
    let mut parallel = GraphEdge { from: root.clone(), to: node("AWS::S3::Bucket", "us-east-1"), relation: "replica".to_owned(), provenance: "reviewed".to_owned() };
    seed_edge(&pool, &parallel).await?;
    parallel.relation = "references".to_owned();
    seed_edge(&pool, &parallel).await?;
    seed_edge(&pool, &GraphEdge { from: root.clone(), to: root.clone(), relation: "self".to_owned(), provenance: "inferred".to_owned() }).await?;
    let mut snapshot = db.snapshot().await?;
    let first = snapshot.edges_page(&root, EdgePageRequest { limit: 2, after: None }).await?;
    let second = snapshot.edges_page(&root, EdgePageRequest { limit: 2, after: first.next }).await?;
    let third = snapshot.edges_page(&root, EdgePageRequest { limit: 2, after: second.next }).await?;
    assert_eq!((first.total_matching, second.total_matching, third.total_matching), (5, 5, 5));
    assert!(third.next.is_none());
    let identities = first.edges.iter().chain(&second.edges).chain(&third.edges).map(|edge| (edge.to.region.as_str(), edge.relation.as_str(), edge.provenance.as_str())).collect::<Vec<_>>();
    assert_eq!(identities, vec![("global", "self", "inferred"), ("us-east-1", "references", "reviewed"), ("us-east-1", "replica", "catalog"), ("us-east-1", "replica", "reviewed"), ("eu-west-1", "replica", "catalog")]);
    snapshot.finish().await?;
    pool.close().await;
    Ok(())
}

fn named_record(value: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut builder = RecordBuilder::new(1024, 8, 4)?;
    builder.begin_map(None)?;
    builder.scalar(Some(KeyRef::Name("name")), ValueRef::Text(value))?;
    builder.end()?;
    Ok(builder.finish()?.into_bytes())
}

#[tokio::test]
async fn given_identical_account_region_and_id_when_details_differ_by_kind_then_binary_rows_are_exact() -> TestResult {
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let db = populated(&path).await?;
    let pool = writer(&path).await?;
    let bytes = named_record("instance-detail")?;
    seed(&pool, Seed { table: "instance_records", account: "111111111111", region: "global", kind: "AWS::EC2::Instance", id: "shared" }, &bytes).await?;
    let instance = db.detail(&node("AWS::EC2::Instance", "global")).await?.ok_or("missing instance")?;
    let bucket = db.detail(&node("AWS::S3::Bucket", "global")).await?.ok_or("missing bucket")?;
    assert_eq!(instance.record.as_bytes(), bytes.as_slice());
    assert_eq!(bucket.record.as_bytes(), record_bytes()?.as_slice());
    assert_ne!(instance.key.kind, bucket.key.kind);
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_literal_wildcards_and_sql_text_when_filtering_then_bound_values_do_not_expand() -> TestResult {
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let db = populated(&path).await?;
    let pool = writer(&path).await?;
    let text = "%_' OR 1=1 --";
    query("UPDATE resources SET name=? WHERE account='111111111111' AND region='global'").bind(text).execute(&pool).await?;
    let page = db.page(&ResourceFilter { text: Some(text.to_owned()), ..ResourceFilter::default() }, PageRequest { limit: 10, after: None }).await?;
    assert_eq!(page.total_matching, 1);
    assert_eq!(page.resources.first().and_then(|row| row.name.as_deref()), Some(text));
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_untrusted_kind_table_mapping_when_reading_detail_then_identifier_is_rejected() -> TestResult {
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let db = populated(&path).await?;
    let pool = writer(&path).await?;
    query("UPDATE kinds SET table_name=? WHERE kind='AWS::S3::Bucket'").bind("bucket_records\"; DELETE FROM resources; --").execute(&pool).await?;
    assert!(matches!(db.detail(&node("AWS::S3::Bucket", "global")).await, Err(ViewError::SqlName)));
    assert_eq!(db.counts().await?.resources, 4);
    pool.close().await;
    Ok(())
}
