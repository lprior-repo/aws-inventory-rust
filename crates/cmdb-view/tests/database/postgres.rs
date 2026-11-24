use super::postgres_support as pg;
use super::{PageRequest, ResourceFilter, TestResult};
use cmdb_view::{DatabaseCounts, EdgePageRequest, ResourceTag, StoredTagObservation, TagFilter};
use inventory_core::inventory::TagCoverage;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_tag_filter_when_writer_changes_tags_then_snapshot_keeps_metadata()
-> TestResult {
    // Given real swept resources and literal tag metadata in the documented tables.
    let fixture = pg::Fixture::new().await?;
    sqlx::query("INSERT INTO inventory.tags SELECT account,region,kind,id,$1,$2 FROM inventory.resources WHERE region='global'")
        .bind("env%_!").bind("Production").execute(&fixture.pool).await?;
    sqlx::query("INSERT INTO inventory.tag_observations SELECT account,region,kind,id,'complete',10,10 FROM inventory.resources WHERE region='global' ON CONFLICT(account,region,kind,id) DO UPDATE SET coverage='complete',observed_at=10,last_complete_at=10")
        .execute(&fixture.pool).await?;
    let db = fixture.reader().await?;
    let mut snapshot = db.snapshot().await?;
    let filter = ResourceFilter {
        tag: Some(TagFilter {
            key: "env%_!".into(),
            value: Some("Production".into()),
        }),
        ..ResourceFilter::default()
    };
    // When another connection replaces tag values and coverage.
    sqlx::query("UPDATE inventory.tags SET tag_value='Staging'")
        .execute(&fixture.pool)
        .await?;
    sqlx::query("UPDATE inventory.tag_observations SET coverage='partial',observed_at=20")
        .execute(&fixture.pool)
        .await?;
    let page = snapshot.page_rows(&filter, tag_page_request()).await?;
    let detail = snapshot
        .detail(&pg::root())
        .await?
        .ok_or("missing pinned detail")?;
    // Then filtering and both projections retain the same complete observation.
    assert_eq!(page.total_matching, 1);
    let row = page.rows.first().ok_or("missing pinned row")?;
    assert_tag_metadata(
        &row.tags,
        row.tag_observation.as_ref(),
        "Production",
        TagCoverage::Complete,
        10,
    );
    assert_eq!(detail.tags, row.tags);
    assert_eq!(detail.tag_observation, row.tag_observation);
    snapshot.finish().await?;
    assert_current_tag_state(&db, &filter).await?;
    drop(db);
    fixture.finish().await
}

fn tag_page_request() -> PageRequest {
    PageRequest {
        limit: 10,
        after: None,
    }
}

fn assert_tag_metadata(
    tags: &[ResourceTag],
    observation: Option<&StoredTagObservation>,
    value: &str,
    coverage: TagCoverage,
    observed: i64,
) {
    assert_eq!(
        tags,
        &[ResourceTag {
            key: "env%_!".into(),
            value: value.into()
        }]
    );
    assert_eq!(
        observation,
        Some(&StoredTagObservation {
            coverage,
            observed_at_ms: observed,
            last_complete_at_ms: Some(10)
        })
    );
}

async fn assert_current_tag_state(db: &cmdb_view::Database, filter: &ResourceFilter) -> TestResult {
    assert_eq!(
        db.page_rows(filter, tag_page_request())
            .await?
            .total_matching,
        0
    );
    let current = db
        .detail(&pg::root())
        .await?
        .ok_or("missing current detail")?;
    assert_tag_metadata(
        &current.tags,
        current.tag_observation.as_ref(),
        "Staging",
        TagCoverage::Partial,
        20,
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_invalid_page_limits_when_reading_then_every_page_api_rejects_them()
-> TestResult {
    let fixture = pg::Fixture::new().await?;
    let db = fixture.reader().await?;
    assert_eq!(
        db.counts().await?,
        DatabaseCounts {
            accounts: 2,
            regions: 4,
            services: 1,
            resources: 4
        }
    );
    for limit in [0, 1001] {
        let error = db
            .page(
                &ResourceFilter::default(),
                PageRequest { limit, after: None },
            )
            .await
            .err()
            .ok_or("expected page limit error")?;
        assert_eq!(error.to_string(), "page limit must be in 1..=1000");
        let error = db
            .page_rows(
                &ResourceFilter::default(),
                PageRequest { limit, after: None },
            )
            .await
            .err()
            .ok_or("expected row limit error")?;
        assert_eq!(error.to_string(), "page limit must be in 1..=1000");
        let error = db
            .edges_page(&pg::root(), EdgePageRequest { limit, after: None })
            .await
            .err()
            .ok_or("expected edge limit error")?;
        assert_eq!(error.to_string(), "page limit must be in 1..=1000");
        let error = db
            .scope_freshness(100, limit)
            .await
            .err()
            .ok_or("expected freshness limit error")?;
        assert_eq!(error.to_string(), "page limit must be in 1..=1000");
    }
    drop(db);
    fixture.finish().await
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn postgres_given_equal_scope_times_when_limiting_freshness_then_text_order_precedes_encoded_lengths()
-> TestResult {
    // Given a lexical-first region whose encoded length exceeds global's length.
    let fixture = pg::Fixture::new().await?;
    fixture
        .commit("lexical", "111111111111", "ap-east-1", 0, &["shared"], 50)
        .await?;
    sqlx::query("UPDATE inventory.scope_outcomes SET observed_at=10,outcome='complete'")
        .execute(&fixture.pool)
        .await?;
    let db = fixture.reader().await?;
    // When the report selects only the oldest first scope.
    let report = db.scope_freshness(100, 1).await?;
    // Then selection happens in public field order, before the SQL limit.
    assert_eq!(report.total_scopes, 5);
    assert_eq!(report.counts.complete, 5);
    assert_eq!(
        report
            .scopes
            .iter()
            .map(|scope| (
                scope.account.as_str(),
                scope.region.as_str(),
                scope.operation.as_str(),
                scope.age_ms,
                scope.complete_age_ms
            ))
            .collect::<Vec<_>>(),
        vec![("111111111111", "ap-east-1", "ListThings0", 90, Some(90))]
    );
    drop(db);
    fixture.finish().await
}
