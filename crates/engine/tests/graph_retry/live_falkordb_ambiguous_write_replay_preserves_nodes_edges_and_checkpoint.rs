use super::*;

#[tokio::test]
#[ignore = "requires native FALKORDB_TEST_URL; proxy drops a confirmed server write response"]
pub(super) async fn live_falkordb_ambiguous_write_replay_preserves_nodes_edges_and_checkpoint()
-> TestResult {
    let upstream = std::env::var("FALKORDB_TEST_URL")?;
    let name = format!(
        "inventory_retry_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    );
    let folder = tempfile::tempdir()?;
    let input = plan(folder.path(), &name).await?;
    let checkpoint = folder.path().join("ack.sqlite");
    let server = Endpoint::start(Fault::DropAt(3), Some(upstream.clone())).await?;
    let mut connection = redis::Client::open(upstream)?
        .get_multiplexed_async_connection()
        .await?;
    let assertions = async {
        let first = graph::push_file_with_options(
            input.clone(),
            checkpoint.clone(),
            &server.url,
            options(),
            &CancellationToken::new(),
        )
        .await?;
        if (first.applied_batches, first.retries, first.reconnects) != (5, 1, 1)
            || server.state.committed_before_drop.load(Ordering::Relaxed) != 1
        {
            return Err(
                "ambiguous native write did not retry exactly once after a confirmed commit".into(),
            );
        }
        let resumed = graph::push_file_with_options(
            input.clone(),
            checkpoint.clone(),
            &server.url,
            options(),
            &CancellationToken::new(),
        )
        .await?;
        if (
            resumed.applied_batches,
            resumed.skipped_batches,
            resumed.attempts,
        ) != (0, 5, 0)
        {
            return Err("checkpointed native resume issued a graph operation".into());
        }
        graph::push_file_with_options(
            input,
            folder.path().join("fresh.sqlite"),
            &server.url,
            options(),
            &CancellationToken::new(),
        )
        .await?;
        for (query, expected) in [
            (inventory_graph::cypher::diagnostics::RESOURCE_COUNT, 2),
            (
                inventory_graph::cypher::diagnostics::LEGACY_RELATION_COUNT,
                1,
            ),
        ] {
            let value: redis::Value = redis::cmd(inventory_graph::cypher::COMMAND)
                .arg(&name)
                .arg(query)
                .query_async(&mut connection)
                .await?;
            let redis::Value::Array(parts) = value else {
                return Err("invalid graph response".into());
            };
            let Some(redis::Value::Array(rows)) = parts.get(1) else {
                return Err("missing graph rows".into());
            };
            let Some(redis::Value::Array(cells)) = rows.first() else {
                return Err("missing graph row".into());
            };
            if cells.first() != Some(&redis::Value::Int(expected)) {
                return Err("ambiguous replay changed native graph cardinality".into());
            }
        }
        if acknowledgements(&checkpoint).await? != 5 {
            return Err("native write acknowledgements were incomplete".into());
        }
        TestResult::Ok(())
    }
    .await;
    let cleanup: redis::RedisResult<redis::Value> = redis::cmd("GRAPH.DELETE")
        .arg(&name)
        .query_async(&mut connection)
        .await;
    server.close().await?;
    cleanup?;
    assertions
}
