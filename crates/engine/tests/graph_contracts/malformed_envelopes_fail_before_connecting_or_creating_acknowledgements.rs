//! Behavior: malformed_envelopes_fail_before_connecting_or_creating_acknowledgements.
use super::*;

#[tokio::test]
pub(super) async fn malformed_envelopes_fail_before_connecting_or_creating_acknowledgements()
-> TestResult {
    let folder = tempfile::tempdir()?;
    let store = frozen_store(&folder.path().join("inventory.sqlite"), 256, "small").await?;
    let original = folder.path().join("original.jsonl");
    graph::export_snapshot(&store, SCAN, "envelopes", &original).await?;
    let batches = read_batches(&original).await?;
    assert!(
        batches
            .iter()
            .any(|batch| batch.stage == "resources" && batch.rows == 256)
    );
    let valid_ack = folder.path().join("valid-ack.sqlite");
    let valid = graph::push_file(original, valid_ack.clone(), "not-a-redis-url").await;
    assert!(matches!(valid, Err(GraphError::Redis(_))));
    assert!(!valid_ack.exists());

    // Whitespace preserves the generated Cypher while reaching the advertised
    // 2 MiB query limit exactly. Validation must accept this inclusive boundary.
    let mut padded = batches.clone();
    let query = &mut stage_mut(&mut padded, "resources")?.query;
    query.push_str(&" ".repeat(QUERY_LIMIT.saturating_sub(query.len())));
    let padded_path = folder.path().join("padded.jsonl");
    write_batches(&padded_path, &padded).await?;
    let padded_ack = folder.path().join("padded-ack.sqlite");
    let valid = graph::push_file(padded_path, padded_ack.clone(), "not-a-redis-url").await;
    assert!(matches!(valid, Err(GraphError::Redis(_))));
    assert!(!padded_ack.exists());

    for (index, change) in [
        Corruption::MixedGraph,
        Corruption::MixedScan,
        Corruption::SequenceGap,
        Corruption::Version,
        Corruption::QueryBytes,
        Corruption::Rows,
        Corruption::FirstStage,
        Corruption::SecondStage,
        Corruption::RepeatedIndex,
        Corruption::BackwardStage,
        Corruption::IndexRows,
        Corruption::BeginRows,
        Corruption::EndRows,
        Corruption::TrailingBatch,
        Corruption::MissingEnd,
        Corruption::UnknownStage,
    ]
    .into_iter()
    .enumerate()
    {
        let mut damaged = batches.clone();
        corrupt(&mut damaged, change)?;
        let input = folder.path().join(format!("damaged-{index}.jsonl"));
        let ack = folder.path().join(format!("ack-{index}.sqlite"));
        write_batches(&input, &damaged).await?;
        let outcome = graph::push_file(input, ack.clone(), "not-a-redis-url").await;
        assert!(
            matches!(&outcome, Err(GraphError::Invalid(_))),
            "{change:?}: {outcome:?}"
        );
        assert!(
            !ack.exists(),
            "{change:?} created an acknowledgement database"
        );
    }
    store.close().await;
    Ok(())
}
