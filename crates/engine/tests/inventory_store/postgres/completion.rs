use super::{TestResult, claim, collect, run, single, support, work};
use inventory_core::inventory::{ResourceId, ScopeOutcome};
use inventory_engine::inventory::{DatabaseUrl, PgStore};
use sqlx::PgPool;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_two_complete_misses_when_partial_intervenes_then_only_complete_authority_deletes()
-> TestResult {
    run(two_misses).await
}

async fn two_misses(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    single(store, "initial", "item", "value").await?;
    store.seed_works(&[work("partial", None)?]).await?;
    store
        .finish_scope(
            claim(store, "partial", 100).await?,
            ScopeOutcome::Partial,
            101,
            None,
        )
        .await?;
    assert_eq!(misses(pool).await?, 0);
    let first = collect(store, "miss-one", Vec::new(), Vec::new()).await?;
    assert_eq!(first.stats.resources_deleted, 0);
    assert_eq!(misses(pool).await?, 1);
    let second = collect(store, "miss-two", Vec::new(), Vec::new()).await?;
    assert_eq!(second.stats.resources_deleted, 1);
    let counts: (i64, i64, i64) = sqlx::query_as("SELECT (SELECT count(*) FROM inventory.resources),(SELECT count(*) FROM inventory.thing_table),(SELECT count(*) FROM inventory.projection_outbox WHERE deleted)")
        .fetch_one(pool).await?;
    assert_eq!(counts, (0, 0, 1));
    Ok(())
}

