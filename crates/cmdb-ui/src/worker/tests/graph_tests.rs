use super::*;
use crate::data::Data;
use std::io::Write;
use std::thread;
use std::time::Duration;

#[test]
fn given_unconfigured_graph_when_retried_then_configuration_error_remains_visible() {
    let mut access = GraphAccess::new(None, None, DEFAULT_GRAPH_LIMITS);
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
        DEFAULT_GRAPH_LIMITS,
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
    let mut access = GraphAccess::new(None, None, DEFAULT_GRAPH_LIMITS);
    worker.refresh(Filter::default())?;
    let request = revision(commands.try_recv()?)?;
    events.send(crate::worker::Output {
        revision: Some(request),
        event: access.refresh().await,
    })?;
    events.send(page(request, 12))?;
    let mut data = Data::default();
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
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let endpoint = format!("redis://{}", listener.local_addr()?);
    let server = thread::spawn(move || {
        for (fail, neighborhood) in [(true, true), (false, true), (false, false)] {
            let (socket, _) = listener.accept().map_err(|error| error.to_string())?;
            serve_graph_connection(socket, fail, neighborhood)?;
        }
        Ok::<_, String>(())
    });
    let mut access = GraphAccess::new(
        Some(endpoint),
        Some("inventory".to_owned()),
        DEFAULT_GRAPH_LIMITS,
    );
    let first = access.expand(node("vpc-root")).await;
    assert!(
        matches!(first, WorkerEvent::Graph { error: Some(error), .. } if error.contains("temporarily unavailable"))
    );
    let refreshed = access.refresh().await;
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
