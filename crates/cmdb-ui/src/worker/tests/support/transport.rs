use super::*;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

pub(crate) struct FaultableEndpoint {
    pub endpoint: String,
    online: Arc<AtomicBool>,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    thread: Option<JoinHandle<Result<(), String>>>,
}

impl FaultableEndpoint {
    pub fn start(upstream: &str) -> TestResult<Self> {
        let client = redis::Client::open(upstream)?;
        let redis::ConnectionAddr::Tcp(host, port) = client.get_connection_info().addr() else {
            return Err("native graph fault fixture requires a TCP endpoint".into());
        };
        let target = (host.clone(), *port);
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let endpoint = format!("redis://{}", listener.local_addr()?);
        let online = Arc::new(AtomicBool::new(true));
        let mode = Arc::clone(&online);
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let thread = thread::spawn(move || {
            let runtime = Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| error.to_string())?;
            runtime
                .block_on(serve(listener, target, mode, stopped))
                .map_err(|error| error.to_string())
        });
        Ok(Self {
            endpoint,
            online,
            stop: Some(stop),
            thread: Some(thread),
        })
    }

    pub fn set_online(&self, online: bool) {
        self.online.store(online, Ordering::Release);
    }

    pub fn finish(mut self) -> TestResult {
        self.stop();
        self.thread
            .take()
            .ok_or("native graph proxy thread missing")?
            .join()
            .map_err(|_| "native graph proxy panicked")??;
        Ok(())
    }

    fn stop(&mut self) {
        if let Some(stop) = self.stop.take()
            && stop.send(()).is_err()
        {
            eprintln!("native graph proxy already stopped");
        }
    }
}

impl Drop for FaultableEndpoint {
    fn drop(&mut self) {
        self.stop();
        if let Some(thread) = self.thread.take() {
            match thread.join() {
                Ok(Ok(())) => {}
                Ok(Err(error)) => eprintln!("native graph proxy cleanup failed: {error}"),
                Err(_) => eprintln!("native graph proxy panicked during cleanup"),
            }
        }
    }
}

async fn serve(
    listener: TcpListener,
    target: (String, u16),
    online: Arc<AtomicBool>,
    mut stopped: tokio::sync::oneshot::Receiver<()>,
) -> TestResult {
    let listener = tokio::net::TcpListener::from_std(listener)?;
    for _ in 0..256 {
        let (mut stream, _) = tokio::select! {
            _ = &mut stopped => return Ok(()),
            accepted = listener.accept() => accepted?,
        };
        let exchange = async {
            if online.load(Ordering::Acquire) {
                let mut upstream =
                    tokio::net::TcpStream::connect((target.0.as_str(), target.1)).await?;
                tokio::io::copy_bidirectional(&mut stream, &mut upstream).await?;
            } else {
                fail_query(stream).await?;
            }
            Ok::<(), Box<dyn std::error::Error>>(())
        };
        tokio::select! {
            _ = &mut stopped => return Ok(()),
            result = tokio::time::timeout(Duration::from_secs(5), exchange) => result??,
        }
    }
    Err("native graph proxy connection budget exhausted".into())
}

async fn fail_query(stream: tokio::net::TcpStream) -> TestResult {
    let mut reader = tokio::io::BufReader::new(stream);
    for _ in 0..8 {
        let command = read_command(&mut reader).await?;
        match command.first().map(String::as_str) {
            Some("GRAPH.RO_QUERY") => {
                reader.get_mut().shutdown().await?;
                return Ok(());
            }
            Some("CLIENT" | "SELECT") => {
                reader.get_mut().write_all(b"+OK\r\n").await?;
                reader.get_mut().flush().await?;
            }
            other => return Err(format!("unexpected native graph handshake: {other:?}").into()),
        }
    }
    Err("native graph proxy handshake budget exhausted".into())
}

async fn read_command(
    reader: &mut tokio::io::BufReader<tokio::net::TcpStream>,
) -> TestResult<Vec<String>> {
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    let count: usize = line
        .strip_prefix('*')
        .ok_or("expected RESP array")?
        .trim()
        .parse()?;
    if count > 16 {
        return Err("RESP command exceeds test bound".into());
    }
    let mut command = Vec::with_capacity(count);
    for _ in 0..count {
        line.clear();
        reader.read_line(&mut line).await?;
        let length: usize = line
            .strip_prefix('$')
            .ok_or("expected RESP bulk string")?
            .trim()
            .parse()?;
        if length > 65_536 {
            return Err("RESP value exceeds test bound".into());
        }
        let mut bytes = vec![0; length];
        reader.read_exact(&mut bytes).await?;
        let mut end = [0; 2];
        reader.read_exact(&mut end).await?;
        assert_eq!(end, *b"\r\n");
        command.push(String::from_utf8(bytes)?);
    }
    Ok(command)
}

pub(crate) fn refresh_batch(worker: &DataWorker) -> TestResult<Vec<WorkerEvent>> {
    let events = worker
        .events
        .as_ref()
        .ok_or("worker output receiver missing")?;
    let started = std::time::Instant::now();
    let mut outputs = Vec::new();
    let (mut graph, mut snapshot) = (false, false);
    for _ in 0..EVENT_CAPACITY {
        let remaining = Duration::from_secs(5).saturating_sub(started.elapsed());
        let output = events.recv_timeout(remaining)?;
        graph |= matches!(output.event, WorkerEvent::Graph { .. });
        snapshot |= matches!(output.event, WorkerEvent::Snapshot { .. });
        outputs.push(output);
        if graph && snapshot {
            return Ok(worker.drain_test_events(outputs.into_iter()));
        }
    }
    Err("native graph refresh output budget exhausted".into())
}

pub(crate) fn first_refresh_snapshot(worker: &DataWorker) -> TestResult<Vec<WorkerEvent>> {
    let output = worker
        .events
        .as_ref()
        .ok_or("worker output receiver missing")?
        .recv_timeout(Duration::from_secs(5))?;
    if !matches!(
        &output.event,
        WorkerEvent::Snapshot {
            total: 0,
            error: None,
            ..
        }
    ) {
        return Err(format!(
            "expected the empty refresh Snapshot first: {:?}",
            output.event
        )
        .into());
    }
    Ok(worker.drain_test_events(std::iter::once(output)))
}

pub(crate) fn refresh_graph(
    worker: &DataWorker,
    expected_error: Option<&str>,
) -> TestResult<Vec<WorkerEvent>> {
    let events = worker
        .events
        .as_ref()
        .ok_or("worker output receiver missing")?;
    let started = std::time::Instant::now();
    let mut outputs = Vec::new();
    for _ in 0..EVENT_CAPACITY {
        let output =
            events.recv_timeout(Duration::from_secs(5).saturating_sub(started.elapsed()))?;
        let complete = if let WorkerEvent::Graph { error, .. } = &output.event {
            assert_eq!(error.as_deref(), expected_error);
            true
        } else {
            false
        };
        if complete && !worker.revisions.accepts(&output) {
            return Err(format!(
                "refresh Graph {:?} was fenced after its Snapshot",
                output.revision
            )
            .into());
        }
        outputs.push(output);
        if complete {
            return Ok(worker.drain_test_events(outputs.into_iter()));
        }
    }
    Err("native graph refresh output budget exhausted".into())
}
