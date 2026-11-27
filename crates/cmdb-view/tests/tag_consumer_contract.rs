#[path = "support/tag_writer.rs"]
mod support;

use cmdb_view::{
    Database, PageCursor, ResourceFilter, ResourceTag, StoredResource, StoredTagObservation,
    ViewError,
};
use inventory_core::inventory::{Observation, TagCoverage};
use inventory_engine::inventory::{DatabaseUrl, PgStore, PgWriteConfig, StoreError};
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
    bounded(async {
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
        let database = Database::open(fixture.url()).await?;
        for (account, region, expected, foreign) in [
            (ACCOUNT, REGION, "own-region", "foreign-region"),
            (FOREIGN, REGION, "foreign-region", "own-region"),
            (ACCOUNT, "global", "own-global", "foreign-global"),
            (FOREIGN, "global", "foreign-global", "own-global"),
        ] {
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
        drop(database);
        fixture.finish().await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn given_complete_then_every_degraded_and_absent_observation_when_reading_retained_and_reopened_database_then_history_is_exact()
-> TestResult {
    bounded(async {
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
        let database = Database::open(fixture.url()).await?;
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
        let fixture = fixture.reopen().await?;
        let reopened = Database::open(fixture.url()).await?;
        assert_eq!(reopened.detail(&selected).await?, Some(final_row.clone()));
        assert_eq!(
            reopened.page(&literal, page(1)).await?.resources,
            vec![final_row]
        );
        drop(reopened);
        fixture.finish().await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn given_legacy_metadata_without_tag_status_when_writer_records_complete_empty_then_reader_distinguishes_unknown_from_complete_empty_across_reopen()
-> TestResult {
    bounded(async {
        let fixture = fixture().await?;
        commit(
            &fixture.store,
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
        sqlx::query("DELETE FROM inventory.tag_observations")
            .execute(&fixture.pool)
            .await?;
        let database = Database::open(fixture.url()).await?;
        let selected = node(ACCOUNT, REGION, "shared");
        let mut retained = database.snapshot().await?;
        let legacy = retained
            .detail(&selected)
            .await?
            .ok_or("legacy row missing")?;
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
            &fixture.store,
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
        let fixture = fixture.reopen().await?;
        let reopened = Database::open(fixture.url()).await?;
        assert_eq!(reopened.detail(&selected).await?, Some(empty.clone()));
        assert_eq!(
            reopened
                .page(&ResourceFilter::default(), page(1))
                .await?
                .resources,
            vec![empty]
        );
        drop(reopened);
        fixture.finish().await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn given_schema_version_other_than_fourteen_when_reader_or_writer_opens_then_both_reject_with_typed_errors_and_recover_at_fourteen()
-> TestResult {
    bounded(async {
        let fixture = fixture().await?;
        commit(
            &fixture.store,
            "versioned",
            ACCOUNT,
            REGION,
            vec![observation(ACCOUNT, REGION, "shared", "versioned")?],
            vec![tags(
                ACCOUNT,
                REGION,
                "shared",
                TagCoverage::Complete,
                &[("owner", "alice")],
            )?],
            100,
        )
        .await?;
        for found in [13_i64, 15] {
            fixture
                .set_schema_version(found.to_string().as_bytes())
                .await?;
            assert!(matches!(
                Database::open(fixture.url()).await,
                Err(ViewError::PostgresVersion)
            ));
            assert!(matches!(
                PgStore::open(&DatabaseUrl::parse(fixture.url())?, PgWriteConfig::default()).await,
                Err(StoreError::SchemaVersion { found: rejected }) if rejected == found
            ));
        }
        fixture.set_schema_version(b"14").await?;
        let database = Database::open(fixture.url()).await?;
        let row = database
            .detail(&node(ACCOUNT, REGION, "shared"))
            .await?
            .ok_or("versioned row missing after restore")?;
        assert_metadata(&row, &observation(ACCOUNT, REGION, "shared", "versioned")?);
        assert_eq!(
            row.tags,
            vec![ResourceTag {
                key: "owner".into(),
                value: "alice".into()
            }]
        );
        assert_eq!(
            row.tag_observation,
            status(TagCoverage::Complete, 100, Some(100))
        );
        drop(database);
        fixture.finish().await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn given_over_one_thousand_literal_tag_matches_when_paging_at_both_limits_then_order_coverage_and_rejections_are_exact()
-> TestResult {
    bounded(async {
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
        let database = Database::open(fixture.url()).await?;
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
        let key_only = filter(ACCOUNT, REGION, literal_key, None);
        let first = database.page(&key_only, page(1000)).await?;
        assert_eq!(first.total_matching, 1004);
        assert_eq!(first.resources.len(), 1000);
        let next = first.next.clone().ok_or("key-only traversal cursor missing")?;
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
        drop(database);
        fixture.finish().await?;
        Ok(())
    })
    .await
}
