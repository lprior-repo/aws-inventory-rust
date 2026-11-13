//! Behavior: cancellation_and_overall_deadline_interrupt_pending_network_without_ack.
use super::*;

#[tokio::test]
pub(super) async fn cancellation_and_overall_deadline_interrupt_pending_network_without_ack()
-> TestResult {
    for explicit in [true, false] {
        let folder = tempfile::tempdir()?;
        let input = plan(folder.path(), "retry_graph").await?;
        let checkpoint = folder.path().join("ack.sqlite");
        let server = Endpoint::start(Fault::Hang, None).await?;
        let token = CancellationToken::new();
        let config = PushOptions {
            overall_timeout_ms: 200,
            ..options()
        };
        let operation =
            graph::push_file_with_options(input, checkpoint.clone(), &server.url, config, &token);
        let canceller = async {
            server.query_started().await?;
            if explicit {
                token.cancel();
            }
            TestResult::Ok(())
        };
        let (result, cancelled) = tokio::join!(operation, canceller);
        cancelled?;
        assert!(if explicit {
            matches!(result, Err(GraphError::Cancelled))
        } else {
            matches!(result, Err(GraphError::Deadline))
        });
        assert_eq!(server.state.queries.load(Ordering::Relaxed), 1);
        assert_eq!(acknowledgements(&checkpoint).await?, 0);
        server.close().await?;
    }
    Ok(())
}
