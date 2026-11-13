//! Behavior: custom_non_idempotent_query_gets_no_ambiguous_replay.
use super::*;

#[tokio::test]
pub(super) async fn custom_non_idempotent_query_gets_no_ambiguous_replay() -> TestResult {
    let folder = tempfile::tempdir()?;
    let input = plan(folder.path(), "retry_graph").await?;
    let checkpoint = folder.path().join("ack.sqlite");
    let mut batches = tokio::fs::read_to_string(&input)
        .await?
        .lines()
        .map(serde_json::from_str::<graph::GraphBatch>)
        .collect::<Result<Vec<_>, _>>()?;
    let resource = batches
        .iter_mut()
        .find(|batch| batch.stage == "resources")
        .ok_or("missing resource batch")?;
    resource.query = "CREATE (r:Resource {id:'custom-non-idempotent'})".into();
    let bytes = batches
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    tokio::fs::write(&input, bytes).await?;
    let server = Endpoint::start(Fault::DropAt(3), None).await?;
    let result = graph::push_file_with_options(
        input,
        checkpoint.clone(),
        &server.url,
        options(),
        &CancellationToken::new(),
    )
    .await;
    assert!(matches!(
        result,
        Err(GraphError::RetryExhausted { attempts: 1 })
    ));
    assert_eq!(server.state.queries.load(Ordering::Relaxed), 3);
    assert_eq!(server.state.connections.load(Ordering::Relaxed), 1);
    assert_eq!(acknowledgements(&checkpoint).await?, 2);
    server.close().await
}
