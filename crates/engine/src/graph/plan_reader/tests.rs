//! Public delivery scenarios for the graph plan's ordered stage protocol.
use crate::graph::{GraphBatch, GraphError, push_file};
use futures_util::{StreamExt, TryStreamExt, stream};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn batch(sequence: usize, stage: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(serde_json::to_string(&GraphBatch {
        version: 1,
        graph: "graph".into(),
        scan_id: "scan".into(),
        sequence: u64::try_from(sequence)?,
        stage: stage.into(),
        rows: 0,
        query: "RETURN 1".into(),
    })?)
}

async fn assert_plan(stages: &[&str], expected: &str) -> TestResult {
    // Given a real plan file and a URL that cannot initiate network I/O.
    let directory = tempfile::tempdir()?;
    let input = directory.path().join("plan.jsonl");
    let checkpoint = directory.path().join("checkpoint.sqlite");
    let records = stages
        .iter()
        .enumerate()
        .map(|(sequence, stage)| batch(sequence, stage))
        .collect::<Result<Vec<_>, _>>()?;
    tokio::fs::write(&input, records.join("\n")).await?;
    // When delivery validates the entire plan before opening its destination.
    let outcome = push_file(input, checkpoint.clone(), "not-a-redis-url").await;
    // Then validation either returns its exact error or reaches URL parsing.
    let observed = match outcome {
        Err(GraphError::Invalid(message)) => message,
        Err(GraphError::Redis(_)) => "validated".into(),
        other => format!("unexpected delivery result: {other:?}"),
    };
    assert_eq!(observed, expected, "stages: {stages:?}");
    assert!(!checkpoint.exists());
    Ok(())
}

async fn assert_plans(cases: &[(&[&str], &str)]) -> TestResult {
    stream::iter(cases)
        .map(Ok)
        .try_for_each(|(stages, expected)| assert_plan(stages, expected))
        .await
}

#[tokio::test]
async fn delivery_accepts_empty_and_repeated_ordered_data_stages() -> TestResult {
    assert_plans(&[
        (&["index", "begin", "end"], "validated"),
        (
            &["index", "begin", "resources", "resources", "end"],
            "validated",
        ),
        (&["index", "begin", "edges", "edges", "end"], "validated"),
        (
            &["index", "begin", "resources", "edges", "end"],
            "validated",
        ),
    ])
    .await
}

#[tokio::test]
async fn delivery_requires_index_then_begin_before_data_or_end() -> TestResult {
    let error = "projection stages out of order";
    assert_plans(&[
        (&["begin"], error),
        (&["resources"], error),
        (&["edges"], error),
        (&["end"], error),
        (&["index", "index"], error),
        (&["index", "resources"], error),
        (&["index", "edges"], error),
        (&["index", "end"], error),
    ])
    .await
}

#[tokio::test]
async fn delivery_rejects_backward_or_repeated_control_stages() -> TestResult {
    let error = "projection stages out of order";
    assert_plans(&[
        (&["index", "begin", "index"], error),
        (&["index", "begin", "begin"], error),
        (&["index", "begin", "resources", "begin"], error),
        (&["index", "begin", "edges", "resources"], error),
    ])
    .await
}

#[tokio::test]
async fn delivery_requires_end_at_eof_and_rejects_every_stage_after_end() -> TestResult {
    let missing = "projection is empty or lacks a final end marker";
    let trailing = "mixed identities, noncontiguous sequence, or data after end marker";
    assert_plans(&[
        (&[], missing),
        (&["index"], missing),
        (&["index", "begin"], missing),
        (&["index", "begin", "resources"], missing),
        (&["index", "begin", "edges"], missing),
        (&["index", "begin", "end", "index"], trailing),
        (&["index", "begin", "end", "begin"], trailing),
        (&["index", "begin", "end", "resources"], trailing),
        (&["index", "begin", "end", "edges"], trailing),
        (&["index", "begin", "end", "end"], trailing),
    ])
    .await
}

#[tokio::test]
async fn delivery_rejects_unknown_stage_spellings_at_the_boundary() -> TestResult {
    assert_plans(&[
        (&[""], "invalid projection stage"),
        (&["INDEX"], "invalid projection stage"),
        (&["index", "begin", "unknown"], "invalid projection stage"),
    ])
    .await
}
