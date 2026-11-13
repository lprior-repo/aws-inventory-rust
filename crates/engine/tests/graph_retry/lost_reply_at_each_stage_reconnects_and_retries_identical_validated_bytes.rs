//! Behavior: lost_reply_at_each_stage_reconnects_and_retries_identical_validated_bytes.
use super::*;

#[tokio::test]
pub(super) async fn lost_reply_at_each_stage_reconnects_and_retries_identical_validated_bytes()
-> TestResult {
    for at in 1_u64..=5 {
        let folder = tempfile::tempdir()?;
        let input = plan(folder.path(), "retry_graph").await?;
        let checkpoint = folder.path().join("ack.sqlite");
        let server = Endpoint::start(Fault::DropAt(at), None).await?;
        let result = graph::push_file_with_options(
            input,
            checkpoint.clone(),
            &server.url,
            options(),
            &CancellationToken::new(),
        )
        .await;
        let summary = result?;
        assert_eq!(
            (
                summary.applied_batches,
                summary.attempts,
                summary.retries,
                summary.reconnects
            ),
            (5, 6, 1, 1)
        );
        let observed = server
            .state
            .observed
            .lock()
            .map_err(|_| "poisoned observations")?
            .clone();
        assert_eq!(
            observed.get(usize::try_from(at.saturating_sub(1))?),
            observed.get(usize::try_from(at)?)
        );
        assert_eq!(acknowledgements(&checkpoint).await?, 5);
        server.close().await?;
    }
    Ok(())
}
