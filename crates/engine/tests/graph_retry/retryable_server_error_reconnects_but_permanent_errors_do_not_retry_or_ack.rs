//! Behavior: retryable_server_error_reconnects_but_permanent_errors_do_not_retry_or_ack.
use super::*;

#[tokio::test]
pub(super) async fn retryable_server_error_reconnects_but_permanent_errors_do_not_retry_or_ack()
-> TestResult {
    for error in [
        "TRYAGAIN temporary",
        "NOPERM forbidden-secret",
        "ERR syntax-secret",
        "WRONGPASS authentication-secret",
    ] {
        let folder = tempfile::tempdir()?;
        let input = plan(folder.path(), "retry_graph").await?;
        let checkpoint = folder.path().join("ack.sqlite");
        let server = Endpoint::start(Fault::ErrorAt(3, error), None).await?;
        let result = graph::push_file_with_options(
            input,
            checkpoint.clone(),
            &server.url,
            options(),
            &CancellationToken::new(),
        )
        .await;
        if error.starts_with("TRYAGAIN") {
            let summary = result?;
            assert_eq!(
                (summary.applied_batches, summary.retries, summary.reconnects),
                (5, 1, 1)
            );
            assert_eq!(acknowledgements(&checkpoint).await?, 5);
        } else {
            assert!(matches!(&result, Err(GraphError::Redis(_))));
            assert!(!format!("{result:?}").contains("-secret"));
            assert_eq!(server.state.queries.load(Ordering::Relaxed), 3);
            assert_eq!(server.state.connections.load(Ordering::Relaxed), 1);
            assert_eq!(acknowledgements(&checkpoint).await?, 2);
        }
        server.close().await?;
    }
    Ok(())
}
