use super::*;
use fixtures::key;
use inventory_core::inventory::{Tag, TagCoverage, TagObservation};

fn observed(node: &GraphNode, key_name: &str, value: &str) -> TestResult<TagObservation> {
    Ok(TagObservation::new(
        key(node)?,
        TagCoverage::Complete,
        vec![Tag::new(key_name, value)?],
    )?)
}

#[test]
fn given_committed_scoped_tags_when_real_worker_filters_then_exact_empty_value_and_identity_are_preserved()
-> TestResult {
    supervised_async(|| async {
        // Given identical resource IDs with distinct account and region tag values.
        let fixture = Fixture::new().await?;
        let tag_key = "owner' OR 1=1 --";
        let expected = fixture
            .commit_tagged(
                "east",
                (ACCOUNT, REGION),
                &[("shared", "east")],
                &[],
                vec![observed(&root(), tag_key, "")?],
                1,
            )
            .await?;
        let west = node(ACCOUNT, "us-west-2", KIND, "shared");
        fixture
            .commit_tagged(
                "west",
                (ACCOUNT, "us-west-2"),
                &[("shared", "west")],
                &[],
                vec![observed(&west, tag_key, "different")?],
                1,
            )
            .await?;
        let foreign = node("222222222222", REGION, KIND, "shared");
        fixture
            .commit_tagged(
                "foreign",
                ("222222222222", REGION),
                &[("shared", "foreign")],
                &[],
                vec![observed(&foreign, "other", "")?],
                1,
            )
            .await?;
        let worker = fixture.worker(None);
        let (counts, initial, total) = snapshot(&worker).await?;
        assert_eq!((counts.resources, initial.len(), total), (3, 3, 3));
        // When an exact empty tag value passes through the public worker interface.
        worker.refresh(Filter {
            tag: Some(cmdb_view::TagFilter {
                key: tag_key.into(),
                value: Some(String::new()),
            }),
            ..Filter::default()
        })?;
        let (counts, rows, total) = snapshot(&worker).await?;
        // Then neither hostile SQL-looking keys nor same-id scope collisions broaden it.
        assert_eq!(rows, expected);
        assert_eq!(total, 1);
        assert_eq!(counts.resources, 3);
        assert_eq!(
            rows.first().map(|row| row.tags.as_slice()),
            Some(
                [cmdb_view::ResourceTag {
                    key: tag_key.into(),
                    value: String::new()
                }]
                .as_slice()
            )
        );
        worker.refresh(Filter {
            tag: Some(cmdb_view::TagFilter {
                key: tag_key.into(),
                value: None,
            }),
            account: Some(ACCOUNT.into()),
            region: Some("us-west-2".into()),
            ..Filter::default()
        })?;
        let (_, rows, total) = snapshot(&worker).await?;
        assert_eq!(total, 1);
        assert_eq!(
            rows.first().map(|row| row.region.as_str()),
            Some("us-west-2")
        );
        assert_eq!(
            rows.first()
                .and_then(|row| row.tags.first())
                .map(|tag| tag.value.as_str()),
            Some("different")
        );
        Ok(())
    })
}

#[test]
fn given_live_native_tags_and_committed_sql_tags_when_expanding_then_affiliations_keep_root_scope_and_authority()
-> TestResult {
    supervised_async(|| async {
        if !falkor_available().await? {
            return Ok(());
        }
        // Given real SQLite tags and native topology with an independently current tag.
        let fixture = Fixture::new().await?;
        let a = root();
        let b = node(ACCOUNT, REGION, KIND, "b");
        let link = edge(&a, &b, "references", "model");
        fixture
            .commit_tagged(
                "before",
                (ACCOUNT, REGION),
                &[("shared", "root"), ("b", "neighbor")],
                &[],
                vec![observed(&a, "owner", "before")?],
                1,
            )
            .await?;
        let native = NativeGraph::new(std::slice::from_ref(&link)).await?;
        let worker = fixture.worker(Some(&native));
        let result: TestResult = async {
            snapshot(&worker).await?;
            let after = fixture
                .commit_tagged(
                    "after",
                    (ACCOUNT, REGION),
                    &[("shared", "root"), ("b", "neighbor")],
                    &[],
                    vec![observed(&a, "owner", "after")?],
                    2,
                )
                .await?;
            native
                .query(&inventory_graph::cypher::diagnostics::foreign_tag_fixture(
                    &digest(&a)?,
                    ACCOUNT,
                    REGION,
                )?)
                .await?;
            // When the actual worker reads live native tags and pinned SQL details.
            worker.expand(a.clone())?;
            let expanded = graph(&worker).await?;
            // Then tag membership is scoped to the requested root, never a dependency.
            assert_edges(&expanded, std::slice::from_ref(&link))?;
            assert_eq!(expanded.tag_root, Some(digest(&a)?));
            assert_eq!(
                expanded.tag_affiliations,
                vec![cmdb_view::TagAffiliation {
                    tag: cmdb_view::ScopedTag {
                        account: ACCOUNT.into(),
                        region: REGION.into(),
                        key: "owner".into(),
                        value: "after".into()
                    },
                    provenance: "inventory.tags".into(),
                }]
            );
            assert!(!expanded.tags_truncated);
            assert_eq!(
                expanded.nodes.iter().find(|row| row.id == "shared"),
                after.iter().find(|row| row.id == "shared")
            );
            worker.expand(b)?;
            let untagged = graph(&worker).await?;
            assert!(untagged.tag_affiliations.is_empty());
            assert!(!untagged.tags_truncated);
            assert_eq!(
                untagged.tag_root,
                Some(digest(&node(ACCOUNT, REGION, KIND, "b"))?)
            );
            Ok(())
        }
        .await;
        drop(worker);
        let cleanup = native.delete().await;
        result?;
        cleanup
    })
}

#[test]
fn given_complete_tags_when_later_observation_fails_then_real_worker_exposes_retained_values_and_coverage()
-> TestResult {
    supervised_async(|| async {
        // Given a complete writer observation whose empty value is meaningful.
        let fixture = Fixture::new().await?;
        fixture
            .commit_tagged(
                "complete",
                (ACCOUNT, REGION),
                &[("shared", "root")],
                &[],
                vec![observed(&root(), "owner", "")?],
                1,
            )
            .await?;
        let worker = fixture.worker(None);
        let (_, before, _) = snapshot(&worker).await?;
        let complete = before.first().ok_or("initial resource missing")?;
        assert_eq!(
            complete.tag_observation,
            Some(cmdb_view::StoredTagObservation {
                coverage: TagCoverage::Complete,
                observed_at_ms: 1,
                last_complete_at_ms: Some(1),
            })
        );
        // When a failed observation publishes coverage but cannot erase known tags.
        fixture
            .commit_tagged(
                "failed",
                (ACCOUNT, REGION),
                &[("shared", "root")],
                &[],
                vec![TagObservation::new(
                    key(&root())?,
                    TagCoverage::Failed,
                    vec![],
                )?],
                2,
            )
            .await?;
        worker.refresh(Filter::default())?;
        let (_, after, total) = snapshot(&worker).await?;
        let retained = after.first().ok_or("retained resource missing")?;
        // Then the consumer sees exact retained values AND a non-complete latest state.
        assert_eq!(total, 1);
        assert_eq!(retained.tags, complete.tags);
        assert_eq!(
            retained.tag_observation,
            Some(cmdb_view::StoredTagObservation {
                coverage: TagCoverage::Failed,
                observed_at_ms: 2,
                last_complete_at_ms: Some(1),
            })
        );
        assert!(!retained.tags_truncated);
        Ok(())
    })
}
