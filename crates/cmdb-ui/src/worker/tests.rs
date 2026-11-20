pub(crate) mod isolation;
pub(crate) mod real;
mod shutdown;
pub(crate) mod support;

use super::*;
use std::error::Error;

type TestResult = Result<(), Box<dyn Error>>;

fn harness(
    capacity: usize,
) -> (
    DataWorker,
    CommandReceiver<Command>,
    SyncSender<Output>,
    Cancellation,
) {
    let (commands, receiver) = tokio::sync::mpsc::channel(capacity);
    let (events, outputs) = mpsc::sync_channel(EVENT_CAPACITY);
    let (shutdown, cancellation) = shutdown_channel();
    let worker = DataWorker {
        commands,
        events: Some(outputs),
        revisions: Arc::new(Revisions::default()),
        shutdown,
        thread: None,
    };
    (worker, receiver, events, cancellation)
}

fn page(revision: u64, total: u64) -> Output {
    Output {
        revision: Some(revision),
        event: WorkerEvent::Snapshot {
            counts: Counts {
                resources: total,
                ..Counts::default()
            },
            rows: Vec::new(),
            total,
            error: None,
        },
    }
}

fn node(id: &str) -> GraphNode {
    GraphNode {
        account: "111111111111".to_owned(),
        region: "us-east-1".to_owned(),
        kind: "AWS::EC2::VPC".to_owned(),
        id: id.to_owned(),
    }
}

fn graph(revision: u64, error: &str) -> Output {
    Output {
        revision: Some(revision),
        event: graph_failure(error.to_owned()),
    }
}

fn revision(command: Command) -> Result<u64, String> {
    match command {
        Command::Refresh { revision, .. } | Command::Expand { revision, .. } => Ok(revision),
        Command::Stop => Err("unexpected stop command".to_owned()),
    }
}

#[test]
fn given_new_filter_when_old_page_finishes_last_then_only_latest_page_is_delivered() -> TestResult {
    let (worker, mut commands, events, _cancellation) = harness(2);
    worker.refresh(Filter {
        query: "old".to_owned(),
        ..Filter::default()
    })?;
    let old = revision(commands.try_recv()?)?;
    worker.refresh(Filter {
        query: "new".to_owned(),
        ..Filter::default()
    })?;
    let new = revision(commands.try_recv()?)?;
    events.send(page(new, 7))?;
    events.send(page(old, 99))?;
    let drained = worker.drain();
    assert_eq!(drained.len(), 1);
    assert!(matches!(
        drained.first(),
        Some(WorkerEvent::Snapshot {
            total: 7,
            error: None,
            ..
        })
    ));
    Ok(())
}

#[test]
fn given_full_queue_when_new_filter_is_rejected_then_accepted_page_is_preserved() -> TestResult {
    let (worker, mut commands, events, _cancellation) = harness(1);
    worker.refresh(Filter::default())?;
    let rejection = worker.refresh(Filter {
        query: "rejected".to_owned(),
        ..Filter::default()
    });
    assert_eq!(rejection, Err("inventory worker queue is full".to_owned()));
    events.send(page(revision(commands.try_recv()?)?, 3))?;
    assert!(matches!(
        worker.drain().first(),
        Some(WorkerEvent::Snapshot {
            total: 3,
            error: None,
            ..
        })
    ));
    Ok(())
}

#[test]
fn given_new_selection_when_old_expansion_finishes_then_only_new_graph_is_delivered() -> TestResult
{
    let (worker, mut commands, events, _cancellation) = harness(2);
    worker.expand(node("vpc-old"))?;
    let old = revision(commands.try_recv()?)?;
    worker.expand(node("vpc-new"))?;
    let new = revision(commands.try_recv()?)?;
    events.send(graph(new, "new selection"))?;
    events.send(graph(old, "old selection"))?;
    let drained = worker.drain();
    assert_eq!(drained.len(), 1);
    assert!(
        matches!(drained.first(), Some(WorkerEvent::Graph { error: Some(error), .. }) if error == "new selection")
    );
    Ok(())
}

