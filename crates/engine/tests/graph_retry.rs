//! Real TCP fault injection; the optional live case also uses native FalkorDB.
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

#[derive(Clone, Copy)]
enum Fault {
    DropAt(u64),
    ErrorAt(u64, &'static str),
    AlwaysRetry,
    Hang,
}

struct State {
    queries: AtomicU64,
    connections: AtomicU64,
    committed_before_drop: AtomicU64,
    observed: Mutex<Vec<String>>,
    query_started: Notify,
    fault: Fault,
}

struct Endpoint {
    url: String,
    state: Arc<State>,
    stop: CancellationToken,
    task: Option<JoinHandle<io::Result<()>>>,
}

impl Endpoint {
    async fn start(fault: Fault, backend: Option<String>) -> Result<Self, io::Error> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
        let url = format!("redis://{}", listener.local_addr()?);
        let state = Arc::new(State {
            queries: AtomicU64::new(0),
            connections: AtomicU64::new(0),
            committed_before_drop: AtomicU64::new(0),
            observed: Mutex::new(Vec::new()),
            query_started: Notify::new(),
            fault,
        });
        let stop = CancellationToken::new();
        let owner = stop.clone();
        let shared = Arc::clone(&state);
        let task = tokio::spawn(async move {
            let mut clients = JoinSet::new();
            for _ in 0..64 {
                let accepted = tokio::select! {
                    biased;
                    () = owner.cancelled() => break,
                    result = listener.accept() => result,
                };
                let (socket, _) = accepted?;
                shared.connections.fetch_add(1, Ordering::Relaxed);
                let state = Arc::clone(&shared);
                let cancellation = owner.clone();
                let target = backend.clone();
                clients.spawn(async move {
                    tokio::select! {
                        biased;
                        () = cancellation.cancelled() => Ok(()),
                        result = serve(socket, state, target) => result,
                    }
                });
            }
            clients.abort_all();
            while let Some(result) = clients.join_next().await {
                match result {
                    Ok(Ok(())) => {}
                    Ok(Err(error))
                        if matches!(
                            error.kind(),
                            io::ErrorKind::BrokenPipe
                                | io::ErrorKind::UnexpectedEof
                                | io::ErrorKind::ConnectionReset
                        ) => {}
                    Err(error) if error.is_cancelled() => {}
                    Ok(Err(error)) => return Err(error),
                    Err(error) => return Err(io::Error::other(error)),
                }
            }
            Ok(())
        });
        Ok(Self {
            url,
            state,
            stop,
            task: Some(task),
        })
    }

    async fn query_started(&self) -> TestResult {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let notified = self.state.query_started.notified();
                if self.state.queries.load(Ordering::Relaxed) > 0 {
                    break;
                }
                notified.await;
            }
        })
        .await
        .map_err(|_| "graph retry fixture query start exceeded its 5-second deadline")?;
        Ok(())
    }

    async fn close(mut self) -> TestResult {
        self.stop.cancel();
        if let Some(task) = self.task.as_mut() {
            tokio::time::timeout(Duration::from_secs(5), task)
                .await
                .map_err(|_| "graph retry fixture shutdown exceeded its 5-second deadline")???;
        }
        Ok(())
    }
}

impl Drop for Endpoint {
    fn drop(&mut self) {
        self.stop.cancel();
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}

async fn line(reader: &mut BufReader<TcpStream>) -> io::Result<Option<String>> {
    let mut bytes = Vec::new();
    (&mut *reader)
        .take(128)
        .read_until(b'\n', &mut bytes)
        .await?;
    if bytes.is_empty() {
        return Ok(None);
    }
    if bytes.len() >= 128 || !bytes.ends_with(b"\r\n") {
        return Err(io::Error::other("invalid bounded RESP header"));
    }
    String::from_utf8(bytes).map(Some).map_err(io::Error::other)
}

async fn command(reader: &mut BufReader<TcpStream>) -> io::Result<Option<Vec<Vec<u8>>>> {
    let Some(header) = line(reader).await? else {
        return Ok(None);
    };
    let count: usize = header
        .strip_prefix('*')
        .ok_or_else(|| io::Error::other("expected RESP array"))?
        .trim()
        .parse()
        .map_err(io::Error::other)?;
    if !(1..=16).contains(&count) {
        return Err(io::Error::other("command argument bound"));
    }
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        let header = line(reader)
            .await?
            .ok_or_else(|| io::Error::other("missing bulk header"))?;
        let size: usize = header
            .strip_prefix('$')
            .ok_or_else(|| io::Error::other("expected bulk"))?
            .trim()
            .parse()
            .map_err(io::Error::other)?;
        if size > 2_105_344 {
            return Err(io::Error::other("command byte bound"));
        }
        let mut body = vec![0; size];
        reader.read_exact(&mut body).await?;
        let mut ending = [0; 2];
        reader.read_exact(&mut ending).await?;
        if ending != *b"\r\n" {
            return Err(io::Error::other("bad bulk ending"));
        }
        result.push(body);
    }
    Ok(Some(result))
}

