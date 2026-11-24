use super::{Database, PageRequest, ResourceFilter, Seed, TestResult, postgres_support as pg};
use cmdb_view::{DatabaseCounts, LodSummary, ResourceRow, TagFilter};
use inventory_core::record::{Exclusions, FieldId, Key, Record, Value};
use sqlx::SqlitePool;
use std::collections::BTreeMap;

struct Pair {
    pg: pg::Fixture,
    sqlite: SqlitePool,
    directory: tempfile::TempDir,
}

impl Pair {
    async fn new() -> TestResult<Self> {
        let pg = pg::Fixture::new().await?;
        let directory = tempfile::tempdir()?;
        let sqlite = super::schema15(&directory.path().join("inventory.sqlite")).await?;
        let reader = pg.reader().await?;
        let page = reader
            .page(&ResourceFilter::default(), request(None))
            .await?;
        for resource in page.resources {
            let key = resource.key;
            let table = if key.kind.as_str() == "AWS::Example::Kind0" {
                "bucket_records"
            } else {
                "instance_records"
            };
            super::seed(
                &sqlite,
                Seed {
                    table,
                    account: key.account.as_str(),
                    region: key.region.as_str(),
                    kind: key.kind.as_str(),
                    id: key.id.as_str(),
                },
                resource.record.as_bytes(),
            )
            .await?;
        }
        for target in [
            pg::node("111111111111", "us-east-1", 0, "shared"),
            pg::node("222222222222", "eu-west-1", 0, "shared"),
        ] {
            super::seed_edge(
                &sqlite,
                &pg::edge(&pg::root(), &target, "replica", "catalog"),
            )
            .await?;
        }
        Ok(Self {
            pg,
            sqlite,
            directory,
        })
    }

    async fn readers(&self) -> TestResult<[Database; 2]> {
        let path = self.directory.path().join("inventory.sqlite");
        Ok([
            Database::open(path.to_str().ok_or("non-UTF-8 path")?).await?,
            self.pg.reader().await?,
        ])
    }

    async fn records(&self, record: &Record) -> TestResult {
        let hash = record.content_hash(&Exclusions::none())?;
        sqlx::query("UPDATE inventory.kind_0 SET record=$1,content_hash=$2")
            .bind(record.as_bytes())
            .bind(hash.as_bytes().as_slice())
            .execute(&self.pg.pool)
            .await?;
        sqlx::query(
            "UPDATE inventory.resources SET content_hash=$1 WHERE kind='AWS::Example::Kind0'",
        )
        .bind(hash.as_bytes().as_slice())
        .execute(&self.pg.pool)
        .await?;
        sqlx::query("UPDATE bucket_records SET record=?,content_hash=?")
            .bind(record.as_bytes())
            .bind(hash.as_bytes().as_slice())
            .execute(&self.sqlite)
            .await?;
        sqlx::query("UPDATE resources SET content_hash=? WHERE kind='AWS::Example::Kind0'")
            .bind(hash.as_bytes().as_slice())
            .execute(&self.sqlite)
            .await?;
        Ok(())
    }

    async fn finish(self) -> TestResult {
        self.sqlite.close().await;
        self.pg.finish().await
    }
}

pub(super) fn record_with_padding(id: &str, at: i64, padding: usize) -> TestResult<Record> {
    let fields = [
        (2, Value::Int(at)),
        (4, Value::Text(id.to_owned())),
        (5, Value::Bytes(vec![0; padding])),
    ]
    .into_iter()
    .map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
    .collect::<TestResult<BTreeMap<_, _>>>()?;
    Ok(Record::encode(&Value::Map(fields))?)
}

fn request(after: Option<cmdb_view::PageCursor>) -> PageRequest {
    PageRequest { limit: 1000, after }
}

fn locations(rows: &[ResourceRow]) -> Vec<(String, String)> {
    rows.iter()
        .map(|row| {
            (
                row.key.account.as_str().to_owned(),
                row.key.region.as_str().to_owned(),
            )
        })
        .collect()
}