#[test]
fn given_filter_change_when_prior_expansion_finishes_then_graph_is_not_applied() -> TestResult {
    let (worker, mut commands, events, _cancellation) = harness(2);
    worker.expand(node("vpc-old"))?;
    let old = revision(commands.try_recv()?)?;
    worker.refresh(Filter::default())?;
    let new = revision(commands.try_recv()?)?;
    events.send(graph(old, "old selection"))?;
    events.send(page(new, 4))?;
    let drained = worker.drain();
    assert_eq!(drained.len(), 1);
    assert!(matches!(
        drained.first(),
        Some(WorkerEvent::Snapshot { total: 4, .. })
    ));
    Ok(())
}

#[test]
fn given_pending_page_when_selection_changes_then_page_still_finishes_loading() -> TestResult {
    let (worker, mut commands, events, _cancellation) = harness(2);
    worker.refresh(Filter::default())?;
    let request = revision(commands.try_recv()?)?;
    worker.expand(node("vpc-new"))?;
    events.send(page(request, 5))?;
    assert!(matches!(
        worker.drain().first(),
        Some(WorkerEvent::Snapshot { total: 5, .. })
    ));
    Ok(())
}

#[test]
fn given_offline_graph_when_table_arrives_then_graph_status_precedes_page_selection() -> TestResult
{
    let (worker, mut commands, events, _cancellation) = harness(1);
    worker.refresh(Filter::default())?;
    let request = revision(commands.try_recv()?)?;
    events.send(page(request, 8))?;
    events.send(graph(request, "connection refused"))?;
    let drained = worker.drain();
    assert!(
        matches!(drained.first(), Some(WorkerEvent::Graph { error: Some(error), .. }) if error == "connection refused")
    );
    assert!(matches!(
        drained.last(),
        Some(WorkerEvent::Snapshot {
            total: 8,
            error: None,
            ..
        })
    ));
    Ok(())
}

#[test]
fn given_startup_failure_after_filter_change_then_terminal_error_is_not_filtered() -> TestResult {
    let (worker, _commands, events, _cancellation) = harness(1);
    worker.refresh(Filter::default())?;
    events.send(Output {
        revision: None,
        event: database_error("schema version 13".to_owned()),
    })?;
    assert!(
        matches!(worker.drain().first(), Some(WorkerEvent::Snapshot { error: Some(error), .. }) if error == "schema version 13")
    );
    Ok(())
}

#[test]
fn given_closed_intake_when_command_is_submitted_then_stopped_error_is_explicit() {
    let (worker, commands, _events, _cancellation) = harness(1);
    drop(commands);
    assert_eq!(
        worker.refresh(Filter::default()),
        Err("inventory worker stopped".to_owned())
    );
}

#[tokio::test]
async fn given_active_query_when_shutdown_requested_then_pending_io_is_cancelled() -> TestResult {
    let (shutdown, mut cancellation) = shutdown_channel();
    let (started, ready) = tokio::sync::oneshot::channel();
    let operation = cancellation.run(async {
        started
            .send(())
            .map_err(|()| "query handshake disconnected")?;
        std::future::pending::<Result<(), String>>().await
    });
    let stop = async {
        ready.await?;
        shutdown.stop();
        Ok::<_, Box<dyn Error>>(())
    };
    let (outcome, stopped) = tokio::join!(operation, stop);
    stopped?;
    assert!(outcome.is_none());
    Ok(())
}

#[test]
fn given_full_result_queue_when_worker_dropped_then_blocked_producer_is_released() -> TestResult {
    let (mut worker, _commands, events, _cancellation) = harness(1);
    (0..EVENT_CAPACITY).try_for_each(|_| events.send(page(0, 1)).map_err(Box::new))?;
    let (started, ready) = mpsc::sync_channel(1);
    let (outcome, result) = mpsc::sync_channel(1);
    worker.thread = Some(thread::spawn(move || {
        if started.send(()).is_err() {
            return;
        }
        assert!(outcome.send(events.send(page(0, 2)).is_err()).is_ok());
    }));
    ready.recv_timeout(Duration::from_secs(1))?;
    let (finished, completion) = mpsc::sync_channel(1);
    let dropper = thread::spawn(move || {
        drop(worker);
        assert!(finished.send(()).is_ok());
    });
    completion.recv_timeout(Duration::from_secs(1))?;
    assert!(dropper.join().is_ok());
    assert!(result.recv_timeout(Duration::from_secs(1))?);
    Ok(())
}