fn encode(value: &redis::Value, output: &mut Vec<u8>) -> io::Result<()> {
    match value {
        redis::Value::Nil => output.extend_from_slice(b"$-1\r\n"),
        redis::Value::Int(value) => output.extend_from_slice(format!(":{value}\r\n").as_bytes()),
        redis::Value::Okay => output.extend_from_slice(b"+OK\r\n"),
        redis::Value::SimpleString(value) => {
            output.extend_from_slice(format!("+{value}\r\n").as_bytes())
        }
        redis::Value::BulkString(bytes) => {
            output.extend_from_slice(format!("${}\r\n", bytes.len()).as_bytes());
            output.extend_from_slice(bytes);
            output.extend_from_slice(b"\r\n");
        }
        redis::Value::Array(values) => {
            output.extend_from_slice(format!("*{}\r\n", values.len()).as_bytes());
            for value in values {
                encode(value, output)?;
            }
        }
        _ => return Err(io::Error::other("unsupported test-proxy RESP response")),
    }
    if output.len() > 4_194_304 {
        return Err(io::Error::other("test proxy response bound"));
    }
    Ok(())
}

async fn serve(socket: TcpStream, state: Arc<State>, backend: Option<String>) -> io::Result<()> {
    let mut upstream = if let Some(url) = backend {
        Some(
            redis::Client::open(url)
                .map_err(io::Error::other)?
                .get_multiplexed_async_connection()
                .await
                .map_err(io::Error::other)?,
        )
    } else {
        None
    };
    let mut reader = BufReader::new(socket);
    for _ in 0..128 {
        let Some(arguments) = command(&mut reader).await? else {
            return Ok(());
        };
        if arguments.first().map(Vec::as_slice) != Some(b"GRAPH.QUERY") {
            reader.get_mut().write_all(b"+OK\r\n").await?;
            continue;
        }
        let number = state
            .queries
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);
        let query = arguments
            .get(2)
            .ok_or_else(|| io::Error::other("missing query"))?;
        {
            let mut observed = state
                .observed
                .lock()
                .map_err(|_| io::Error::other("poisoned test state"))?;
            if observed.len() >= 64 {
                return Err(io::Error::other("test query count bound"));
            }
            observed.push(String::from_utf8(query.clone()).map_err(io::Error::other)?);
        }
        state.query_started.notify_waiters();
        let mut response = Vec::new();
        if let Some(connection) = &mut upstream {
            let mut command = redis::cmd("GRAPH.QUERY");
            for argument in arguments.iter().skip(1) {
                command.arg(argument);
            }
            let result: redis::RedisResult<redis::Value> = command.query_async(connection).await;
            match result {
                Ok(value) => {
                    encode(&value, &mut response)?;
                    if matches!(state.fault, Fault::DropAt(at) if at == number) {
                        state.committed_before_drop.fetch_add(1, Ordering::Relaxed);
                    }
                }
                Err(error) => response.extend_from_slice(
                    format!(
                        "-{} {}\r\n",
                        error.code().unwrap_or("ERR"),
                        error.detail().unwrap_or("rejected")
                    )
                    .as_bytes(),
                ),
            }
        } else {
            response.extend_from_slice(b"+OK\r\n");
        }
        match state.fault {
            Fault::DropAt(at) if at == number => return Ok(()),
            Fault::ErrorAt(at, error) if at == number => {
                response = format!("-{error}\r\n").into_bytes()
            }
            Fault::AlwaysRetry => response = b"-TRYAGAIN temporary\r\n".to_vec(),
            Fault::Hang => std::future::pending::<()>().await,
            _ => {}
        }
        reader.get_mut().write_all(&response).await?;
    }
    Err(io::Error::other("test command-count bound"))
}

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

#[tokio::test]
async fn lost_reply_at_each_stage_reconnects_and_retries_identical_validated_bytes() -> TestResult {
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

#[tokio::test]
async fn custom_non_idempotent_query_gets_no_ambiguous_replay() -> TestResult {
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

#[tokio::test]
async fn retryable_server_error_reconnects_but_permanent_errors_do_not_retry_or_ack() -> TestResult
{
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

#[tokio::test]
async fn retry_budget_is_exact_and_no_failed_write_is_acknowledged() -> TestResult {
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

#[tokio::test]
async fn cancellation_and_overall_deadline_interrupt_pending_network_without_ack() -> TestResult {
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

#[tokio::test]
async fn changed_unacknowledged_tail_is_rejected_before_any_reconnect() -> TestResult {
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

#[tokio::test]
async fn pre_cancelled_and_invalid_options_do_not_open_input_or_checkpoint() -> TestResult {
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

#[tokio::test]
#[ignore = "requires native FALKORDB_TEST_URL; proxy drops a confirmed server write response"]
async fn live_falkordb_ambiguous_write_replay_preserves_nodes_edges_and_checkpoint() -> TestResult {
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
            ("MATCH (r:Resource) RETURN count(r)", 2),
            ("MATCH ()-[r:RELATES_TO]->() RETURN count(r)", 1),
        ] {
            let value: redis::Value = redis::cmd("GRAPH.QUERY")
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
