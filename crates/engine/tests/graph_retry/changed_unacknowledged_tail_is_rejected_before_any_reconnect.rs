//! Behavior: changed_unacknowledged_tail_is_rejected_before_any_reconnect.
use super::*;

#[tokio::test]
pub(super) async fn changed_unacknowledged_tail_is_rejected_before_any_reconnect() -> TestResult {
    let folder = tempfile::tempdir()?;
    let input = plan(folder.path(), "retry_graph").await?;
    let checkpoint = folder.path().join("ack.sqlite");
    let server = Endpoint::start(Fault::ErrorAt(3, "ERR rejected"), None).await?;
    assert!(
        graph::push_file_with_options(
            input.clone(),
            checkpoint.clone(),
            &server.url,
            options(),
            &CancellationToken::new()
        )
        .await
        .is_err()
    );
    assert_eq!(acknowledgements(&checkpoint).await?, 2);
    let mut batches = tokio::fs::read_to_string(&input)
        .await?
        .lines()
        .map(serde_json::from_str::<graph::GraphBatch>)
        .collect::<Result<Vec<_>, _>>()?;
    let last = batches.last_mut().ok_or("missing final marker")?;
    last.query = last
        .query
        .replace("state=\"complete\"", "state=\"partial\"");
    let bytes = batches
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    tokio::fs::write(&input, bytes).await?;
    let result = graph::push_file_with_options(
        input,
        checkpoint.clone(),
        &server.url,
        options(),
        &CancellationToken::new(),
    )
    .await;
    assert!(matches!(result, Err(GraphError::Invalid(_))));
    assert_eq!(server.state.queries.load(Ordering::Relaxed), 3);
    assert_eq!(server.state.connections.load(Ordering::Relaxed), 1);
    assert_eq!(acknowledgements(&checkpoint).await?, 2);
    server.close().await
}
