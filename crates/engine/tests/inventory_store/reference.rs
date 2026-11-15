use inventory_core::{inventory::{Edge, EdgeSource, Observation, ReferenceTarget, Relation, UnresolvedReference}, record::FieldId};
use inventory_engine::inventory::{BinaryStore, ProjectionEdgeHeader, ProjectionEdgePage, SweepId};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use super::support::*;

#[tokio::test]
async fn given_a_target_identity_value_changes_when_an_old_reference_exists_then_its_edge_is_retracted_and_reference_kept()
-> TestResult {
    let (directory, store, _) = fixture().await?;
    store
        .seed_work(relationship_work("reference-before", "us-east-1", Some("old-name"), None)?)
        .await?;
    let first_sweep = SweepId::new("reference-before".to_owned())?;
    let first = store
        .claim(&first_sweep, 100, 20)
        .await?
        .ok_or("missing initial lease")?;
    let mut initial = page(
        first,
        vec![
            observation("target", "old-name")?,
            observation("source", "source-name")?,
        ],
        true,
        None,
        101,
    );
    initial.unresolved.push(unresolved("source", "old-name")?);
    store.commit_page(initial).await?;

    store
        .seed_work(relationship_work("reference-after", "us-east-1", Some("old-name"), None)?)
        .await?;
    let second_sweep = SweepId::new("reference-after".to_owned())?;
    let second = store
        .claim(&second_sweep, 200, 20)
        .await?
        .ok_or("missing update lease")?;
    store
        .commit_page(page(
            second,
            vec![observation("target", "new-name")?],
            false,
            Some(vec![8]),
            201,
        ))
        .await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    let stale_edges: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM edges WHERE from_id='source' AND to_id='target'")
            .fetch_one(&pool)
            .await?;
    let retained_references: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM unresolved_refs WHERE from_id='source'")
            .fetch_one(&pool)
            .await?;
    pool.close().await;

    assert_eq!(stale_edges, 0);
    assert_eq!(retained_references, 1);
    Ok(())
}

fn dependency(path: u32) -> TestResult<UnresolvedReference> {
    let from = resource("source")?;
    Ok(UnresolvedReference {
        target: ReferenceTarget::new(
            from.account.clone(), from.region.clone(), from.kind.clone(),
            vec![FieldId::new(path)?], inventory_core::inventory::ResourceId::parse("beta")?,
        )?,
        from, relation: Relation::parse("depends-on")?, source: EdgeSource::Model,
    })
}

fn pinned(target: &str) -> TestResult<Edge> {
    Ok(Edge {
        from: resource("source")?, to: resource(target)?,
        relation: Relation::parse("depends-on")?, source: EdgeSource::Model,
    })
}

async fn commit_beta_page(
    store: &BinaryStore, sweep: &str, now: i64, observations: Vec<Observation>,
    edges: Vec<Edge>, references: Vec<UnresolvedReference>,
) -> TestResult {
    store.seed_work(relationship_work(sweep, "us-east-1", None, Some("beta"))?).await?;
    let lease = store.claim(&SweepId::new(sweep.to_owned())?, now, 20)
        .await?.ok_or("missing beta dependency lease")?;
    let mut commit = page(lease, observations, true, None, now.checked_add(1).ok_or("test timestamp overflow")?);
    commit.edges = edges;
    commit.unresolved = references;
    store.commit_page(commit).await?;
    Ok(())
}

async fn assert_dependency_edges(store: &BinaryStore, targets: &[&str]) -> TestResult {
    let expected = targets.iter().map(|target| Ok(ProjectionEdgeHeader {
        from: resource("source")?, to: resource(target)?,
        relation: Relation::parse("depends-on")?, source: "model".to_owned(),
    })).collect::<TestResult<Vec<_>>>()?;
    assert_eq!(store.projection_edges_for(&resource("source")?, None, 10).await?,
        ProjectionEdgePage { edges: expected, next: None });
    Ok(())
}

async fn assert_reopened_dependency_edges(directory: &tempfile::TempDir, targets: &[&str]) -> TestResult {
    let store = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_dependency_edges(&store, targets).await
}

#[tokio::test]
async fn given_two_independent_paths_prove_one_edge_when_either_column_changes_then_only_loss_of_the_last_proof_retracts()
-> TestResult {
    // KindSchema sorts columns by path. Swap which path becomes invalid so the
    // surviving proof is visited both before and after the invalidating proof.
    for name_changes_first in [false, true] {
        let (directory, store) = dual_fixture().await?;
        commit_beta_page(&store, "two-proofs", 100, vec![
            dual_observation("target", "beta", "beta", "initial")?, observation("source", "alpha")?,
        ], Vec::new(), vec![dependency(1)?, dependency(4)?]).await?;
        assert_dependency_edges(&store, &["target"]).await?;

        let (name, alias) = if name_changes_first { ("gamma", "beta") } else { ("beta", "gamma") };
        commit_beta_page(&store, "one-proof", 200,
            vec![dual_observation("target", name, alias, "one-proof")?], Vec::new(), Vec::new()).await?;
        assert_dependency_edges(&store, &["target"]).await?;
        assert_reopened_dependency_edges(&directory, &["target"]).await?;

        commit_beta_page(&store, "no-proofs", 300,
            vec![dual_observation("target", "gamma", "gamma", "no-proofs")?], Vec::new(), Vec::new()).await?;
        assert_dependency_edges(&store, &[]).await?;
        assert_reopened_dependency_edges(&directory, &[]).await?;
    }
    Ok(())
}

