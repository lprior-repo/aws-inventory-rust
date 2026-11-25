#[path = "support/tag_writer.rs"]
mod support;

use cmdb_view::{
    Database, PageCursor, ResourceFilter, ResourceTag, StoredResource, StoredTagObservation,
    ViewError,
};
use inventory_core::inventory::{Observation, TagCoverage};
use inventory_engine::inventory::BinaryStore;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use support::*;

fn assert_metadata(actual: &StoredResource, expected: &Observation) {
    assert_eq!(&actual.key, expected.key());
    assert_eq!(actual.name.as_deref(), expected.name());
    assert_eq!(actual.content_hash, expected.content_hash());
    assert_eq!(&actual.record, expected.record());
    assert!(!actual.tags_truncated);
}

fn status(
    coverage: TagCoverage,
    observed_at_ms: i64,
    last_complete_at_ms: Option<i64>,
) -> Option<StoredTagObservation> {
    Some(StoredTagObservation {
        coverage,
        observed_at_ms,
        last_complete_at_ms,
    })
}

#[tokio::test]
async fn given_account_only_and_global_identity_collisions_when_reading_tags_then_all_four_identity_components_are_bound()
-> TestResult {
    let fixture = fixture().await?;
    for (sweep, account, region, value) in [
        ("own-regional", ACCOUNT, REGION, "own-region"),
        ("foreign-regional", FOREIGN, REGION, "foreign-region"),
        ("own-global", ACCOUNT, "global", "own-global"),
        ("foreign-global", FOREIGN, "global", "foreign-global"),
    ] {
        commit(
            &fixture.store,
            sweep,
            account,
            region,
            vec![observation(account, region, "shared", value)?],
            vec![tags(
                account,
                region,
                "shared",
                TagCoverage::Complete,
                &[("owner", value)],
            )?],
            100,
        )
        .await?;
    }
    let database = Database::open(fixture.path.to_str().ok_or("non-UTF-8 database path")?).await?;
    for (account, region, expected, foreign) in [
        (ACCOUNT, REGION, "own-region", "foreign-region"),
        (FOREIGN, REGION, "foreign-region", "own-region"),
        (ACCOUNT, "global", "own-global", "foreign-global"),
        (FOREIGN, "global", "foreign-global", "own-global"),
    ] {
        // Given identical region/kind/id in different accounts, both the
        // predicate and decoded pairs must refer to the selected account.
        let selected = database
            .page(&filter(account, region, "owner", Some(expected)), page(1))
            .await?;
        assert_eq!(selected.total_matching, 1);
        assert_eq!(selected.next, None);
        let resource = selected.resources.first().ok_or("identity row missing")?;
        assert_eq!(selected.resources.len(), 1);
        assert_metadata(resource, &observation(account, region, "shared", expected)?);
        assert_eq!(
            resource.tags,
            vec![ResourceTag {
                key: "owner".into(),
                value: expected.into()
            }]
        );
        assert_eq!(
            resource.tag_observation,
            status(TagCoverage::Complete, 100, Some(100))
        );
        assert_eq!(
            database.detail(&node(account, region, "shared")).await?,
            Some(resource.clone())
        );
        let excluded = database
            .page(&filter(account, region, "owner", Some(foreign)), page(1))
            .await?;
        assert_eq!(excluded.total_matching, 0);
        assert!(excluded.resources.is_empty());
        assert_eq!(excluded.next, None);
    }
    // Then the key-only traversal preserves full identity across both account
    // and global/regional boundaries, rather than deduplicating the shared ID.
    let all = ResourceFilter {
        tag: Some(cmdb_view::TagFilter {
            key: "owner".into(),
            value: None,
        }),
        ..ResourceFilter::default()
    };
    let mut request = page(1);
    for (index, (account, region)) in [
        (ACCOUNT, "global"),
        (ACCOUNT, REGION),
        (FOREIGN, "global"),
        (FOREIGN, REGION),
    ]
    .into_iter()
    .enumerate()
    {
        let result = database.page(&all, request.clone()).await?;
        assert_eq!(result.total_matching, 4);
        assert_eq!(result.resources.len(), 1);
        assert_eq!(
            result.resources.first().map(|resource| &resource.key),
            Some(&key(account, region, "shared")?)
        );
        let cursor = PageCursor {
            account: account.into(),
            region: region.into(),
            kind: KIND.into(),
            id: "shared".into(),
        };
        assert_eq!(result.next, (index < 3).then_some(cursor));
        request.after = result.next;
    }
    Ok(())
}