async fn budget_pages(reader: &Database, bytes: usize) -> TestResult {
    let mut after = None;
    let mut all = Vec::new();
    let mut pages = Vec::new();
    for _ in 0..3 {
        let page = reader
            .page_rows(&ResourceFilter::default(), request(after))
            .await?;
        assert_eq!(page.total_matching, 4);
        for row in page
            .rows
            .iter()
            .filter(|row| row.key.kind.as_str() == "AWS::Example::Kind0")
        {
            assert_eq!(row.record_bytes, bytes);
        }
        pages.push((page.rows.len(), page.truncated, page.next.is_some()));
        all.extend(locations(&page.rows));
        after = page.next;
    }
    assert_eq!(
        pages,
        vec![(1, true, true), (1, true, true), (2, false, false)]
    );
    assert_eq!(
        all,
        vec![
            ("111111111111".into(), "global".into()),
            ("111111111111".into(), "us-east-1".into()),
            ("222222222222".into(), "eu-west-1".into()),
            ("222222222222".into(), "us-west-2".into())
        ]
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_and_sqlite_given_budget_truncation_when_paging_then_every_row_remains_reachable()
-> TestResult {
    let pair = Pair::new().await?;
    let record = record_with_padding("shared", 10, 6_291_456)?;
    pair.records(&record).await?;
    for reader in pair.readers().await? {
        budget_pages(&reader, record.as_bytes().len()).await?;
    }
    pair.finish().await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_and_sqlite_given_oversized_first_row_when_paging_then_budget_failure_is_explicit()
-> TestResult {
    let pair = Pair::new().await?;
    pair.records(&record_with_padding("shared", 10, 6_291_456)?)
        .await?;
    let oversized_value = "x".repeat(5_242_880);
    sqlx::query("INSERT INTO inventory.tags SELECT account,region,kind,id,'oversized',$1 FROM inventory.resources WHERE region='global'")
        .bind(oversized_value.as_str()).execute(&pair.pg.pool).await?;
    sqlx::query("INSERT INTO tags SELECT account,region,kind,id,'oversized',? FROM resources WHERE region='global'")
        .bind(oversized_value.as_str()).execute(&pair.sqlite).await?;
    for reader in pair.readers().await? {
        let error = reader
            .page_rows(&ResourceFilter::default(), request(None))
            .await
            .err()
            .ok_or("expected budget error")?;
        assert_eq!(
            error.to_string(),
            "resource row exceeds the 10 MiB page budget"
        );
    }
    pair.finish().await
}

async fn missing_record(reader: &Database) -> TestResult {
    let error = reader
        .page_rows(&ResourceFilter::default(), request(None))
        .await
        .err()
        .ok_or("expected missing row")?;
    assert_eq!(error.to_string(), "invalid stored resource row");
    let error = reader
        .page(&ResourceFilter::default(), request(None))
        .await
        .err()
        .ok_or("expected missing payload")?;
    assert_eq!(error.to_string(), "invalid stored resource row");
    let error = reader
        .detail(&pg::root())
        .await
        .err()
        .ok_or("expected missing detail")?;
    assert_eq!(error.to_string(), "invalid stored resource row");
    assert_eq!(
        reader
            .detail(&pg::node("111111111111", "global", 0, "absent"))
            .await?,
        None
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_and_sqlite_given_missing_record_or_mapping_when_reading_then_failure_is_typed()
-> TestResult {
    let pair = Pair::new().await?;
    sqlx::query("DELETE FROM inventory.kind_0")
        .execute(&pair.pg.pool)
        .await?;
    sqlx::query("DELETE FROM bucket_records")
        .execute(&pair.sqlite)
        .await?;
    for reader in pair.readers().await? {
        missing_record(&reader).await?;
    }
    sqlx::query("DELETE FROM inventory.kinds WHERE kind='AWS::Example::Kind0'")
        .execute(&pair.pg.pool)
        .await?;
    sqlx::query("DELETE FROM kinds WHERE kind='AWS::Example::Kind0'")
        .execute(&pair.sqlite)
        .await?;
    for reader in pair.readers().await? {
        missing_record(&reader).await?;
    }
    pair.finish().await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_and_sqlite_given_empty_store_when_reading_then_counts_pages_details_and_lod_are_empty()
-> TestResult {
    let pair = Pair::new().await?;
    sqlx::query("DELETE FROM inventory.resources")
        .execute(&pair.pg.pool)
        .await?;
    sqlx::query("DELETE FROM resources")
        .execute(&pair.sqlite)
        .await?;
    for reader in pair.readers().await? {
        assert_eq!(reader.counts().await?, DatabaseCounts::default());
        assert_eq!(reader.lod_summary().await?, LodSummary::default());
        let page = reader
            .page(&ResourceFilter::default(), request(None))
            .await?;
        assert_eq!(
            (page.total_matching, page.resources, page.next),
            (0, Vec::new(), None)
        );
        let rows = reader
            .page_rows(&ResourceFilter::default(), request(None))
            .await?;
        assert_eq!(
            (rows.total_matching, rows.rows, rows.next, rows.truncated),
            (0, Vec::new(), None, false)
        );
        let edges = reader
            .edges_page(
                &pg::root(),
                cmdb_view::EdgePageRequest {
                    limit: 1000,
                    after: None,
                },
            )
            .await?;
        assert_eq!(
            (edges.total_matching, edges.edges, edges.next),
            (0, Vec::new(), None)
        );
        assert_eq!(reader.detail(&pg::root()).await?, None);
    }
    pair.finish().await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_and_sqlite_given_independent_filters_when_reading_then_each_filter_constrains_rows()
-> TestResult {
    let pair = Pair::new().await?;
    let filters = [
        (
            ResourceFilter {
                account: Some("111111111111".into()),
                ..ResourceFilter::default()
            },
            vec![
                ("111111111111".into(), "global".into()),
                ("111111111111".into(), "us-east-1".into()),
            ],
        ),
        (
            ResourceFilter {
                region: Some("eu-west-1".into()),
                ..ResourceFilter::default()
            },
            vec![("222222222222".into(), "eu-west-1".into())],
        ),
        (
            ResourceFilter {
                kind: Some("AWS::Example::Kind1".into()),
                ..ResourceFilter::default()
            },
            vec![("222222222222".into(), "us-west-2".into())],
        ),
    ];
    for reader in pair.readers().await? {
        assert_eq!(
            reader.counts().await?,
            DatabaseCounts {
                accounts: 2,
                regions: 4,
                services: 1,
                resources: 4
            }
        );
        assert_eq!(
            reader.lod_summary().await?,
            LodSummary {
                resources: 4,
                edges: 2,
                connected_resources: 3,
                isolated_resources: 1
            }
        );
        for (filter, expected) in &filters {
            let page = reader.page_rows(filter, request(None)).await?;
            assert_eq!(page.total_matching, u64::try_from(expected.len())?);
            assert_eq!(&locations(&page.rows), expected);
            let page = reader.page(filter, request(None)).await?;
            assert_eq!(page.total_matching, u64::try_from(expected.len())?);
        }
    }
    pair.finish().await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_and_sqlite_given_one_row_pages_when_following_keysets_then_order_and_totals_are_exact()
-> TestResult {
    let pair = Pair::new().await?;
    for reader in pair.readers().await? {
        let mut after = None;
        let mut all = Vec::new();
        for position in 0..4 {
            let page = reader
                .page_rows(&ResourceFilter::default(), PageRequest { limit: 1, after })
                .await?;
            assert_eq!(
                (page.total_matching, page.rows.len(), page.truncated),
                (4, 1, false)
            );
            assert_eq!(page.next.is_some(), position < 3);
            all.extend(locations(&page.rows));
            after = page.next;
        }
        assert_eq!(
            all,
            vec![
                ("111111111111".into(), "global".into()),
                ("111111111111".into(), "us-east-1".into()),
                ("222222222222".into(), "eu-west-1".into()),
                ("222222222222".into(), "us-west-2".into())
            ]
        );
    }
    pair.finish().await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_and_sqlite_given_tag_decoys_when_filtering_then_key_value_and_empty_value_are_exact()
-> TestResult {
    let pair = Pair::new().await?;
    sqlx::query("INSERT INTO inventory.tags SELECT account,region,kind,id,CASE WHEN region='eu-west-1' THEN 'envAb!' ELSE 'env%_!' END,CASE region WHEN 'global' THEN 'Production' WHEN 'us-east-1' THEN 'Staging' WHEN 'eu-west-1' THEN 'Production' ELSE '' END FROM inventory.resources").execute(&pair.pg.pool).await?;
    sqlx::query("INSERT INTO tags SELECT account,region,kind,id,CASE WHEN region='eu-west-1' THEN 'envAb!' ELSE 'env%_!' END,CASE region WHEN 'global' THEN 'Production' WHEN 'us-east-1' THEN 'Staging' WHEN 'eu-west-1' THEN 'Production' ELSE '' END FROM resources").execute(&pair.sqlite).await?;
    for reader in pair.readers().await? {
        for (value, regions) in [
            (Some("Production"), vec!["global"]),
            (None, vec!["global", "us-east-1", "us-west-2"]),
            (Some(""), vec!["us-west-2"]),
        ] {
            let filter = ResourceFilter {
                tag: Some(TagFilter {
                    key: "env%_!".into(),
                    value: value.map(str::to_owned),
                }),
                ..ResourceFilter::default()
            };
            let page = reader.page_rows(&filter, request(None)).await?;
            assert_eq!(page.total_matching, u64::try_from(regions.len())?);
            assert_eq!(
                page.rows
                    .iter()
                    .map(|row| row.key.region.as_str())
                    .collect::<Vec<_>>(),
                regions
            );
        }
    }
    pair.finish().await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_and_sqlite_given_excess_tags_when_reading_then_caps_are_explicit_without_false_cursor()
-> TestResult {
    let pair = Pair::new().await?;
    sqlx::query("INSERT INTO inventory.tags SELECT account,region,kind,id,'t'||lpad(i::text,4,'0'),'v' FROM inventory.resources CROSS JOIN generate_series(0,1004) i WHERE region='global'").execute(&pair.pg.pool).await?;
    sqlx::query("WITH RECURSIVE n(i) AS (VALUES(0) UNION ALL SELECT i+1 FROM n WHERE i<1004) INSERT INTO tags SELECT account,region,kind,id,printf('t%04d',i),'v' FROM resources CROSS JOIN n WHERE region='global'").execute(&pair.sqlite).await?;
    for reader in pair.readers().await? {
        let detail = reader
            .detail(&pg::root())
            .await?
            .ok_or("missing tagged detail")?;
        assert_eq!((detail.tags.len(), detail.tags_truncated), (1000, true));
        assert_eq!(
            detail.tags.first().map(|tag| tag.key.as_str()),
            Some("t0000")
        );
        assert_eq!(
            detail.tags.last().map(|tag| tag.key.as_str()),
            Some("t0999")
        );
        let page = reader
            .page_rows(&ResourceFilter::default(), request(None))
            .await?;
        assert_eq!(
            (page.rows.len(), page.truncated, page.next),
            (4, true, None)
        );
        let row = page.rows.first().ok_or("missing tagged row")?;
        assert_eq!((row.tags.len(), row.tags_truncated), (1000, true));
        assert_eq!(row.tags, detail.tags);
    }
    pair.finish().await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_and_sqlite_given_case_variants_when_searching_then_only_ascii_case_folds()
-> TestResult {
    let pair = Pair::new().await?;
    sqlx::query("UPDATE inventory.resources SET name='Ä Ab' WHERE region='global'")
        .execute(&pair.pg.pool)
        .await?;
    sqlx::query("UPDATE resources SET name='Ä Ab' WHERE region='global'")
        .execute(&pair.sqlite)
        .await?;
    for reader in pair.readers().await? {
        for (text, expected) in [("ä", 0), ("aB", 1)] {
            let filter = ResourceFilter {
                text: Some(text.into()),
                ..ResourceFilter::default()
            };
            assert_eq!(
                reader
                    .page_rows(&filter, request(None))
                    .await?
                    .total_matching,
                expected
            );
            assert_eq!(
                reader.page(&filter, request(None)).await?.total_matching,
                expected
            );
        }
    }
    pair.finish().await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_and_sqlite_given_wildcard_decoys_when_searching_then_escape_characters_are_literal()
-> TestResult {
    let pair = Pair::new().await?;
    sqlx::query("UPDATE inventory.resources SET name=CASE region WHEN 'global' THEN 'literal!%_token' ELSE 'literalXXtoken' END WHERE account='111111111111'").execute(&pair.pg.pool).await?;
    sqlx::query("UPDATE resources SET name=CASE region WHEN 'global' THEN 'literal!%_token' ELSE 'literalXXtoken' END WHERE account='111111111111'").execute(&pair.sqlite).await?;
    for reader in pair.readers().await? {
        for text in ["!", "%", "_"] {
            let filter = ResourceFilter {
                account: Some("111111111111".into()),
                text: Some(text.into()),
                ..ResourceFilter::default()
            };
            let page = reader.page_rows(&filter, request(None)).await?;
            assert_eq!(page.total_matching, 1);
            assert_eq!(
                locations(&page.rows),
                vec![("111111111111".into(), "global".into())]
            );
            assert_eq!(
                page.rows.first().and_then(|row| row.name.as_deref()),
                Some("literal!%_token")
            );
            let page = reader.page(&filter, request(None)).await?;
            assert_eq!(page.total_matching, 1);
            assert_eq!(
                page.resources.first().map(|row| row.key.region.as_str()),
                Some("global")
            );
        }
    }
    pair.finish().await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_and_sqlite_given_exact_tag_limit_when_reading_then_no_truncation_is_reported()
-> TestResult {
    let pair = Pair::new().await?;
    sqlx::query("INSERT INTO inventory.tags SELECT account,region,kind,id,'t'||lpad(i::text,4,'0'),'v' FROM inventory.resources CROSS JOIN generate_series(0,999) i WHERE region='global'").execute(&pair.pg.pool).await?;
    sqlx::query("WITH RECURSIVE n(i) AS (VALUES(0) UNION ALL SELECT i+1 FROM n WHERE i<999) INSERT INTO tags SELECT account,region,kind,id,printf('t%04d',i),'v' FROM resources CROSS JOIN n WHERE region='global'").execute(&pair.sqlite).await?;
    for reader in pair.readers().await? {
        let detail = reader
            .detail(&pg::root())
            .await?
            .ok_or("missing tagged detail")?;
        assert_eq!((detail.tags.len(), detail.tags_truncated), (1000, false));
        assert_eq!(
            detail.tags.first().map(|tag| tag.key.as_str()),
            Some("t0000")
        );
        assert_eq!(
            detail.tags.last().map(|tag| tag.key.as_str()),
            Some("t0999")
        );
        let page = reader
            .page_rows(&ResourceFilter::default(), request(None))
            .await?;
        assert_eq!(
            (page.rows.len(), page.truncated, page.next),
            (4, false, None)
        );
        let row = page.rows.first().ok_or("missing tagged row")?;
        assert_eq!((row.tags.len(), row.tags_truncated), (1000, false));
        assert_eq!(row.tags, detail.tags);
        let page = reader
            .page(&ResourceFilter::default(), request(None))
            .await?;
        assert_eq!((page.resources.len(), page.next), (4, None));
        let resource = page.resources.first().ok_or("missing tagged resource")?;
        assert_eq!(
            (resource.tags.len(), resource.tags_truncated),
            (1000, false)
        );
        assert_eq!(resource.tags, detail.tags);
    }
    pair.finish().await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_and_sqlite_given_out_of_order_tags_when_reading_then_sorted_pairs_stay_associated()
-> TestResult {
    let pair = Pair::new().await?;
    for (key, value) in [
        ("z", "first"),
        ("Alpha", "last"),
        ("a", ""),
        ("é", "unicode"),
        ("!", "punctuation"),
        ("a,b", "quoted,\"value"),
        ("a\"", "backslash\\value"),
    ] {
        sqlx::query("INSERT INTO inventory.tags SELECT account,region,kind,id,$1,$2 FROM inventory.resources WHERE region='global'")
            .bind(key).bind(value).execute(&pair.pg.pool).await?;
        sqlx::query("INSERT INTO tags SELECT account,region,kind,id,?,? FROM resources WHERE region='global'")
            .bind(key).bind(value).execute(&pair.sqlite).await?;
    }
    let expected = [
        ("!", "punctuation"),
        ("Alpha", "last"),
        ("a", ""),
        ("a\"", "backslash\\value"),
        ("a,b", "quoted,\"value"),
        ("z", "first"),
        ("é", "unicode"),
    ]
    .map(|(key, value)| cmdb_view::ResourceTag {
        key: key.into(),
        value: value.into(),
    })
    .to_vec();
    for reader in pair.readers().await? {
        let detail = reader
            .detail(&pg::root())
            .await?
            .ok_or("missing tagged detail")?;
        assert_eq!(detail.tags, expected);
        assert_eq!((detail.tags.len(), detail.tags_truncated), (7, false));
        let page = reader
            .page_rows(&ResourceFilter::default(), request(None))
            .await?;
        let row = page.rows.first().ok_or("missing tagged row")?;
        assert_eq!(row.tags, expected);
        assert_eq!((row.tags.len(), row.tags_truncated), (7, false));
        let page = reader
            .page(&ResourceFilter::default(), request(None))
            .await?;
        let resource = page.resources.first().ok_or("missing tagged resource")?;
        assert_eq!(resource.tags, expected);
        assert_eq!((resource.tags.len(), resource.tags_truncated), (7, false));
    }
    pair.finish().await
}
