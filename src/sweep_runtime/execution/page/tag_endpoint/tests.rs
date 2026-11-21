mod branches;
use crate::sweep_runtime::consumer_fixture as fixture;
mod presence;
mod sources;
mod support;
mod trust;
use super::*;
use catalog_tool as fixture_catalog;
use fixture::TestResult;
use inventory_core::inventory::{Region, ScopeOutcome, TagCoverage};
use inventory_engine::inventory::BinaryStore;
use support::*;

#[tokio::test]
async fn given_paged_role_tags_when_process_reopens_then_pending_pairs_finish_as_one_owner_snapshot()
-> TestResult {
    let compiled = iam()?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("iam", "ListRoleTags")
        .ok_or("tag plan missing")?;
    let (directory, store, owner, sweep) = seeded(&compiled, "tag-restart").await?;
    let lease = claim(&store, &sweep, 2).await?;
    assert_eq!(lease.scope().parent(), Some(&owner));
    let input = Record::from_bytes(lease.bindings().to_vec())?;
    let role_name = compiled.fields.id("RoleName").ok_or("RoleName missing")?;
    assert!(matches!(
        input.root().get_field(role_name)?,
        Some(ValueRef::Text("owner"))
    ));
    // Given: the first decoded AWS page is partial and cannot remove an existing pair.
    let parts = consume(
        &catalog,
        &lease,
        xml(plan, Some(&[("alpha", "one")]), Some("resume-1"))?,
        10,
    )?;
    assert_eq!(pair(&parts.tags)?.coverage(), TagCoverage::Partial);
    assert_eq!(pair(&parts.tags)?.resource(), &owner);
    commit(&store, lease, parts, 3).await?;
    assert_eq!(
        tags(&store, &owner).await?,
        vec![("known".into(), "old".into())]
    );
    drop(store);
    // When: a new store handle resumes only persisted work/cursor/accumulation bytes.
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_eq!(store.discover_active_sweep().await?.as_ref(), Some(&sweep));
    let lease = claim(&store, &sweep, 4).await?;
    let cursor = decode_cursor(lease.token())?;
    assert_eq!(cursor.pages, 1);
    let parts = consume(
        &catalog,
        &lease,
        xml(plan, Some(&[("omega", "")]), None)?,
        10,
    )?;
    assert_eq!(pair(&parts.tags)?.coverage(), TagCoverage::Complete);
    assert_eq!(pair(&parts.tags)?.tags().len(), 2);
    commit(&store, lease, parts, 5).await?;
    // Then: the old pair is removed only after both pages become the complete snapshot.
    assert_eq!(
        tags(&store, &owner).await?,
        vec![("alpha".into(), "one".into()), ("omega".into(), "".into())]
    );
    assert!(store.claim(&sweep, 6, 1000).await?.is_none());
    assert_eq!(
        store.projection_resources(None, 100).await?.len(),
        1,
        "tag endpoint must not invent a TagResource"
    );
    drop(store);
    directory.close()?;
    Ok(())
}

#[tokio::test]
async fn given_missing_or_explicit_empty_tags_when_terminal_then_only_explicit_empty_removes_known_pairs()
-> TestResult {
    let compiled = iam()?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("iam", "ListRoleTags")
        .ok_or("tag plan missing")?;
    for (sweep_id, page, expected) in [
        ("tag-missing", None, TagCoverage::Unobserved),
        ("tag-empty", Some(&[][..]), TagCoverage::Complete),
    ] {
        let (directory, store, owner, sweep) = seeded(&compiled, sweep_id).await?;
        let lease = claim(&store, &sweep, 2).await?;
        let parts = consume(&catalog, &lease, xml(plan, page, None)?, 10)?;
        assert_eq!(pair(&parts.tags)?.coverage(), expected);
        commit(&store, lease, parts, 3).await?;
        let observed = tags(&store, &owner).await?;
        if expected == TagCoverage::Unobserved {
            assert_eq!(observed, vec![("known".into(), "old".into())]);
        } else {
            assert!(observed.is_empty());
        }
        drop(store);
        directory.close()?;
    }
    Ok(())
}

#[tokio::test]
async fn given_partial_tag_pages_when_denied_repeated_or_capped_then_known_pairs_survive()
-> TestResult {
    let compiled = iam()?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("iam", "ListRoleTags")
        .ok_or("tag plan missing")?;
    for failure in ["denied", "repeat", "page-cap"] {
        let (directory, store, owner, sweep) = seeded(&compiled, &format!("tag-{failure}")).await?;
        let lease = claim(&store, &sweep, 2).await?;
        let parts = consume(
            &catalog,
            &lease,
            xml(plan, Some(&[("visible", "pair")]), Some("same"))?,
            10,
        )?;
        commit(&store, lease, parts, 3).await?;
        let lease = claim(&store, &sweep, 4).await?;
        if failure == "denied" {
            store
                .finish_scope(lease, ScopeOutcome::Inaccessible, 5)
                .await?;
        } else {
            let cap = if failure == "page-cap" { 2 } else { 10 };
            let token = if failure == "page-cap" {
                "different"
            } else {
                "same"
            };
            let parts = consume(
                &catalog,
                &lease,
                xml(plan, Some(&[("new", "pair")]), Some(token))?,
                cap,
            )?;
            assert!(parts.next.is_some());
            assert_eq!(pair(&parts.tags)?.coverage(), TagCoverage::Partial);
            if failure == "page-cap" {
                commit(&store, lease, parts, 5).await?;
                let lease = claim(&store, &sweep, 6).await?;
                assert!(decode_cursor(lease.token())?.partial);
                store.finish_scope(lease, ScopeOutcome::Partial, 7).await?;
            } else {
                // Repetition is rejected by durable token-digest admission, not a fabricated completion.
                let result = commit(&store, lease, parts, 5).await?;
                assert!(result.repeated_token);
            }
        }
        assert_eq!(
            tags(&store, &owner).await?,
            vec![("known".into(), "old".into())],
            "{failure}"
        );
        drop(store);
        directory.close()?;
    }
    Ok(())
}