#[test]
fn given_full_results_when_ui_drains_then_critical_output_is_preserved() -> TestResult {
    let (worker, _commands, events, _cancellation) = harness(1);
    (0..EVENT_CAPACITY).try_for_each(|_| events.send(page(0, 1)).map_err(Box::new))?;
    let (started, ready) = mpsc::sync_channel(1);
    let producer = thread::spawn(move || {
        if started.send(()).is_err() {
            return;
        }
        assert!(events.send(graph(0, "critical graph failure")).is_ok());
    });
    ready.recv_timeout(Duration::from_secs(1))?;
    assert!(matches!(
        worker.drain().last(),
        Some(WorkerEvent::Snapshot { total: 1, .. })
    ));
    assert!(producer.join().is_ok());
    assert!(
        matches!(worker.drain().first(), Some(WorkerEvent::Graph { error: Some(error), .. }) if error == "critical graph failure")
    );
    Ok(())
}

#[test]
fn given_unconfigured_graph_when_retried_then_configuration_error_remains_visible() {
    let mut access = GraphAccess::new(None, None);
    access.reconnect();
    assert!(
        matches!(access.status(), WorkerEvent::GraphStatus { error: Some(error) } if error.contains("not configured"))
    );
}

#[test]
fn given_invalid_graph_name_when_refreshed_then_configuration_failure_is_classified() {
    let mut access = GraphAccess::new(
        Some("redis://127.0.0.1:6379".to_owned()),
        Some(String::new()),
    );
    access.reconnect();
    assert!(
        matches!(access.status(), WorkerEvent::GraphStatus { error: Some(error) } if error == "invalid graph configuration · graph name")
    );
}

#[test]
fn given_full_queue_when_new_selection_rejected_then_accepted_graph_is_preserved() -> TestResult {
    let (worker, mut commands, events, _cancellation) = harness(1);
    worker.expand(node("vpc-old"))?;
    assert_eq!(
        worker.expand(node("vpc-new")),
        Err("inventory worker queue is full".to_owned())
    );
    events.send(graph(revision(commands.try_recv()?)?, "old selection"))?;
    assert!(
        matches!(worker.drain().first(), Some(WorkerEvent::Graph { error: Some(error), .. }) if error == "old selection")
    );
    Ok(())
}

#[tokio::test]
async fn given_unconfigured_graph_when_user_refreshes_then_sqlite_counts_stay_usable() -> TestResult
{
    let (worker, mut commands, events, _cancellation) = harness(1);
    let mut access = GraphAccess::new(None, None);
    worker.refresh(Filter::default())?;
    let request = revision(commands.try_recv()?)?;
    events.send(Output {
        revision: Some(request),
        event: access.refresh().await,
    })?;
    events.send(page(request, 12))?;
    let mut data = crate::data::Data::default();
    let mut pending_graph = None;
    worker.drain().into_iter().for_each(|event| match event {
        WorkerEvent::Snapshot {
            counts,
            rows,
            total,
            ..
        } => {
            data.set_counts(counts);
            data.replace_page(rows, total);
        }
        WorkerEvent::Graph { graph, error } => pending_graph = Some((graph, error)),
        WorkerEvent::GraphStatus { error } => pending_graph = Some((Graph::default(), error)),
    });
    if let Some((graph, error)) = pending_graph {
        data.set_graph(graph, error);
    }
    assert_eq!(data.total_len(), 12);
    assert!(data.graph_status().contains("not configured"));
    Ok(())
}

// A local RESP peer exercises the real Redis transport and native graph reader.
// The first connection rejects a query; the second returns a real neighborhood.
fn graph_reply(with_edge: bool) -> String {
    if !with_edge {
        return "*3\r\n*0\r\n*0\r\n*0\r\n".to_owned();
    }
    let fields = [
        "111111111111",
        "us-east-1",
        "AWS::EC2::VPC",
        "vpc-root",
        "111111111111",
        "us-east-1",
        "AWS::EC2::VPC",
        "vpc-neighbor",
        "contains",
        "catalog",
    ];
    fields.iter().fold(
        "*3\r\n*0\r\n*1\r\n*10\r\n".to_owned(),
        |mut reply, field| {
            reply.push_str(&format!("${}\r\n{field}\r\n", field.len()));
            reply
        },
    ) + "*0\r\n"
}