#[tokio::test]
async fn given_an_ambiguous_proof_and_an_independent_unique_proof_when_target_columns_refresh_then_the_unique_edge_survives_in_both_orders()
-> TestResult {
    for name_is_ambiguous in [false, true] {
        let (directory, store) = dual_fixture().await?;
        let (other_name, other_alias) = if name_is_ambiguous { ("beta", "other") } else { ("other", "beta") };
        commit_beta_page(&store, "ambiguous-and-unique", 100, vec![
            dual_observation("target", "beta", "beta", "initial")?,
            dual_observation("other", other_name, other_alias, "initial")?,
            observation("source", "alpha")?,
        ], Vec::new(), vec![dependency(1)?, dependency(4)?]).await?;
        assert_dependency_edges(&store, &["target"]).await?;
        // Change a modeled non-proof revision so a real target-column refresh
        // runs while both proof values remain unchanged.
        commit_beta_page(&store, "refresh-both", 200, vec![
            dual_observation("target", "beta", "beta", "refreshed")?,
        ], Vec::new(), Vec::new()).await?;
        assert_dependency_edges(&store, &["target"]).await?;
        assert_reopened_dependency_edges(&directory, &["target"]).await?;

        let (name, alias) = if name_is_ambiguous { ("beta", "gamma") } else { ("gamma", "beta") };
        commit_beta_page(&store, "lose-unique", 300,
            vec![dual_observation("target", name, alias, "no-unique-proof")?], Vec::new(), Vec::new()).await?;
        assert_dependency_edges(&store, &[]).await?;
        assert_reopened_dependency_edges(&directory, &[]).await?;
    }
    Ok(())
}

#[tokio::test]
async fn given_two_explicit_same_value_targets_and_an_ambiguous_wildcard_when_committed_then_only_both_pinned_identities_are_projected()
-> TestResult {
    let (directory, store, _) = fixture().await?;
    commit_beta_page(&store, "two-pins-and-wildcard", 100, vec![
        observation("target-a", "beta")?, observation("target-b", "beta")?,
        observation("target-c", "beta")?, observation("source", "alpha")?,
    ], vec![pinned("target-a")?, pinned("target-b")?], vec![dependency(1)?]).await?;
    // The third same-name candidate must not acquire an edge from the ambiguous
    // wildcard, and ambiguity must not erase either explicit identity proof.
    assert_dependency_edges(&store, &["target-a", "target-b"]).await?;
    assert_reopened_dependency_edges(&directory, &["target-a", "target-b"]).await?;
    Ok(())
}

#[tokio::test]
async fn given_a_wildcard_is_replaced_by_a_pin_with_the_same_reference_tuple_when_its_target_changes_then_the_old_wildcard_cannot_redirect()
-> TestResult {
    let (directory, store, _) = fixture().await?;
    commit_beta_page(&store, "old-wildcard", 100, vec![
        observation("target-a", "beta")?, observation("target-b", "beta")?, observation("source", "alpha")?,
    ], Vec::new(), vec![dependency(1)?]).await?;
    assert_dependency_edges(&store, &[]).await?;
    commit_beta_page(&store, "replace-with-pin", 200, vec![
        observation("target-a", "beta")?, observation("source", "alpha")?,
    ], vec![pinned("target-a")?], Vec::new()).await?;
    assert_dependency_edges(&store, &["target-a"]).await?;
    assert_reopened_dependency_edges(&directory, &["target-a"]).await?;
    commit_beta_page(&store, "pin-value-changed", 300,
        vec![observation("target-a", "gamma")?], Vec::new(), Vec::new()).await?;
    assert_dependency_edges(&store, &[]).await?;
    assert_reopened_dependency_edges(&directory, &[]).await?;
    Ok(())
}

#[tokio::test]
async fn given_a_pin_is_replaced_by_another_identity_with_the_same_reference_tuple_when_committed_then_the_old_identity_grant_is_retired()
-> TestResult {
    let (directory, store, _) = fixture().await?;
    commit_beta_page(&store, "old-pin", 100, vec![
        observation("target-a", "beta")?, observation("target-b", "beta")?, observation("source", "alpha")?,
    ], vec![pinned("target-a")?], Vec::new()).await?;
    assert_dependency_edges(&store, &["target-a"]).await?;
    commit_beta_page(&store, "new-pin", 200, vec![
        observation("target-b", "beta")?, observation("source", "alpha")?,
    ], vec![pinned("target-b")?], Vec::new()).await?;
    assert_dependency_edges(&store, &["target-b"]).await?;
    assert_reopened_dependency_edges(&directory, &["target-b"]).await?;
    commit_beta_page(&store, "new-pin-value-changed", 300,
        vec![observation("target-b", "gamma")?], Vec::new(), Vec::new()).await?;
    assert_dependency_edges(&store, &[]).await?;
    assert_reopened_dependency_edges(&directory, &[]).await?;
    Ok(())
}
