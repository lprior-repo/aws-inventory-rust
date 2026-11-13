//! Behavior: pre_cancelled_and_invalid_options_do_not_open_input_or_checkpoint.
use super::*;

#[tokio::test]
pub(super) async fn pre_cancelled_and_invalid_options_do_not_open_input_or_checkpoint() -> TestResult
{
    let folder = tempfile::tempdir()?;
    let input = folder.path().join("missing.jsonl");
    let checkpoint = folder.path().join("ack.sqlite");
    let token = CancellationToken::new();
    token.cancel();
    let result = graph::push_file_with_options(
        input.clone(),
        checkpoint.clone(),
        "redis://127.0.0.1:1",
        options(),
        &token,
    )
    .await;
    assert!(matches!(result, Err(GraphError::Cancelled)));
    let invalid = PushOptions {
        max_attempts: 11,
        ..options()
    };
    let result = graph::push_file_with_options(
        input,
        checkpoint.clone(),
        "redis://127.0.0.1:1",
        invalid,
        &CancellationToken::new(),
    )
    .await;
    assert!(matches!(result, Err(GraphError::Invalid(_))));
    assert!(!checkpoint.exists());
    Ok(())
}
