use super::super::support::{TestResult, fixture, observation, page, work};
use super::backend::{Call, RecordingBackend};
use inventory_engine::inventory::{
    BinaryStore, InventoryStore, ProjectionIdentity, SweepId, run_projection,
};

async fn commit(store: &BinaryStore, sweep: &str, at: i64, values: &[(&str, &str)]) -> TestResult {
    store.seed_work(work(sweep, "us-east-1")?).await?;
    let lease = store
        .claim(&SweepId::new(sweep.into())?, at, 100)
        .await?
        .ok_or("missing lease")?;
    let observations = values
        .iter()
        .map(|(id, value)| observation(id, value))
        .collect::<Result<Vec<_>, _>>()?;
    store
        .commit_page(page(lease, observations, true, None, at.saturating_add(1)))
        .await?;
    Ok(())
}

fn previous_destination() -> ProjectionIdentity {
    ProjectionIdentity {
        endpoint: "recording://previous-endpoint".into(),
        graph: "trait_contract".into(),
    }
}

#[tokio::test]
async fn given_a_and_b_projected_and_a_pending_when_target_changes_then_no_delta_reaches_the_new_target_and_a_full_rebuild_is_required()
-> TestResult {
    let (_directory, store, _) = fixture().await?;
    commit(&store, "first", 1, &[("a", "one"), ("b", "one")]).await?;
    let previous = RecordingBackend::new(false);
    assert_eq!(run_projection(&store, &previous, 1000).await?.pending, 0);
    store
        .write_projection_identity(&previous_destination())
        .await?;
    commit(&store, "second", 10, &[("a", "two")]).await?;
    assert_eq!(store.projection_state().await?.pending, 1);

    let fresh = RecordingBackend::new(false);
    let report = run_projection(&store, &fresh, 1000).await?;

    assert!(report.rebuild_incomplete);
    assert_eq!((report.pending, report.acknowledged), (1, 0));
    assert_eq!(
        fresh.calls().await,
        Vec::new(),
        "no delta for the old graph"
    );
    assert_eq!(
        store.read_projection_identity().await?,
        Some(previous_destination()),
        "the new destination is not accepted before a rebuild"
    );
    let state = store.projection_state().await?;
    assert_eq!((state.pending, state.rebuild_required), (1, true));
    Ok(())
}

#[tokio::test]
async fn given_a_pending_delta_and_requested_rebuild_when_target_changed_then_the_new_target_is_fully_rebuilt_before_identity_is_accepted()
-> TestResult {
    let (_directory, store, _) = fixture().await?;
    commit(&store, "first", 1, &[("a", "one"), ("b", "one")]).await?;
    run_projection(&store, &RecordingBackend::new(false), 1000).await?;
    store
        .write_projection_identity(&previous_destination())
        .await?;
    commit(&store, "second", 10, &[("a", "two")]).await?;
    let fresh = RecordingBackend::new(false);
    assert!(
        run_projection(&store, &fresh, 1000)
            .await?
            .rebuild_incomplete
    );

    store.request_graph_rebuild().await?;
    let report = run_projection(&store, &fresh, 1000).await?;

    assert!(!report.rebuild_incomplete && !report.offline);
    assert_eq!(report.pending, 0);
    let calls = fresh.calls().await;
    assert!(calls.contains(&Call::Clear));
    let upserts = calls
        .iter()
        .filter(|call| matches!(call, Call::Upsert(_)))
        .count();
    assert_eq!(upserts, 2, "both A and B reach the new target");
    let state = store.projection_state().await?;
    assert_eq!((state.pending, state.rebuild_required), (0, false));
    let identity = store
        .read_projection_identity()
        .await?
        .ok_or("missing identity")?;
    assert_eq!(identity.endpoint, "recording://projection");
    Ok(())
}
