use super::*;

#[tokio::test]
pub(super) async fn retry_budget_is_exact_and_no_failed_write_is_acknowledged() -> TestResult {
    let folder = tempfile::tempdir()?;
    let input = plan(folder.path(), "retry_graph").await?;
    let checkpoint = folder.path().join("ack.sqlite");
    let server = Endpoint::start(Fault::AlwaysRetry, None).await?;
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
        Err(GraphError::RetryExhausted { attempts: 3 })
    ));
    assert_eq!(server.state.queries.load(Ordering::Relaxed), 3);
    assert_eq!(acknowledgements(&checkpoint).await?, 0);
    server.close().await
}
