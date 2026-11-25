use super::{TestResult, peer};
use crate::falkor::FalkorDb;
use crate::{GraphBackend, GraphError, cypher};
use std::io::{Error, ErrorKind};
use std::time::Duration;

#[tokio::test]
async fn given_a_server_closing_after_500ms_when_writing_then_report_connection_reset() -> TestResult
{
    let peer = peer::Peer::start(|query| {
        if matches!(query, cypher::RESOURCE_INDEX | cypher::TAG_INDEX) {
            return Ok(peer::empty());
        }
        std::thread::sleep(Duration::from_millis(700));
        Err("server closed fixture".into())
    })?;
    let graph = FalkorDb::connect(&peer.endpoint, "server-closed")?;
    graph.claim_projection(1).await?;
    let result = graph.request("RETURN 1").await;
    drop(graph);
    let server_error = peer.finish().await.err().ok_or("expected server closure")?;
    assert_eq!(server_error.to_string(), "server closed fixture");
    let error = result.err().ok_or("expected connection reset")?;
    assert_eq!(
        error.to_string(),
        format!(
            "graph command failed: connection reset: {}",
            Error::from(ErrorKind::BrokenPipe)
        )
    );
    Ok(())
}

#[test]
fn given_a_refused_connection_when_converted_then_preserve_refusal() -> TestResult {
    let error = GraphError::from(redis::RedisError::from(Error::from(
        ErrorKind::ConnectionRefused,
    )));
    assert_eq!(
        error.to_string(),
        format!(
            "graph command failed: connection refused: {}",
            Error::from(ErrorKind::ConnectionRefused)
        )
    );
    Ok(())
}