async fn misses(pool: &PgPool) -> TestResult<i64> {
    Ok(
        sqlx::query_scalar(
            "SELECT missing_complete FROM inventory.resource_scopes WHERE id='item'",
        )
        .fetch_one(pool)
        .await?,
    )
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_three_level_parent_chain_when_root_retires_then_every_exclusive_descendant_is_deleted()
-> TestResult {
    run(chain).await
}

async fn chain(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    store.seed_works(&[work("tree", None)?]).await?;
    admit(store, "tree", "parent", true).await?;
    admit(store, "tree", "child", true).await?;
    admit(store, "tree", "grandchild", false).await?;
    let first = collect(store, "tree-miss-one", Vec::new(), Vec::new()).await?;
    assert_eq!(first.stats.resources_deleted, 0);
    let second = collect(store, "tree-miss-two", Vec::new(), Vec::new()).await?;
    assert_eq!(second.stats.resources_deleted, 3);
    let counts: (i64, i64, i64) = sqlx::query_as("SELECT (SELECT count(*) FROM inventory.resources),(SELECT count(*) FROM inventory.resource_parents),(SELECT count(*) FROM inventory.thing_table)")
        .fetch_one(pool).await?;
    assert_eq!(counts, (0, 0, 0));
    Ok(())
}

async fn admit(store: &PgStore, sweep: &str, id: &str, child: bool) -> TestResult {
    let lease = claim(store, sweep, 100).await?;
    let mut page = support::page(lease, vec![support::observation(id, id)?], true, None, 101);
    if child {
        page.children
            .push(work(sweep, Some(support::resource(id)?))?);
    }
    assert_eq!(store.commit_page(page).await?.next_generation, None);
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_shared_child_when_one_parent_retires_then_other_owner_and_facet_survive()
-> TestResult {
    run(shared_child).await
}

async fn shared_child(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    seed_shared(store).await?;
    for _ in 0..2 {
        child_facet(store).await?;
    }
    for sweep in ["shared-miss-one", "shared-miss-two"] {
        single(store, sweep, "parent-two", "parent-two").await?;
    }
    let value: String =
        sqlx::query_scalar("SELECT f_1 FROM inventory.thing_table WHERE id='child'")
            .fetch_one(pool)
            .await?;
    assert_eq!(value, "from-two");
    let counts: (i64, i64, i64) = sqlx::query_as("SELECT (SELECT count(*) FROM inventory.resources),(SELECT count(*) FROM inventory.resource_facets WHERE id='child'),(SELECT count(*) FROM inventory.resource_parents WHERE child_id='child')")
        .fetch_one(pool).await?;
    assert_eq!(counts, (2, 1, 1));
    Ok(())
}

async fn child_facet(store: &PgStore) -> TestResult {
    let lease = claim(store, "shared", 100).await?;
    let parent = lease.scope().parent().ok_or("missing parent")?;
    let value = if parent.id.as_str() == "parent-one" {
        "from-one"
    } else {
        "from-two"
    };
    let page = support::page(
        lease,
        vec![support::observation("child", value)?],
        true,
        None,
        101,
    );
    store.commit_page(page).await?;
    Ok(())
}

async fn seed_shared(store: &PgStore) -> TestResult {
    store.seed_works(&[work("shared", None)?]).await?;
    let observations = ["parent-one", "parent-two"]
        .into_iter()
        .map(|id| support::observation(id, id))
        .collect::<TestResult<Vec<_>>>()?;
    let mut page = support::page(
        claim(store, "shared", 100).await?,
        observations,
        true,
        None,
        101,
    );
    page.children = ["parent-one", "parent-two"]
        .into_iter()
        .map(|id| work("shared", Some(support::resource(id)?)))
        .collect::<TestResult<Vec<_>>>()?;
    store.commit_page(page).await?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_failed_scope_when_last_work_finishes_then_no_negative_observation_is_authorized()
-> TestResult {
    run(failed).await
}

async fn failed(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    single(store, "failed-initial", "item", "value").await?;
    for sweep in ["failure-one", "failure-two"] {
        store.seed_works(&[work(sweep, None)?]).await?;
        let stats = store
            .finish_scope(
                claim(store, sweep, 100).await?,
                ScopeOutcome::Failed,
                101,
                None,
            )
            .await?;
        assert_eq!(stats.resources_deleted, 0);
    }
    assert_eq!(misses(pool).await?, 0);
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_scope_member_when_missing_two_sweeps_then_child_is_retired() -> TestResult {
    run(scope_member_retirement).await
}

async fn scope_member_retirement(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    // Initial sweep: parent discovers child as scope member
    store.seed_works(&[work("sm-init", None)?]).await?;
    let lease = claim(store, "sm-init", 100).await?;
    let child_resource = support::resource("child-thing")?;
    let mut child_work = work("sm-init", Some(child_resource))?;
    child_work.mark_scope_member(ResourceId::parse("listed-child")?);
    let mut page = support::page(
        lease,
        vec![support::observation("parent", "parent")?],
        true,
        None,
        101,
    );
    page.children.push(child_work);
    store.commit_page(page).await?;

    // Claim and observe the child resource in its own scope
    let child_lease = claim(store, "sm-init", 102).await?;
    let child_page = support::page(
        child_lease,
        vec![support::observation("child-thing", "child-value")?],
        true,
        None,
        103,
    );
    store.commit_page(child_page).await?;

    // Verify child was persisted
    let child_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM inventory.resources WHERE id='child-thing'")
            .fetch_one(pool)
            .await?;
    assert_eq!(child_count, 1);

    // First miss sweep: child not listed
    store.seed_works(&[work("sm-miss-one", None)?]).await?;
    let lease = claim(store, "sm-miss-one", 100).await?;
    let page = support::page(
        lease,
        vec![support::observation("parent", "parent")?],
        true,
        None,
        101,
    );
    store.commit_page(page).await?;

    // Child should still exist after first miss
    let child_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM inventory.resources WHERE id='child-thing'")
            .fetch_one(pool)
            .await?;
    assert_eq!(child_count, 1);

    // Second miss sweep: child not listed again
    store.seed_works(&[work("sm-miss-two", None)?]).await?;
    let lease = claim(store, "sm-miss-two", 100).await?;
    let page = support::page(
        lease,
        vec![support::observation("parent", "parent")?],
        true,
        None,
        101,
    );
    let stats = store.commit_page(page).await?;

    // Child should be retired after second miss
    let child_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM inventory.resources WHERE id='child-thing'")
            .fetch_one(pool)
            .await?;
    assert_eq!(child_count, 0);
    assert_eq!(stats.stats.resources_deleted, 1);

    Ok(())
}
