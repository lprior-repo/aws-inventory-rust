use super::*;
use cmdb_view::{EdgeCursor, EdgePageRequest, GraphEdge, GraphNode, ViewError};

fn node(kind: &str, region: &str) -> GraphNode {
    GraphNode {
        account: "111111111111".to_owned(),
        region: region.to_owned(),
        kind: kind.to_owned(),
        id: "shared".to_owned(),
    }
}

#[tokio::test]
async fn given_parallel_edges_and_self_loop_when_paging_then_every_identity_appears_once()
-> TestResult {
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let db = populated(&path).await?;
    let pool = writer(&path).await?;
    let root = node("AWS::S3::Bucket", "global");
    let mut parallel = GraphEdge {
        from: root.clone(),
        to: node("AWS::S3::Bucket", "us-east-1"),
        relation: "replica".to_owned(),
        provenance: "reviewed".to_owned(),
    };
    seed_edge(&pool, &parallel).await?;
    parallel.relation = "references".to_owned();
    seed_edge(&pool, &parallel).await?;
    seed_edge(
        &pool,
        &GraphEdge {
            from: root.clone(),
            to: root.clone(),
            relation: "self".to_owned(),
            provenance: "inferred".to_owned(),
        },
    )
    .await?;
    let mut snapshot = db.snapshot().await?;
    let first = snapshot
        .edges_page(
            &root,
            EdgePageRequest {
                limit: 2,
                after: None,
            },
        )
        .await?;
    let second = snapshot
        .edges_page(
            &root,
            EdgePageRequest {
                limit: 2,
                after: first.next,
            },
        )
        .await?;
    let third = snapshot
        .edges_page(
            &root,
            EdgePageRequest {
                limit: 2,
                after: second.next,
            },
        )
        .await?;
    assert_eq!(
        (
            first.total_matching,
            second.total_matching,
            third.total_matching
        ),
        (5, 5, 5)
    );
    assert!(third.next.is_none());
    let identities = first
        .edges
        .iter()
        .chain(&second.edges)
        .chain(&third.edges)
        .map(|edge| {
            (
                edge.to.region.as_str(),
                edge.relation.as_str(),
                edge.provenance.as_str(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        identities,
        vec![
            ("global", "self", "inferred"),
            ("us-east-1", "references", "reviewed"),
            ("us-east-1", "replica", "catalog"),
            ("us-east-1", "replica", "reviewed"),
            ("eu-west-1", "replica", "catalog")
        ]
    );
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
async fn given_identical_account_region_and_id_when_details_differ_by_kind_then_binary_rows_are_exact()
-> TestResult {
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let db = populated(&path).await?;
    let pool = writer(&path).await?;
    let bytes = named_record("instance-detail")?;
    seed(
        &pool,
        Seed {
            table: "instance_records",
            account: "111111111111",
            region: "global",
            kind: "AWS::EC2::Instance",
            id: "shared",
        },
        &bytes,
    )
    .await?;
    let instance = db
        .detail(&node("AWS::EC2::Instance", "global"))
        .await?
        .ok_or("missing instance")?;
    let bucket = db
        .detail(&node("AWS::S3::Bucket", "global"))
        .await?
        .ok_or("missing bucket")?;
    assert_eq!(instance.record.as_bytes(), bytes.as_slice());
    assert_eq!(bucket.record.as_bytes(), record_bytes()?.as_slice());
    assert_ne!(instance.key.kind, bucket.key.kind);
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_literal_wildcards_and_sql_text_when_filtering_then_bound_values_do_not_expand()
-> TestResult {
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let db = populated(&path).await?;
    let pool = writer(&path).await?;
    let text = "%_' OR 1=1 --";
    query("UPDATE resources SET name=? WHERE account='111111111111' AND region='global'")
        .bind(text)
        .execute(&pool)
        .await?;
    let page = db
        .page(
            &ResourceFilter {
                text: Some(text.to_owned()),
                ..ResourceFilter::default()
            },
            PageRequest {
                limit: 10,
                after: None,
            },
        )
        .await?;
    assert_eq!(page.total_matching, 1);
    assert_eq!(
        page.resources.first().and_then(|row| row.name.as_deref()),
        Some(text)
    );
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_untrusted_kind_table_mapping_when_reading_detail_then_identifier_is_rejected()
-> TestResult {
    let dir = tempdir()?;
    let path = dir.path().join("inventory.sqlite");
    let db = populated(&path).await?;
    let pool = writer(&path).await?;
    query("UPDATE kinds SET table_name=? WHERE kind='AWS::S3::Bucket'")
        .bind("bucket_records\"; DELETE FROM resources; --")
        .execute(&pool)
        .await?;
    assert!(matches!(
        db.detail(&node("AWS::S3::Bucket", "global")).await,
        Err(ViewError::SqlName)
    ));
    assert_eq!(db.counts().await?.resources, 4);
    pool.close().await;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_parallel_edges_and_self_loop_when_paging_then_every_identity_appears_once()
-> TestResult {
    use super::postgres_support as pg;
    // Given real PgStore resources, two existing edges, a parallel edge and a self-loop.
    let fixture = super::support::postgres_fixture().await?;
    let db = fixture.reader().await?;
    let root = pg::root();
    let target = pg::node("111111111111", "us-east-1", 0, "shared");
    fixture
        .edge(&pg::edge(&root, &target, "replica", "reviewed"))
        .await?;
    fixture
        .edge(&pg::edge(&root, &target, "references", "reviewed"))
        .await?;
    fixture
        .edge(&pg::edge(&root, &root, "self", "inferred"))
        .await?;
    // When cursor pages cross relation and provenance identities.
    let first = db.edges_page(&root, pg_edge_request(None)).await?;
    let second = db.edges_page(&root, pg_edge_request(first.next)).await?;
    let third = db.edges_page(&root, pg_edge_request(second.next)).await?;
    // Then every exact edge appears once, including the self-loop.
    assert_eq!(
        (
            first.total_matching,
            second.total_matching,
            third.total_matching
        ),
        (5, 5, 5)
    );
    let identities = first
        .edges
        .iter()
        .chain(&second.edges)
        .chain(&third.edges)
        .map(|edge| {
            (
                edge.to.region.as_str(),
                edge.relation.as_str(),
                edge.provenance.as_str(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        identities,
        vec![
            ("global", "self", "inferred"),
            ("us-east-1", "references", "reviewed"),
            ("us-east-1", "replica", "catalog"),
            ("us-east-1", "replica", "reviewed"),
            ("eu-west-1", "replica", "catalog")
        ]
    );
    assert_eq!(third.next, None);
    drop(db);
    fixture.finish().await
}

fn pg_edge_request(after: Option<EdgeCursor>) -> EdgePageRequest {
    EdgePageRequest { limit: 2, after }
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_same_identity_in_two_kinds_when_reading_then_records_remain_exact()
-> TestResult {
    use super::postgres_support as pg;
    // Given the same account, region and ID stored through two distinct kind tables.
    let fixture = super::support::postgres_fixture().await?;
    let padded = pg_pad_other_kind(&fixture).await?;
    let db = fixture.reader().await?;
    // When reading both kinds and a lightweight page.
    let bucket = db.detail(&pg::root()).await?.ok_or("missing kind zero")?;
    let other = db
        .detail(&pg::node("111111111111", "global", 1, "shared"))
        .await?
        .ok_or("missing kind one")?;
    let page = db
        .page_rows(
            &ResourceFilter::default(),
            PageRequest {
                limit: 10,
                after: None,
            },
        )
        .await?;
    // Then the payload, hash and byte length all follow the complete key.
    assert_eq!(bucket.record, pg::record("shared", 10)?);
    assert_eq!(other.record, padded);
    assert_ne!(
        bucket.record.as_bytes().len(),
        other.record.as_bytes().len()
    );
    assert_eq!(
        other.content_hash,
        other
            .record
            .content_hash(&inventory_core::record::Exclusions::none())?
    );
    assert_eq!(page.total_matching, 5);
    let lengths = page
        .rows
        .iter()
        .map(|row| (row.key.clone(), row.record_bytes))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        lengths.get(&bucket.key),
        Some(&bucket.record.as_bytes().len())
    );
    assert_eq!(
        lengths.get(&other.key),
        Some(&other.record.as_bytes().len())
    );
    assert_eq!(page.next, None);
    drop(db);
    fixture.finish().await
}

async fn pg_pad_other_kind(
    fixture: &super::postgres_support::Fixture,
) -> TestResult<inventory_core::record::Record> {
    fixture
        .commit("other-kind", "111111111111", "global", 1, &["shared"], 50)
        .await?;
    let padded = super::parity::record_with_padding("shared", 50, 40)?;
    let hash = padded.content_hash(&inventory_core::record::Exclusions::none())?;
    sqlx::query("UPDATE inventory.kind_1 SET record=$1,content_hash=$2 WHERE region='global'")
        .bind(padded.as_bytes())
        .bind(hash.as_bytes().as_slice())
        .execute(&fixture.pool)
        .await?;
    sqlx::query("UPDATE inventory.resources SET content_hash=$1 WHERE region='global' AND kind='AWS::Example::Kind1'")
        .bind(hash.as_bytes().as_slice()).execute(&fixture.pool).await?;
    Ok(padded)
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_literal_wildcards_when_searching_then_values_never_expand() -> TestResult {
    // Given punctuation including the explicit LIKE escape character.
    let fixture = super::support::postgres_fixture().await?;
    fixture
        .commit(
            "search-decoy",
            "111111111111",
            "global",
            0,
            &["shared", "decoy"],
            50,
        )
        .await?;
    let text = "%_!' OR 1=1 --\\";
    sqlx::query("UPDATE inventory.resources SET name=$1 WHERE region='global' AND id='shared'")
        .bind(text)
        .execute(&fixture.pool)
        .await?;
    sqlx::query("UPDATE inventory.resources SET name=$1 WHERE region='global' AND id='decoy'")
        .bind("abc!' OR 1=1 --\\")
        .execute(&fixture.pool)
        .await?;
    let db = fixture.reader().await?;
    // When a case-insensitive search combines identity and literal text.
    let page = db
        .page(
            &ResourceFilter {
                account: Some("111111111111".into()),
                region: Some("global".into()),
                kind: Some("AWS::Example::Kind0".into()),
                text: Some(text.to_lowercase()),
                tag: None,
            },
            PageRequest {
                limit: 10,
                after: None,
            },
        )
        .await?;
    // Then only the one literal match is returned.
    assert_eq!(page.total_matching, 1);
    assert_eq!(
        page.resources
            .iter()
            .map(|row| row.name.as_deref())
            .collect::<Vec<_>>(),
        vec![Some(text)]
    );
    drop(db);
    fixture.finish().await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_untrusted_kind_table_when_reading_detail_then_identifier_is_rejected()
-> TestResult {
    use super::postgres_support as pg;
    // Given a corrupted table mapping in the documented kinds table.
    let fixture = super::support::postgres_fixture().await?;
    sqlx::query("UPDATE inventory.kinds SET table_name=$1 WHERE kind='AWS::Example::Kind0'")
        .bind("kind_0\"; DELETE FROM resources; --")
        .execute(&fixture.pool)
        .await?;
    let db = fixture.reader().await?;
    let error = db
        .detail(&pg::root())
        .await
        .err()
        .ok_or("expected invalid identifier")?;
    assert_eq!(error.to_string(), "stored SQL table name is invalid");
    assert_eq!(db.counts().await?.resources, 4);
    drop(db);
    fixture.finish().await
}
