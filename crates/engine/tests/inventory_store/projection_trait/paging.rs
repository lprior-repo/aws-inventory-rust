use super::fault::{Fault, FaultStore};
use super::{Call, RecordingBackend, TestResult, pending_resource};
use inventory_engine::inventory::{BinaryStore, InventoryStore, ProjectionReport, run_projection};
use inventory_graph::{NodeRow, TagRow};
use std::sync::atomic::Ordering;

async fn tagged_resource() -> TestResult<(tempfile::TempDir, BinaryStore, NodeRow)> {
    let (directory, store, row) = pending_resource().await?;
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    for (key, value) in [("a", "first"), ("b", "second")] {
        sqlx::query(
            "INSERT INTO tags(account,region,kind,id,tag_key,tag_value) VALUES(?1,?2,?3,?4,?5,?6)",
        )
        .bind(row.key.account.as_str())
        .bind(row.key.region.as_str())
        .bind(row.key.kind.as_str())
        .bind(row.key.id.as_str())
        .bind(key)
        .bind(value)
        .execute(&pool)
        .await?;
    }
    pool.close().await;
    Ok((directory, store, row))
}

fn tag(key: &str, value: &str, digest: &str) -> TestResult<TagRow> {
    let bytes = digest
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            Ok::<_, Box<dyn std::error::Error>>(u8::from_str_radix(std::str::from_utf8(pair)?, 16)?)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TagRow {
        key: key.into(),
        value: value.into(),
        identity: bytes
            .try_into()
            .map_err(|_| "invalid expected tag digest")?,
    })
}

fn first_tag() -> TestResult<TagRow> {
    tag(
        "a",
        "first",
        "8497d11972eca81a8ac0f594c1862dbb1baeeb02c1c4b34ad47b1f8585b840a6",
    )
}

#[tokio::test]
async fn given_two_tag_pages_when_runner_projects_then_every_tag_precedes_acknowledgement()
-> TestResult {
    let (_directory, store, expected) = tagged_resource().await?;
    let graph = RecordingBackend::new(false);
    let result = run_projection(&store, &graph, 1).await?;
    assert_eq!(
        graph.calls().await,
        vec![
            Call::Claim,
            Call::Upsert(expected.clone()),
            Call::ClearTags(expected.key.clone()),
            Call::Tag(expected.key.clone(), first_tag()?),
            Call::Tag(
                expected.key.clone(),
                tag(
                    "b",
                    "second",
                    "ba9041603b726bc0c4870c671c18a1298f504d65bb3e7e066d055301a62926c9"
                )?
            ),
            Call::Replace(expected.key, Vec::new()),
        ]
    );
    assert_eq!(
        result,
        ProjectionReport {
            acknowledged: 1,
            pending: 0,
            offline: false,
            rebuild_incomplete: false
        }
    );
    assert_eq!(store.drain_projection(1).await?, Vec::new());
    Ok(())
}

#[tokio::test]
async fn given_stuck_tag_cursor_when_runner_exhausts_page_budget_then_error_keeps_the_revision()
-> TestResult {
    let (_directory, inner, expected) = tagged_resource().await?;
    let store = FaultStore::new(inner, Fault::Tags);
    let pending = store.drain_projection(1).await?;
    let graph = RecordingBackend::new(false);
    let error = run_projection(&store, &graph, 1)
        .await
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "projection storage failed: schema or stored data is invalid: projection tag page bound exceeded"
    );
    assert_eq!(store.reads.load(Ordering::SeqCst), 10_000);
    assert_eq!(store.acknowledgements.load(Ordering::SeqCst), 0);
    assert_eq!(store.drain_projection(1).await?, pending);
    let calls = [
        Call::Claim,
        Call::Upsert(expected.clone()),
        Call::ClearTags(expected.key.clone()),
    ]
    .into_iter()
    .chain(std::iter::repeat_n(
        Call::Tag(expected.key, first_tag()?),
        10_000,
    ))
    .collect::<Vec<_>>();
    assert_eq!(graph.calls().await, calls);
    Ok(())
}

#[tokio::test]
async fn given_short_last_resource_page_when_runner_rebuilds_then_it_acknowledges_exactly_once()
-> TestResult {
    let (_directory, inner, expected) = pending_resource().await?;
    let store = FaultStore::new(inner, Fault::Resources);
    store.request_graph_rebuild().await?;
    let graph = RecordingBackend::new(false);
    let result = run_projection(&store, &graph, 2).await?;
    assert_eq!(
        graph.calls().await,
        vec![
            Call::Claim,
            Call::Clear,
            Call::Upsert(expected.clone()),
            Call::Replace(expected.key, Vec::new())
        ]
    );
    assert_eq!(
        result,
        ProjectionReport {
            acknowledged: 1,
            pending: 0,
            offline: false,
            rebuild_incomplete: false
        }
    );
    assert_eq!(store.reads.load(Ordering::SeqCst), 2);
    assert_eq!(store.acknowledgements.load(Ordering::SeqCst), 1);
    assert_eq!(store.drain_projection(1).await?, Vec::new());
    assert!(!store.projection_state().await?.rebuild_required);
    Ok(())
}

#[tokio::test]
async fn given_nonterminal_full_pages_when_rebuild_budget_ends_then_marker_is_not_acknowledged()
-> TestResult {
    let (_directory, inner, expected) = pending_resource().await?;
    let store = FaultStore::new(inner, Fault::Resources);
    store.request_graph_rebuild().await?;
    let pending = store.drain_projection(1).await?;
    let graph = RecordingBackend::new(false);
    let result = run_projection(&store, &graph, 1).await?;
    assert_eq!(
        result,
        ProjectionReport {
            acknowledged: 0,
            pending: 1,
            offline: false,
            rebuild_incomplete: true
        }
    );
    assert_eq!(store.reads.load(Ordering::SeqCst), 10_001);
    assert_eq!(store.acknowledgements.load(Ordering::SeqCst), 0);
    assert_eq!(store.drain_projection(1).await?, pending);
    assert!(store.projection_state().await?.rebuild_required);
    let calls = [Call::Claim, Call::Clear]
        .into_iter()
        .chain((0..10_000).map(|_| Call::Upsert(expected.clone())))
        .collect::<Vec<_>>();
    assert_eq!(graph.calls().await, calls);
    Ok(())
}