fn resp_command(
    reader: &mut std::io::BufReader<std::net::TcpStream>,
) -> Result<Vec<String>, String> {
    use std::io::{BufRead, Read};
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|error| error.to_string())?;
    let count = line
        .trim()
        .strip_prefix('*')
        .ok_or("expected RESP array")?
        .parse::<usize>()
        .map_err(|error| error.to_string())?;
    if count > 16 {
        return Err("too many command arguments".to_owned());
    }
    (0..count)
        .map(|_| {
            line.clear();
            reader
                .read_line(&mut line)
                .map_err(|error| error.to_string())?;
            let size = line
                .trim()
                .strip_prefix('$')
                .ok_or("expected RESP bulk string")?
                .parse::<usize>()
                .map_err(|error| error.to_string())?;
            if size > 1_000_000 {
                return Err("oversized command".to_owned());
            }
            let mut bytes = vec![0; size.checked_add(2).ok_or("command length overflow")?];
            reader
                .read_exact(&mut bytes)
                .map_err(|error| error.to_string())?;
            bytes.truncate(size);
            String::from_utf8(bytes).map_err(|error| error.to_string())
        })
        .collect()
}

fn serve_graph_connection(
    socket: std::net::TcpStream,
    fail: bool,
    neighborhood: bool,
) -> Result<(), String> {
    use std::io::Write;
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| error.to_string())?;
    socket
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| error.to_string())?;
    let mut reader = std::io::BufReader::new(socket);
    let mut queries = 0_u32;
    for _ in 0..8 {
        let command = resp_command(&mut reader)?;
        let response = match command.first().map(String::as_str) {
            Some("CLIENT" | "SELECT") => "+OK\r\n".to_owned(),
            Some("GRAPH.RO_QUERY") if fail => "-ERR temporarily unavailable\r\n".to_owned(),
            Some("GRAPH.RO_QUERY") => {
                queries = queries.saturating_add(1);
                graph_reply(neighborhood && queries == 1)
            }
            other => return Err(format!("unexpected command {other:?}")),
        };
        reader
            .get_mut()
            .write_all(response.as_bytes())
            .map_err(|error| error.to_string())?;
        if (fail && command.first().is_some_and(|name| name == "GRAPH.RO_QUERY"))
            || queries == if neighborhood { 2 } else { 1 }
        {
            return Ok(());
        }
    }
    Err("graph peer command budget exhausted".to_owned())
}

#[tokio::test]
async fn given_graph_query_failure_when_refreshing_then_recovered_neighborhood_is_not_discarded()
-> TestResult {
    // Given: a native transport peer that fails once, then recovers.
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let endpoint = format!("redis://{}", listener.local_addr()?);
    let server = thread::spawn(move || {
        for (fail, neighborhood) in [(true, true), (false, true), (false, false)] {
            let (socket, _) = listener.accept().map_err(|error| error.to_string())?;
            serve_graph_connection(socket, fail, neighborhood)?;
        }
        Ok::<_, String>(())
    });
    let mut access = GraphAccess::new(Some(endpoint), Some("inventory".to_owned()));
    let first = access.expand(node("vpc-root")).await;
    assert!(
        matches!(first, WorkerEvent::Graph { error: Some(error), .. } if error.contains("temporarily unavailable"))
    );
    // When: refresh retries the previous selection independently of SQLite.
    let refreshed = access.refresh().await;
    // Then: the actual recovered graph, not an empty status event, reaches UI.
    let WorkerEvent::Graph { graph, error } = refreshed else {
        return Err("expected graph event".into());
    };
    assert_eq!(error, None);
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
    assert_eq!(
        graph.edges.first().map(|edge| edge.provenance.as_str()),
        Some("catalog")
    );
    server.join().map_err(|_| "graph peer panicked")??;
    Ok(())
}

#[test]
fn given_full_or_disconnected_intake_when_checking_worker_then_only_disconnection_is_closed()
-> TestResult {
    let (worker, commands, _events, _cancellation) = harness(1);
    assert!(!worker.is_closed());
    worker.refresh(Filter::default())?;
    assert_eq!(
        worker.refresh(Filter::default()),
        Err("inventory worker queue is full".to_owned())
    );
    assert!(!worker.is_closed());
    drop(commands);
    assert!(worker.is_closed());
    Ok(())
}
