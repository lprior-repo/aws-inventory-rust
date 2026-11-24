use super::*;
use inventory_core::record::{Exclusions, Value};

async fn populated_database() -> TestResult<(TestDatabase, Record)> {
    let database = TestDatabase::create(&admin_url()?)
        .await
        .map_err(|error| format!("batch fixture database creation: {error}"))?;
    let (registry, _) = catalog::registry(3, None)?;
    let store = PgStore::create(
        &DatabaseUrl::parse(database.url())?,
        registry,
        PgWriteConfig::default(),
    )
    .await
    .map_err(|error| format!("batch fixture schema creation: {error}"))?;
    store.close().await;
    let writer = PgPool::connect(database.url()).await?;
    let record = Record::encode(&Value::Map(BTreeMap::new()))?;
    let hash = record.content_hash(&Exclusions::default())?;
    for (index, count) in [(0, 167_i32), (1, 167), (2, 166)] {
        let kind = format!("AWS::Example::Kind{index}");
        let table = SqlName::parse(&format!("kind_{index}"))?;
        sqlx::query("INSERT INTO inventory.resources(account,region,kind,id,name,content_hash,first_seen,last_changed) SELECT '111111111111','us-east-1',$1,'id-'||g::text,'name-'||g::text,$2,1,1 FROM generate_series(1,$3) g")
            .bind(&kind).bind(hash.as_bytes().as_slice()).bind(count).execute(&writer).await?;
        let statement = format!(
            "INSERT INTO inventory.\"{}\"(account,region,id,content_hash,first_seen,last_changed,record) SELECT '111111111111','us-east-1','id-'||g::text,$1,1,1,$2 FROM generate_series(1,$3) g",
            table.as_str()
        );
        sqlx::query(sqlx::AssertSqlSafe(statement.as_str()))
            .bind(hash.as_bytes().as_slice())
            .bind(record.as_bytes())
            .bind(count)
            .execute(&writer)
            .await?;
    }
    writer.close().await;
    Ok((database, record))
}

fn request() -> PageRequest {
    PageRequest {
        limit: 500,
        after: None,
    }
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_five_hundred_rows_over_three_kinds_when_paging_then_statement_count_is_bounded()
-> TestResult {
    let (database, record) = populated_database().await?;
    let reader = Database::open(database.url())
        .await
        .map_err(|error| format!("batch fixture reader open: {error}"))?;
    let mut snapshot = reader
        .snapshot()
        .await
        .map_err(|error| format!("batch fixture snapshot begin: {error}"))?;
    let filter = ResourceFilter::default();
    probes::reset_queries();
    let page = snapshot.page(&filter, request()).await?;
    assert_eq!(page.resources.len(), 500);
    assert_eq!(page.total_matching, 500);
    assert_eq!(page.next, None);
    assert_eq!(
        probes::query_count(),
        8,
        "count and metadata plus one table lookup and payload read per kind"
    );
    let mut counts = BTreeMap::new();
    for resource in &page.resources {
        assert_eq!(resource.record, record);
        let count = counts.entry(resource.key.kind.as_str()).or_insert(0_usize);
        *count = count.saturating_add(1);
    }
    assert_eq!(
        counts,
        BTreeMap::from([
            ("AWS::Example::Kind0", 167),
            ("AWS::Example::Kind1", 167),
            ("AWS::Example::Kind2", 166)
        ])
    );
    probes::reset_queries();
    let again = snapshot.page(&filter, request()).await?;
    assert_eq!(again, page);
    assert_eq!(
        probes::query_count(),
        5,
        "kind tables must be cached in the snapshot"
    );
    probes::reset_queries();
    let rows = snapshot.page_rows(&filter, request()).await?;
    assert_eq!(rows.rows.len(), 500);
    assert_eq!(
        probes::query_count(),
        5,
        "row-page lengths must reuse the same kind cache"
    );
    snapshot.finish().await?;
    reader.pool.close().await;
    database.finish().await?;
    Ok(())
}