#[tokio::test]
async fn given_complete_then_every_degraded_and_absent_observation_when_reading_retained_and_reopened_database_then_history_is_exact()
-> TestResult {
    let fixture = fixture().await?;
    let pairs = [("empty", ""), ("owner", "alice")];
    commit(
        &fixture.store,
        "complete",
        ACCOUNT,
        REGION,
        vec![observation(ACCOUNT, REGION, "shared", "original")?],
        vec![tags(
            ACCOUNT,
            REGION,
            "shared",
            TagCoverage::Complete,
            &pairs,
        )?],
        100,
    )
    .await?;
    let database = Database::open(fixture.path.to_str().ok_or("non-UTF-8 database path")?).await?;
    let selected = node(ACCOUNT, REGION, "shared");
    let literal = filter(ACCOUNT, REGION, "empty", Some(""));
    let mut retained = database.snapshot().await?;
    let original = retained
        .detail(&selected)
        .await?
        .ok_or("baseline detail missing")?;
    assert_eq!(
        original.tag_observation,
        status(TagCoverage::Complete, 100, Some(100))
    );
    assert_eq!(
        original.tags,
        vec![
            ResourceTag {
                key: "empty".into(),
                value: String::new()
            },
            ResourceTag {
                key: "owner".into(),
                value: "alice".into()
            }
        ]
    );
    for (sweep, coverage, at) in [
        ("partial", TagCoverage::Partial, 200),
        ("failed", TagCoverage::Failed, 300),
        ("unobserved", TagCoverage::Unobserved, 400),
    ] {
        commit(
            &fixture.store,
            sweep,
            ACCOUNT,
            REGION,
            vec![observation(ACCOUNT, REGION, "shared", sweep)?],
            vec![tags(
                ACCOUNT,
                REGION,
                "shared",
                coverage,
                &[("ignored", "degraded")],
            )?],
            at,
        )
        .await?;
        let actual = database
            .detail(&selected)
            .await?
            .ok_or("degraded detail missing")?;
        assert_metadata(&actual, &observation(ACCOUNT, REGION, "shared", sweep)?);
        assert_eq!(actual.tag_observation, status(coverage, at, Some(100)));
        assert_eq!(actual.tags, original.tags);
        let current = database.page(&literal, page(1)).await?;
        assert_eq!(current.total_matching, 1);
        assert_eq!(current.resources, vec![actual]);
        assert_eq!(retained.detail(&selected).await?, Some(original.clone()));
        assert_eq!(
            retained.page(&literal, page(1)).await?.resources,
            vec![original.clone()]
        );
    }
    // Absence at 500 changes metadata, not the latest tag evidence at 400.
    commit(
        &fixture.store,
        "absent",
        ACCOUNT,
        REGION,
        vec![observation(ACCOUNT, REGION, "shared", "metadata-only")?],
        vec![],
        500,
    )
    .await?;
    let final_row = database
        .detail(&selected)
        .await?
        .ok_or("metadata-only detail missing")?;
    assert_metadata(
        &final_row,
        &observation(ACCOUNT, REGION, "shared", "metadata-only")?,
    );
    assert_eq!(final_row.tags, original.tags);
    assert_eq!(
        final_row.tag_observation,
        status(TagCoverage::Unobserved, 400, Some(100))
    );
    assert_eq!(retained.detail(&selected).await?, Some(original));
    retained.finish().await?;
    drop(database);
    drop(fixture.store);
    let _writer = BinaryStore::open(&fixture.path).await?;
    let reopened = Database::open(fixture.path.to_str().ok_or("non-UTF-8 database path")?).await?;
    assert_eq!(reopened.detail(&selected).await?, Some(final_row.clone()));
    assert_eq!(
        reopened.page(&literal, page(1)).await?.resources,
        vec![final_row]
    );
    Ok(())
}

