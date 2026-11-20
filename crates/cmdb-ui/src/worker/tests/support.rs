//! Deterministic ordering around the production runner, never fabricated outputs.
use super::super::*;
use super::real::fixtures::TestResult;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

pub(crate) struct QueuedWorker {
    pub worker: DataWorker,
    pub start: mpsc::Sender<()>,
}

impl QueuedWorker {
    pub fn release(self) -> TestResult<DataWorker> {
        self.start.send(())?;
        Ok(self.worker)
    }
}

pub(crate) fn queued(
    config: WorkerConfig,
    command_capacity: usize,
    event_capacity: usize,
) -> QueuedWorker {
    let (commands, receiver) = tokio::sync::mpsc::channel(command_capacity);
    let (events, outputs) = mpsc::sync_channel(event_capacity);
    let revisions = Arc::new(Revisions::default());
    let (shutdown, cancellation) = shutdown_channel();
    let context = runner::WorkerContext::new(
        receiver,
        events,
        SignalToUI::default(),
        cancellation,
        Arc::clone(&revisions),
    );
    let (start, gate) = mpsc::channel();
    let thread = thread::spawn(move || {
        if gate.recv().is_ok() {
            runner::run_worker(config, context);
        }
    });
    QueuedWorker {
        worker: DataWorker {
            commands,
            events: Some(outputs),
            revisions,
            shutdown,
            thread: Some(thread),
        },
        start,
    }
}

pub(crate) fn receive_startup_graph(worker: &DataWorker) -> TestResult {
    let outputs = worker
        .events
        .as_ref()
        .ok_or("worker output receiver missing")?;
    assert!(matches!(
        outputs.recv_timeout(Duration::from_secs(2))?,
        Output {
            revision: Some(0),
            event: WorkerEvent::GraphStatus { .. }
        }
    ));
    Ok(())
}

pub(crate) fn close_worker(worker: DataWorker) -> TestResult {
    let started = std::time::Instant::now();
    drop(worker);
    if started.elapsed() >= Duration::from_secs(2) {
        return Err("real worker failed to cancel and join within two seconds".into());
    }
    Ok(())
}

/// An external RESP transport peer which acknowledges the *actual* graph read
/// and deliberately withholds its response. Socket EOF proves runtime cleanup.
pub(crate) struct BlockedGraph {
    pub endpoint: String,
    received: Receiver<Vec<String>>,
    closed: Receiver<Result<(), String>>,
    thread: Option<JoinHandle<()>>,
}

impl BlockedGraph {
    pub fn start() -> TestResult<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let endpoint = format!("redis://{}", listener.local_addr()?);
        let (sent, received) = mpsc::channel();
        let (closed_sender, closed) = mpsc::channel();
        let thread = thread::spawn(move || {
            let result = serve(listener, sent).map_err(|error| error.to_string());
            let _ = closed_sender.send(result);
        });
        Ok(Self {
            endpoint,
            received,
            closed,
            thread: Some(thread),
        })
    }

    pub fn config(&self, database: PathBuf) -> WorkerConfig {
        WorkerConfig {
            database: Some(database),
            graph_url: Some(self.endpoint.clone()),
            graph_name: Some("blocked_shutdown_query".into()),
        }
    }

    pub fn await_query(&self) -> TestResult<Vec<String>> {
        Ok(self.received.recv_timeout(Duration::from_secs(3))?)
    }

    pub fn finish(mut self) -> TestResult {
        self.closed.recv_timeout(Duration::from_secs(2))??;
        self.thread
            .take()
            .ok_or("RESP peer thread missing")?
            .join()
            .map_err(|_| "RESP peer panicked")?;
        Ok(())
    }
}

impl Drop for BlockedGraph {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take()
            && thread.join().is_err()
        {
            eprintln!("owned RESP peer panicked during cleanup");
        }
    }
}

fn serve(listener: TcpListener, sent: mpsc::Sender<Vec<String>>) -> TestResult {
    let deadline = std::time::Instant::now()
        .checked_add(Duration::from_secs(5))
        .ok_or("deadline overflow")?;
    let stream = (0..20_000_000)
        .find_map(|_| match listener.accept() {
            Ok((stream, _)) => Some(Ok(stream)),
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    && std::time::Instant::now() < deadline =>
            {
                thread::yield_now();
                None
            }
            Err(error) => Some(Err(error)),
        })
        .ok_or("RESP peer accept budget exhausted")??;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut reader = BufReader::new(stream);
    for _ in 0..8 {
        let command = read_command(&mut reader)?;
        if command.first().is_some_and(|name| name == "GRAPH.RO_QUERY") {
            sent.send(command)?;
            let mut remaining = Vec::new();
            reader.read_to_end(&mut remaining)?;
            assert!(
                remaining.is_empty(),
                "cancelled query must close without another command"
            );
            return Ok(());
        }
        assert_eq!(command.first().map(String::as_str), Some("CLIENT"));
        reader.get_mut().write_all(b"+OK\r\n")?;
        reader.get_mut().flush()?;
    }
    Err("graph read was not received within handshake budget".into())
}

fn read_command(reader: &mut BufReader<TcpStream>) -> TestResult<Vec<String>> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let count: usize = line
        .strip_prefix('*')
        .ok_or("expected RESP array")?
        .trim()
        .parse()?;
    if count > 16 {
        return Err("RESP command exceeds test bound".into());
    }
    (0..count)
        .map(|_| {
            line.clear();
            reader.read_line(&mut line)?;
            let length: usize = line
                .strip_prefix('$')
                .ok_or("expected RESP bulk string")?
                .trim()
                .parse()?;
            if length > 65_536 {
                return Err("RESP value exceeds test bound".into());
            }
            let mut bytes = vec![0; length];
            reader.read_exact(&mut bytes)?;
            let mut end = [0; 2];
            reader.read_exact(&mut end)?;
            assert_eq!(end, *b"\r\n");
            Ok(String::from_utf8(bytes)?)
        })
        .collect()
}
