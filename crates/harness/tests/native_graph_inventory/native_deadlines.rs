//! Controlled Unix peers exercise client deadlines without installing FalkorDB.
use super::{Duration, NativeGraph, Stdio, TestResult};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    process::Command,
};
use tokio_util::sync::CancellationToken;

#[tokio::test(flavor = "current_thread")]
async fn given_silent_ping_when_readiness_expires_then_owned_child_is_reaped_and_directory_cleaned()
-> TestResult {
    let (mut graph, listener, directory) = fixture()?;
    let cancel = CancellationToken::new();
    let (outcome, transcript) = tokio::join!(
        async {
            let outcome =
                tokio::time::timeout(Duration::from_secs(3), graph.await_ready_attempts(1)).await;
            cancel.cancel();
            outcome
        },
        serve(listener, cancel.clone(), true)
    );
    let reaped_before_cleanup = graph.child.try_wait()?.is_some();
    graph.terminate_child().await?;
    drop(graph);
    assert!(
        !directory.exists(),
        "failed readiness must release its owned installation directory"
    );
    assert!(
        matches!(outcome, Ok(Err(_))),
        "the readiness budget, not the outer watchdog, must finish"
    );
    assert!(
        reaped_before_cleanup,
        "readiness failure must kill and reap the owned child before returning"
    );
    assert!(
        transcript?.iter().any(|command| command == "PING"),
        "peer must finish setup and actually withhold PING"
    );
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn given_silent_unix_peer_when_connect_or_query_stalls_then_graph_operations_reach_their_deadlines()
-> TestResult {
    for finish_setup in [false, true] {
        let (mut graph, listener, directory) = fixture()?;
        let cancel = CancellationToken::new();
        let (outcome, transcript) = tokio::join!(
            async {
                let outcome =
                    tokio::time::timeout(Duration::from_secs(3), graph_deadlines(&mut graph)).await;
                cancel.cancel();
                outcome
            },
            serve(listener, cancel.clone(), finish_setup)
        );
        graph.stop().await?;
        assert!(
            !directory.exists(),
            "query errors must still permit owned child and directory cleanup"
        );
        outcome??;
        let transcript = transcript?;
        assert!(
            !transcript.is_empty(),
            "deadline case must reach the owned Unix peer"
        );
        if finish_setup {
            assert!(transcript.iter().any(|command| command == "GRAPH.RO_QUERY"));
            assert!(
                transcript
                    .iter()
                    .any(|command| command == inventory_graph::cypher::COMMAND)
            );
        }
    }
    Ok(())
}

async fn graph_deadlines(graph: &mut NativeGraph) -> TestResult {
    let rows = graph
        .rows(inventory_graph::cypher::diagnostics::NODE_IDS)
        .await;
    assert!(
        matches!(&rows, Err(error) if error.downcast_ref::<tokio::time::error::Elapsed>().is_some()),
        "complete read operation must expire at its client deadline"
    );
    let create = graph.select_empty_graph("native_silent_peer").await;
    assert!(
        matches!(&create, Err(error) if error.downcast_ref::<tokio::time::error::Elapsed>().is_some()),
        "complete graph creation must expire at its client deadline"
    );
    Ok(())
}

fn fixture() -> TestResult<(NativeGraph, UnixListener, std::path::PathBuf)> {
    let installation = tempfile::tempdir()?;
    let directory = installation.path().to_path_buf();
    let socket = directory.join("silent.sock");
    let listener = UnixListener::bind(&socket)?;
    // A real owned child makes reap assertions independent of Redis behavior.
    let child = Command::new("sleep")
        .arg("60")
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()?;
    let graph = NativeGraph {
        child,
        endpoint: format!("redis+unix://{}", socket.display()),
        name: "native_deadline_component",
        request_budget: Duration::from_millis(200),
        _installation: installation,
    };
    Ok((graph, listener, directory))
}

async fn serve(
    listener: UnixListener,
    cancel: CancellationToken,
    finish_setup: bool,
) -> TestResult<Vec<String>> {
    let mut transcript = Vec::new();
    for _ in 0..4 {
        let (stream, _) = tokio::select! {
            () = cancel.cancelled() => return Ok(transcript),
            accepted = listener.accept() => accepted?,
        };
        let mut stream = BufReader::new(stream);
        for _ in 0..8 {
            let frame = tokio::select! {
                () = cancel.cancelled() => return Ok(transcript),
                frame = read_frame(&mut stream) => frame?,
            };
            let Some(command) = frame else {
                break;
            };
            let operation = command.first().ok_or("empty Redis command")?;
            let withhold = !finish_setup || operation == "PING" || operation.starts_with("GRAPH.");
            transcript.push(operation.clone());
            if !withhold {
                stream.get_mut().write_all(b"+OK\r\n").await?;
            }
        }
    }
    Err("silent peer exceeded its fixed client budget".into())
}

async fn read_frame(stream: &mut BufReader<UnixStream>) -> TestResult<Option<Vec<String>>> {
    let Some(header) = read_line(stream).await? else {
        return Ok(None);
    };
    let count: usize = header
        .strip_prefix('*')
        .ok_or("expected Redis array")?
        .parse()?;
    if !(1..=16).contains(&count) {
        return Err("Redis test argument count exceeds budget".into());
    }
    let mut fields = Vec::with_capacity(count);
    for _ in 0..count {
        let header = read_line(stream)
            .await?
            .ok_or("Redis bulk length missing")?;
        let length: usize = header
            .strip_prefix('$')
            .ok_or("expected Redis bulk string")?
            .parse()?;
        if length > 4096 {
            return Err("Redis test bulk exceeds budget".into());
        }
        let mut bytes = vec![0; length + 2];
        stream.read_exact(&mut bytes).await?;
        if bytes.get(length..) != Some(b"\r\n") {
            return Err("Redis bulk terminator missing".into());
        }
        bytes.truncate(length);
        fields.push(String::from_utf8(bytes)?);
    }
    Ok(Some(fields))
}

async fn read_line(stream: &mut BufReader<UnixStream>) -> TestResult<Option<String>> {
    let mut bytes = Vec::with_capacity(128);
    for _ in 0..128 {
        let byte = match stream.read_u8().await {
            Ok(byte) => byte,
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof && bytes.is_empty() => {
                return Ok(None);
            }
            Err(error) => return Err(error.into()),
        };
        bytes.push(byte);
        if byte == b'\n' {
            let line = String::from_utf8(bytes)?;
            return Ok(Some(
                line.strip_suffix("\r\n")
                    .ok_or("Redis line terminator missing")?
                    .into(),
            ));
        }
    }
    Err("Redis test header exceeds fixed line budget".into())
}
