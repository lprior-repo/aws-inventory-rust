use inventory_core::{CollectionPage, CollectionScope, Edge, Resource, ScanStatus, Source};
use inventory_engine::{
    graph::{self, GraphError, PushOptions},
    store::Store,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;
use tokio::task::{JoinHandle, JoinSet};
use tokio_util::sync::CancellationToken;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

fn options() -> PushOptions {
    PushOptions {
        max_attempts: 3,
        attempt_timeout_ms: 2_000,
        max_backoff_ms: 1,
        overall_timeout_ms: 10_000,
    }
}

async fn plan(
    folder: &Path,
    name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let store = Store::open(folder.join("inventory.sqlite")).await?;
    store.begin_scan("retry-scan", 1).await?;
    store
        .apply_page(
            "retry-scan",
            CollectionPage {
                scope: CollectionScope {
                    account_id: "123456789012".into(),
                    region: "us-east-1".into(),
                    collector: "ec2".into(),
                    parent_id: None,
                    source: Source::Direct,
                },
                page_number: 1,
                next_token: None,
                resources: vec![Resource {
                    id: "instance-one".into(),
                    account_id: "123456789012".into(),
                    region: "us-east-1".into(),
                    kind: "aws_ec2_instance".into(),
                    name: Some("quote\" slash\\ newline\n 😃".into()),
                    properties: json!({"text":"x\"}] DELETE n //"}),
                    tags: BTreeMap::new(),
                    source: Source::Direct,
                    observed_at: 1,
                }],
                edges: vec![Edge {
                    from_id: "instance-one".into(),
                    to_id: "unresolved-vpc".into(),
                    relation: "IN_VPC".into(),
                    source: Source::Direct,
                    observed_at: 1,
                    properties: json!({}),
                }],
            },
        )
        .await?;
    store
        .finish_scan("retry-scan", ScanStatus::Complete, 2)
        .await?;
    let output = folder.join("plan.jsonl");
    graph::export_snapshot(&store, "retry-scan", name, &output).await?;
    store.close().await;
    Ok(output)
}

async fn acknowledgements(path: &Path) -> Result<i64, sqlx::Error> {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(path))
        .await?;
    let result = sqlx::query_scalar("SELECT count(*) FROM projection_ack")
        .fetch_one(&pool)
        .await;
    pool.close().await;
    result
}

#[path = "graph_retry/cancellation_and_overall_deadline_interrupt_pending_network_without_ack.rs"]
mod cancellation_and_overall_deadline_interrupt_pending_network_without_ack;
#[path = "graph_retry/changed_unacknowledged_tail_is_rejected_before_any_reconnect.rs"]
mod changed_unacknowledged_tail_is_rejected_before_any_reconnect;
#[path = "graph_retry/custom_non_idempotent_query_gets_no_ambiguous_replay.rs"]
mod custom_non_idempotent_query_gets_no_ambiguous_replay;
#[path = "graph_retry/endpoint.rs"]
mod endpoint;
#[path = "graph_retry/live_falkordb_ambiguous_write_replay_preserves_nodes_edges_and_checkpoint.rs"]
mod live_falkordb_ambiguous_write_replay_preserves_nodes_edges_and_checkpoint;
#[path = "graph_retry/lost_reply_at_each_stage_reconnects_and_retries_identical_validated_bytes.rs"]
mod lost_reply_at_each_stage_reconnects_and_retries_identical_validated_bytes;
#[path = "graph_retry/pre_cancelled_and_invalid_options_do_not_open_input_or_checkpoint.rs"]
mod pre_cancelled_and_invalid_options_do_not_open_input_or_checkpoint;
#[path = "graph_retry/retry_budget_is_exact_and_no_failed_write_is_acknowledged.rs"]
mod retry_budget_is_exact_and_no_failed_write_is_acknowledged;
#[path = "graph_retry/retryable_server_error_reconnects_but_permanent_errors_do_not_retry_or_ack.rs"]
mod retryable_server_error_reconnects_but_permanent_errors_do_not_retry_or_ack;
use endpoint::*;
#[path = "graph_retry/protocol.rs"]
mod protocol;
use protocol::*;