#[tokio::test]
async fn given_real_schema14_legacy_pairs_when_writer_migrates_then_reader_distinguishes_unknown_from_complete_empty_across_reopen()
-> TestResult {
    let Fixture {
        directory: _directory,
        path,
        store,
    } = fixture().await?;
    commit(
        &store,
        "legacy",
        ACCOUNT,
        REGION,
        vec![observation(ACCOUNT, REGION, "shared", "legacy-metadata")?],
        vec![tags(
            ACCOUNT,
            REGION,
            "shared",
            TagCoverage::Complete,
            &[("legacy", "retained")],
        )?],
        100,
    )
    .await?;
    drop(store);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(&path))
        .await?;
    sqlx::query("DROP TABLE tag_observations")
        .execute(&pool)
        .await?;
    sqlx::query("DROP TABLE resource_facets")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE work_queue DROP COLUMN context")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE unresolved_refs DROP COLUMN target_kinds")
        .execute(&pool)
        .await?;
    sqlx::query("ALTER TABLE unresolved_refs DROP COLUMN resolution")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA user_version=14").execute(&pool).await?;
    pool.close().await;
    assert!(matches!(
        Database::open(path.to_str().ok_or("non-UTF-8 database path")?).await,
        Err(ViewError::Version(14))
    ));
    let store = BinaryStore::open(&path).await?;
    let database = Database::open(path.to_str().ok_or("non-UTF-8 database path")?).await?;
    let selected = node(ACCOUNT, REGION, "shared");
    let mut retained = database.snapshot().await?;
    let legacy = retained
        .detail(&selected)
        .await?
        .ok_or("migrated legacy row missing")?;
    assert_metadata(
        &legacy,
        &observation(ACCOUNT, REGION, "shared", "legacy-metadata")?,
    );
    assert_eq!(
        legacy.tags,
        vec![ResourceTag {
            key: "legacy".into(),
            value: "retained".into()
        }]
    );
    assert_eq!(legacy.tag_observation, None);
    assert_eq!(
        database
            .page(
                &filter(ACCOUNT, REGION, "legacy", Some("retained")),
                page(1)
            )
            .await?
            .resources,
        vec![legacy.clone()]
    );
    commit(
        &store,
        "complete-empty",
        ACCOUNT,
        REGION,
        vec![],
        vec![tags(ACCOUNT, REGION, "shared", TagCoverage::Complete, &[])?],
        200,
    )
    .await?;
    let empty = database
        .detail(&selected)
        .await?
        .ok_or("complete-empty row missing")?;
    assert_metadata(
        &empty,
        &observation(ACCOUNT, REGION, "shared", "legacy-metadata")?,
    );
    assert!(empty.tags.is_empty());
    assert_eq!(
        empty.tag_observation,
        status(TagCoverage::Complete, 200, Some(200))
    );
    let absent = database
        .page(&filter(ACCOUNT, REGION, "legacy", None), page(1))
        .await?;
    assert_eq!(absent.total_matching, 0);
    assert!(absent.resources.is_empty());
    assert_eq!(retained.detail(&selected).await?, Some(legacy));
    retained.finish().await?;
    drop(database);
    drop(store);
    let _writer = BinaryStore::open(&path).await?;
    let reopened = Database::open(path.to_str().ok_or("non-UTF-8 database path")?).await?;
    assert_eq!(reopened.detail(&selected).await?, Some(empty.clone()));
    assert_eq!(
        reopened
            .page(&ResourceFilter::default(), page(1))
            .await?
            .resources,
        vec![empty]
    );
    Ok(())
}

