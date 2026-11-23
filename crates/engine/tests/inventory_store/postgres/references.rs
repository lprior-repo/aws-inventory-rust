#[path = "references/concurrent.rs"]
mod concurrent;

use super::{TestResult, claim, collect, run, support, work};
use inventory_core::inventory::{Edge, Observation};
use inventory_engine::inventory::{DatabaseUrl, PageCommit, PgStore, StoreError};
use sqlx::PgPool;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_late_target_when_ambiguous_or_renamed_then_edges_track_current_unique_matches()
-> TestResult {
    run(late).await
}

async fn late(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    observe(store, "source-only", &[], false).await?;
    assert!(edges(pool).await?.is_empty());
    observe(store, "one-target", &[("target-one", "target-name")], false).await?;
    assert_eq!(edges(pool).await?, ["target-one"]);
    observe_two(store, "two-targets", "target-name", "target-name", false).await?;
    assert!(edges(pool).await?.is_empty());
    observe_two(store, "renamed-one", "renamed", "target-name", false).await?;
    assert_eq!(edges(pool).await?, ["target-two"]);
    observe_two(store, "renamed-both", "renamed", "renamed", false).await?;
    assert!(edges(pool).await?.is_empty());
    assert_eq!(status(pool).await?, "unresolved");
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_ambiguous_unconstrained_proof_when_independent_pin_exists_then_only_pin_survives()
-> TestResult {
    run(pinned).await
}

async fn pinned(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    observe_two(store, "pin", "target-name", "target-name", true).await?;
    assert_eq!(edges(pool).await?, ["target-one"]);
    assert_eq!(status(pool).await?, "resolved");
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_target_retirement_when_recreated_then_waiting_reference_resolves_again() -> TestResult
{
    run(rearrival).await
}

async fn rearrival(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    observe(store, "arrival", &[("target-one", "target-name")], false).await?;
    observe(store, "target-miss-one", &[], false).await?;
    assert_eq!(edges(pool).await?, ["target-one"]);
    observe(store, "target-miss-two", &[], false).await?;
    assert!(edges(pool).await?.is_empty());
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM inventory.resources WHERE id='target-one'")
            .fetch_one(pool)
            .await?;
    assert_eq!(count, 0);
    observe(store, "rearrival", &[("target-one", "target-name")], false).await?;
    assert_eq!(edges(pool).await?, ["target-one"]);
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_forged_relationship_when_page_commits_then_no_authority_or_resource_is_minted()
-> TestResult {
    run(forged).await
}

async fn forged(store: &PgStore, pool: &PgPool, _: &DatabaseUrl) -> TestResult {
    store.seed_works(&[work("forged", None)?]).await?;
    let mut page = support::page(
        claim(store, "forged", 100).await?,
        vec![support::observation("source", "source-name")?],
        true,
        None,
        101,
    );
    page.unresolved
        .push(support::unresolved("source", "forged")?);
    assert!(matches!(
        store.commit_page(page).await,
        Err(StoreError::Invalid(_))
    ));
    let counts: (i64, i64, i64) = sqlx::query_as("SELECT (SELECT count(*) FROM inventory.resources),(SELECT count(*) FROM inventory.reviewed_reference_grants),(SELECT count(*) FROM inventory.reviewed_reference_evidence)")
        .fetch_one(pool).await?;
    assert_eq!(counts, (0, 0, 0));
    Ok(())
}

async fn observe(store: &PgStore, sweep: &str, targets: &[(&str, &str)], pin: bool) -> TestResult {
    store.seed_works(&[work(sweep, None)?]).await?;
    let mut page = support::page(
        claim(store, sweep, 100).await?,
        observations(targets)?,
        true,
        None,
        101,
    );
    relationships(&mut page, pin)?;
    store.commit_page(page).await?;
    Ok(())
}

async fn observe_two(
    store: &PgStore,
    sweep: &str,
    first: &str,
    second: &str,
    pin: bool,
) -> TestResult {
    observe(
        store,
        sweep,
        &[("target-one", first), ("target-two", second)],
        pin,
    )
    .await
}

async fn status(pool: &PgPool) -> TestResult<String> {
    Ok(sqlx::query_scalar(
        "SELECT resolution FROM inventory.unresolved_refs WHERE from_id='source'",
    )
    .fetch_one(pool)
    .await?)
}

fn observations(targets: &[(&str, &str)]) -> TestResult<Vec<Observation>> {
    std::iter::once(support::observation("source", "source-name"))
        .chain(
            targets
                .iter()
                .map(|(id, value)| support::observation(id, value)),
        )
        .collect()
}

fn relationships(page: &mut PageCommit, pin: bool) -> TestResult {
    let reference = support::unresolved("source", "target-name")?;
    if pin {
        page.edges.push(Edge {
            from: reference.from.clone(),
            to: support::resource("target-one")?,
            relation: reference.relation.clone(),
            source: reference.source,
        });
    }
    page.unresolved.push(reference);
    Ok(())
}

async fn edges(pool: &PgPool) -> TestResult<Vec<String>> {
    Ok(sqlx::query_scalar(
        "SELECT to_id FROM inventory.edges WHERE from_id='source' ORDER BY to_kind,to_id",
    )
    .fetch_all(pool)
    .await?)
}
