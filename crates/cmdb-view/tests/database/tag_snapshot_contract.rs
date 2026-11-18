use super::*;
use cmdb_view::{ResourceTag, StoredTagObservation, TagFilter};
use inventory_core::inventory::{Tag, TagCoverage, TagObservation};

async fn commit_tags(
    store: &BinaryStore,
    sweep: &str,
    coverage: TagCoverage,
    tags: Vec<Tag>,
    at: i64,
) -> TestResult {
    let scope = ScopeKey::new(
        AccountId::parse(ACCOUNT)?,
        Region::parse(REGION)?,
        ServiceId::parse("example")?,
        OperationId::parse("ListThings")?,
        None,
    )?;
    let sweep = SweepId::new(sweep.into())?;
    store
        .seed_work(WorkSpec::new(sweep.clone(), scope, None, vec![1])?)
        .await?;
    let lease = store
        .claim(&sweep, at, 100)
        .await?
        .ok_or("missing tag lease")?;
    let resource = key(ACCOUNT, REGION, "root")?;
    let observation = Observation::new(
        resource.clone(),
        record("unchanged")?,
        &Exclusions::none(),
        Some(Arc::from("unchanged")),
    )?;
    store
        .commit_page(PageCommit {
            lease,
            observations: vec![observation],
            edges: vec![],
            unresolved: vec![],
            tags: vec![TagObservation::new(resource, coverage, tags)?],
            next_token: None,
            continuation_digest: None,
            children: vec![],
            done: true,
            observed_at_ms: at,
        })
        .await?;
    Ok(())
}

fn tagged(value: &str) -> ResourceFilter {
    ResourceFilter {
        tag: Some(TagFilter {
            key: "revision".into(),
            value: Some(value.into()),
        }),
        ..filter(ACCOUNT, REGION)
    }
}

fn expected(value: &str) -> Vec<ResourceTag> {
    vec![ResourceTag {
        key: "revision".into(),
        value: value.into(),
    }]
}

#[tokio::test]
async fn given_real_tag_only_commits_when_retained_reader_queries_then_opening_tags_survive_replacement_and_removal()
-> TestResult {
    tokio::time::timeout(Duration::from_secs(15), async {
        // Given: a production writer snapshot and a reader pinned before tag reads.
        let (_directory, store, db) = fixture().await?;
        commit_tags(
            &store,
            "tag-initial",
            TagCoverage::Complete,
            vec![Tag::new("revision", "old")?],
            100,
        )
        .await?;
        let mut retained = db.snapshot().await?;
        // When: unchanged resource content gets new tags and then complete-empty tags.
        commit_tags(
            &store,
            "tag-replace",
            TagCoverage::Complete,
            vec![Tag::new("revision", "new")?],
            200,
        )
        .await?;
        let new = db
            .detail(&node(ACCOUNT, REGION, "root"))
            .await?
            .ok_or("fresh root missing")?;
        commit_tags(&store, "tag-remove", TagCoverage::Complete, vec![], 300).await?;
        let removed = db
            .detail(&node(ACCOUNT, REGION, "root"))
            .await?
            .ok_or("untagged root missing")?;
        let old = retained
            .detail(&node(ACCOUNT, REGION, "root"))
            .await?
            .ok_or("retained root missing")?;
        // Then: old transaction has old tags/filter result, fresh read sees removal.
        assert_eq!(old.tags, expected("old"));
        assert_eq!(
            old.tag_observation,
            Some(StoredTagObservation {
                coverage: TagCoverage::Complete,
                observed_at_ms: 100,
                last_complete_at_ms: Some(100),
            })
        );
        assert_eq!(new.tags, expected("new"));
        assert_eq!(
            new.tag_observation,
            Some(StoredTagObservation {
                coverage: TagCoverage::Complete,
                observed_at_ms: 200,
                last_complete_at_ms: Some(200),
            })
        );
        assert!(removed.tags.is_empty());
        assert!(!removed.tags_truncated);
        assert_eq!(
            removed.tag_observation,
            Some(StoredTagObservation {
                coverage: TagCoverage::Complete,
                observed_at_ms: 300,
                last_complete_at_ms: Some(300),
            })
        );
        assert_eq!(old.content_hash, new.content_hash);
        assert_eq!(old.content_hash, removed.content_hash);
        assert_eq!(
            retained.page(&tagged("old"), page(10)).await?.resources,
            vec![old]
        );
        assert_eq!(
            retained
                .page(&tagged("new"), page(10))
                .await?
                .total_matching,
            0
        );
        assert_eq!(db.page(&tagged("old"), page(10)).await?.total_matching, 0);
        assert_eq!(db.page(&tagged("new"), page(10)).await?.total_matching, 0);
        retained.finish().await?;
        Ok(())
    })
    .await?
}

#[tokio::test]
async fn given_real_known_tags_when_failed_or_partial_snapshots_arrive_then_public_reader_keeps_known_membership()
-> TestResult {
    tokio::time::timeout(Duration::from_secs(15), async {
        // Given: a complete production tag observation.
        let (_directory, store, db) = fixture().await?;
        commit_tags(
            &store,
            "known",
            TagCoverage::Complete,
            vec![Tag::new("revision", "known")?],
            100,
        )
        .await?;
        // When: partial, failed, and unobserved extraction follows.
        commit_tags(
            &store,
            "partial",
            TagCoverage::Partial,
            vec![Tag::new("revision", "partial")?],
            200,
        )
        .await?;
        assert_eq!(db.page(&tagged("known"), page(10)).await?.total_matching, 1);
        assert_eq!(
            db.detail(&node(ACCOUNT, REGION, "root"))
                .await?
                .and_then(|row| row.tag_observation),
            Some(StoredTagObservation {
                coverage: TagCoverage::Partial,
                observed_at_ms: 200,
                last_complete_at_ms: Some(100)
            })
        );
        commit_tags(&store, "failed", TagCoverage::Failed, vec![], 300).await?;
        assert_eq!(
            db.detail(&node(ACCOUNT, REGION, "root"))
                .await?
                .and_then(|row| row.tag_observation),
            Some(StoredTagObservation {
                coverage: TagCoverage::Failed,
                observed_at_ms: 300,
                last_complete_at_ms: Some(100)
            })
        );
        commit_tags(&store, "unobserved", TagCoverage::Unobserved, vec![], 400).await?;
        // Then: consumer details/filters expose only the last complete observation.
        let found = db.page(&tagged("known"), page(10)).await?;
        assert_eq!(found.total_matching, 1);
        assert_eq!(
            found.resources.first().map(|resource| &resource.tags),
            Some(&expected("known"))
        );
        assert_eq!(
            found
                .resources
                .first()
                .and_then(|resource| resource.tag_observation),
            Some(StoredTagObservation {
                coverage: TagCoverage::Unobserved,
                observed_at_ms: 400,
                last_complete_at_ms: Some(100)
            })
        );
        assert_eq!(
            db.page(&tagged("partial"), page(10)).await?.total_matching,
            0
        );
        assert_eq!(db.counts().await?.resources, 1);
        Ok(())
    })
    .await?
}