#[tokio::test]
async fn given_over_one_thousand_literal_tag_matches_when_paging_at_both_limits_then_order_coverage_and_rejections_are_exact()
-> TestResult {
    let fixture = fixture().await?;
    let literal_key = "%_' OR 1=1 --";
    let ids = (0..1003)
        .map(|index| format!("id-{index:04}"))
        .collect::<Vec<_>>();
    let observations = ids
        .iter()
        .map(|id| observation(ACCOUNT, REGION, id, id))
        .collect::<TestResult<Vec<_>>>()?;
    let pairs = ids
        .iter()
        .map(|id| {
            tags(
                ACCOUNT,
                REGION,
                id,
                TagCoverage::Complete,
                &[(literal_key, "")],
            )
        })
        .collect::<TestResult<Vec<_>>>()?;
    commit(
        &fixture.store,
        "large",
        ACCOUNT,
        REGION,
        observations,
        pairs,
        100,
    )
    .await?;
    commit(
        &fixture.store,
        "decoys",
        ACCOUNT,
        REGION,
        vec![
            observation(ACCOUNT, REGION, "decoy-value", "not-empty")?,
            observation(ACCOUNT, REGION, "decoy-absent", "no tag")?,
            observation(ACCOUNT, REGION, "decoy-pattern", "different literal key")?,
        ],
        vec![
            tags(
                ACCOUNT,
                REGION,
                "decoy-value",
                TagCoverage::Complete,
                &[(literal_key, "not-empty")],
            )?,
            tags(
                ACCOUNT,
                REGION,
                "decoy-pattern",
                TagCoverage::Complete,
                &[("abcX' OR 1=1 --", "")],
            )?,
        ],
        200,
    )
    .await?;
    let database = Database::open(fixture.path.to_str().ok_or("non-UTF-8 database path")?).await?;
    let literal = filter(ACCOUNT, REGION, literal_key, Some(""));
    assert_eq!(database.counts().await?.resources, 1006);
    for limit in [1, 1000] {
        let mut request = page(limit);
        let mut actual = Vec::new();
        loop {
            let result = database.page(&literal, request.clone()).await?;
            assert_eq!(result.total_matching, 1003);
            assert!(!result.resources.is_empty());
            assert!(result.resources.len() <= limit);
            for row in &result.resources {
                let id = row.key.id.as_str();
                assert_metadata(row, &observation(ACCOUNT, REGION, id, id)?);
                assert_eq!(
                    row.tags,
                    vec![ResourceTag {
                        key: literal_key.into(),
                        value: String::new()
                    }]
                );
                assert_eq!(
                    row.tag_observation,
                    status(TagCoverage::Complete, 100, Some(100))
                );
                actual.push(id.to_owned());
            }
            assert!(
                actual.len() <= ids.len(),
                "duplicate or nonterminating cursor"
            );
            let expected_next = if actual.len() < ids.len() {
                Some(PageCursor {
                    account: ACCOUNT.into(),
                    region: REGION.into(),
                    kind: KIND.into(),
                    id: actual.last().ok_or("last page id missing")?.clone(),
                })
            } else {
                None
            };
            assert_eq!(result.next, expected_next);
            match result.next {
                Some(cursor) => request.after = Some(cursor),
                None => break,
            }
        }
        assert_eq!(actual, ids);
    }
    // A LIKE mutation must fail in key-only mode too; the same-key nonempty
    // decoy is included, while the pattern-matching different key is excluded.
    let key_only = filter(ACCOUNT, REGION, literal_key, None);
    let first = database.page(&key_only, page(1000)).await?;
    assert_eq!(first.total_matching, 1004);
    assert_eq!(first.resources.len(), 1000);
    let next = first.next.ok_or("key-only traversal cursor missing")?;
    let second = database
        .page(
            &key_only,
            cmdb_view::PageRequest {
                limit: 1000,
                after: Some(next),
            },
        )
        .await?;
    assert_eq!(second.total_matching, 1004);
    assert_eq!(second.resources.len(), 4);
    assert_eq!(second.next, None);
    let actual = first
        .resources
        .iter()
        .chain(&second.resources)
        .map(|row| row.key.id.as_str())
        .collect::<Vec<_>>();
    let expected = std::iter::once("decoy-value")
        .chain(ids.iter().map(String::as_str))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
    assert!(first.resources.iter().chain(&second.resources).all(
        |row| row.tags.len() == 1 && row.tags.first().is_some_and(|tag| tag.key == literal_key)
    ));
    let mut snapshot = database.snapshot().await?;
    for invalid in [0, 1001] {
        let error = match database.page(&literal, page(invalid)).await {
            Err(error) => error,
            Ok(page) => return Err(format!("invalid limit {invalid} returned {page:?}").into()),
        };
        assert!(matches!(error, ViewError::PageLimit));
        assert_eq!(error.to_string(), "page limit must be in 1..=1000");
        let error = match snapshot.page(&literal, page(invalid)).await {
            Err(error) => error,
            Ok(page) => return Err(format!("snapshot limit {invalid} returned {page:?}").into()),
        };
        assert!(matches!(error, ViewError::PageLimit));
        assert_eq!(error.to_string(), "page limit must be in 1..=1000");
    }
    snapshot.finish().await?;
    Ok(())
}
